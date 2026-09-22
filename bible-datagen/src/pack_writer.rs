//! Pack writer module — generates .bible.bin and .offsets.bin data packs.
//!
//! Reads verse text, serializes each record independently via `rkyv`,
//! builds a per-verse offset table, and writes the final binary files
//! with format-version headers and BLAKE3 integrity checksum.

use bible_core::types::{
    VerseRecord, OFFSETS_HEADER_SIZE, OFFSETS_MAGIC, OFFSET_MISSING, PACK_FORMAT_VERSION,
    PACK_HEADER_SIZE, PACK_MAGIC,
};
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

/// Alignment boundary for archived records within the payload.
/// rkyv archives perform best when 8-byte aligned.
const RECORD_ALIGNMENT: usize = 8;

/// Intermediate verse entry before serialization.
struct VerseEntry {
    global_index: u32,
    text: String,
}

/// Builds a data pack from individual verse entries.
///
/// Usage:
/// ```ignore
/// let mut writer = PackWriter::new("KJV", Path::new("./output"));
/// writer.add_verse(0, "In the beginning God created...");
/// writer.add_verse(1, "And the earth was without form...");
/// writer.finalize()?;
/// ```
pub struct PackWriter {
    translation_code: String,
    output_dir: PathBuf,
    entries: Vec<VerseEntry>,
    max_global_index: Option<u32>,
}

impl PackWriter {
    /// Create a new PackWriter for the given translation code.
    pub fn new(translation_code: &str, output_dir: &Path) -> Self {
        Self {
            translation_code: translation_code.to_owned(),
            output_dir: output_dir.to_owned(),
            entries: Vec::new(),
            max_global_index: None,
        }
    }

    /// Add a verse to the pack.
    pub fn add_verse(&mut self, global_index: u32, text: &str) {
        self.max_global_index = Some(
            self.max_global_index
                .map_or(global_index, |m| m.max(global_index)),
        );
        self.entries.push(VerseEntry {
            global_index,
            text: text.to_owned(),
        });
    }

    /// Serialize all accumulated verses and write the pack files.
    ///
    /// Produces `<translation>.bible.bin` and `<translation>.offsets.bin`
    /// in the output directory.
    pub fn finalize(mut self) -> Result<(), io::Error> {
        // Sort by index so payload order matches the offset table.
        self.entries.sort_unstable_by_key(|e| e.global_index);

        let verse_count = u32::try_from(self.entries.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many verse records for u32 verse_count",
            )
        })?;

        // Offset table: entry_count slots, each (offset_u32, length_u32).
        // entry_count = max_global_index + 1 so every valid index has a slot.
        let entry_count = self.max_global_index.map_or(0u32, |m| m + 1);
        let mut offset_table: Vec<(u32, u32)> = vec![(OFFSET_MISSING, 0); entry_count as usize];

        // ── Build payload ───────────────────────────────────────────
        let mut payload: Vec<u8> = Vec::with_capacity(self.entries.len() * 64);

        for entry in &self.entries {
            // Align to RECORD_ALIGNMENT within the payload.
            let padding =
                (RECORD_ALIGNMENT - (payload.len() % RECORD_ALIGNMENT)) % RECORD_ALIGNMENT;
            payload.resize(payload.len() + padding, 0u8);

            let record = VerseRecord {
                text: entry.text.clone(),
            };

            let archived = rkyv::to_bytes::<rkyv::rancor::Error>(&record).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "rkyv serialize failed for verse {}: {:?}",
                        entry.global_index, e
                    ),
                )
            })?;

            let offset_in_payload = u32::try_from(payload.len())
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "payload exceeds 4 GiB"))?;
            let record_len = u32::try_from(archived.len()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("archived verse {} exceeds 4 GiB", entry.global_index),
                )
            })?;

            offset_table[entry.global_index as usize] = (offset_in_payload, record_len);
            payload.extend_from_slice(&archived);
        }

        // ── Checksum ────────────────────────────────────────────────
        let checksum = blake3::hash(&payload);

        // ── Write .bible.bin ────────────────────────────────────────
        fs::create_dir_all(&self.output_dir)?;

        let bible_path = self
            .output_dir
            .join(format!("{}.bible.bin", self.translation_code));
        let offsets_path = self
            .output_dir
            .join(format!("{}.offsets.bin", self.translation_code));

        {
            let mut w = BufWriter::new(File::create(&bible_path)?);
            // Header: magic(4) + version(4) + verse_count(4) + blake3(32)
            w.write_all(&PACK_MAGIC)?;
            w.write_all(&PACK_FORMAT_VERSION.to_le_bytes())?;
            w.write_all(&verse_count.to_le_bytes())?;
            w.write_all(checksum.as_bytes())?;
            debug_assert_eq!(
                4 + 4 + 4 + 32,
                PACK_HEADER_SIZE,
                "header size constant mismatch"
            );
            // Payload
            w.write_all(&payload)?;
            w.flush()?;
        }

        // ── Write .offsets.bin ──────────────────────────────────────
        {
            let mut w = BufWriter::new(File::create(&offsets_path)?);
            // Header: magic(4) + version(4) + entry_count(4)
            w.write_all(&OFFSETS_MAGIC)?;
            w.write_all(&PACK_FORMAT_VERSION.to_le_bytes())?;
            w.write_all(&entry_count.to_le_bytes())?;
            debug_assert_eq!(
                4 + 4 + 4,
                OFFSETS_HEADER_SIZE,
                "offsets header size constant mismatch"
            );
            // Entries: (offset: u32 LE, length: u32 LE) per slot
            for &(offset, length) in &offset_table {
                w.write_all(&offset.to_le_bytes())?;
                w.write_all(&length.to_le_bytes())?;
            }
            w.flush()?;
        }

        // ── Write metadata.json ─────────────────────────────────────
        let metadata_path = self
            .output_dir
            .join(format!("{}.metadata.json", self.translation_code));
        let metadata_json = format!(
            "{{\n  \"name\": \"{}\",\n  \"format_version\": {},\n  \"verse_count\": {},\n  \"checksum\": \"{}\"\n}}",
            self.translation_code, PACK_FORMAT_VERSION, verse_count, checksum.to_hex()
        );
        fs::write(&metadata_path, metadata_json)?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn round_trip_small_pack() {
        let dir = std::env::temp_dir().join("sermon_pack_test");
        let _ = fs::remove_dir_all(&dir);

        let mut writer = PackWriter::new("TEST", &dir);
        writer.add_verse(0, "In the beginning God created the heavens and the earth.");
        writer.add_verse(1, "And the earth was without form, and void.");
        writer.add_verse(4, "And God said, Let there be light.");
        writer.finalize().unwrap();

        // Verify .bible.bin header
        let mut bible_data = Vec::new();
        File::open(dir.join("TEST.bible.bin"))
            .unwrap()
            .read_to_end(&mut bible_data)
            .unwrap();
        assert_eq!(&bible_data[0..4], b"HVBB");
        let version = u32::from_le_bytes(bible_data[4..8].try_into().unwrap());
        assert_eq!(version, PACK_FORMAT_VERSION);
        let vc = u32::from_le_bytes(bible_data[8..12].try_into().unwrap());
        assert_eq!(vc, 3); // 3 verses added

        // Verify checksum
        let stored_hash: [u8; 32] = bible_data[12..44].try_into().unwrap();
        let payload = &bible_data[PACK_HEADER_SIZE..];
        let computed = blake3::hash(payload);
        assert_eq!(stored_hash, *computed.as_bytes());

        // Verify .offsets.bin header
        let mut offsets_data = Vec::new();
        File::open(dir.join("TEST.offsets.bin"))
            .unwrap()
            .read_to_end(&mut offsets_data)
            .unwrap();
        assert_eq!(&offsets_data[0..4], b"HVBO");
        let entry_count = u32::from_le_bytes(offsets_data[8..12].try_into().unwrap());
        assert_eq!(entry_count, 5); // max_index=4, so 5 slots

        // Verify that indices 2 and 3 are marked missing
        let slot2_offset = u32::from_le_bytes(
            offsets_data[OFFSETS_HEADER_SIZE + 2 * 8..OFFSETS_HEADER_SIZE + 2 * 8 + 4]
                .try_into()
                .unwrap(),
        );
        assert_eq!(slot2_offset, OFFSET_MISSING);

        // Verify index 0 has a valid offset
        let slot0_offset = u32::from_le_bytes(
            offsets_data[OFFSETS_HEADER_SIZE..OFFSETS_HEADER_SIZE + 4]
                .try_into()
                .unwrap(),
        );
        assert_ne!(slot0_offset, OFFSET_MISSING);

        let _ = fs::remove_dir_all(&dir);
    }
}
