//! Translation data pack store — mmap-based zero-copy verse access.
//!
//! Manages loaded translation data packs. Each pack consists of:
//! - `*.bible.bin`   — rkyv-archived verse text with format-version header
//! - `*.offsets.bin` — GlobalVerseIndex → (offset, length) table
//!
//! Uses `memmap2` for zero-copy disk access. Verse text is read directly
//! from memory-mapped pages — no heap allocation on the lookup path.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};

use memmap2::Mmap;

use crate::error::EngineError;
use crate::types::{
    GlobalVerseIndex, TranslationCode, VerseRecord, OFFSETS_HEADER_SIZE, OFFSETS_MAGIC,
    OFFSET_MISSING, PACK_FORMAT_VERSION, PACK_HEADER_SIZE, PACK_MAGIC,
};

/// A single entry in the offset table: byte position and length within the payload.
#[derive(Debug, Clone, Copy)]
struct OffsetEntry {
    /// Byte offset within the payload (after the 44-byte bible.bin header).
    offset: u32,
    /// Byte length of the archived record.
    length: u32,
}

/// A loaded translation data pack.
///
/// Holds the memory-mapped file data and offset table for a single
/// Bible translation. The mmap'd bytes are accessed via zero-copy
/// rkyv deserialization after initial validation.
pub struct LoadedTranslation {
    /// Translation identifier (e.g., "KJV").
    pub code: TranslationCode,

    /// Whether this pack has been validated (BLAKE3 checksum verified).
    pub validated: bool,

    /// Format version read from the pack header.
    pub format_version: u32,

    /// Total number of verses in this pack.
    pub verse_count: u32,

    /// Memory-mapped .bible.bin file.
    mmap: Arc<Mmap>,

    /// Per-verse offset table: indexed by GlobalVerseIndex.
    /// Each entry gives the (offset, length) within the payload.
    offsets: Vec<OffsetEntry>,
}

/// Thread-safe registry of loaded translations.
///
/// Switching the active translation is an O(1) swap of an `Arc`.
pub struct TranslationStore {
    translations: Arc<RwLock<HashMap<String, Arc<LoadedTranslation>>>>,
}

impl TranslationStore {
    /// Create a new empty translation store.
    pub fn new() -> Self {
        TranslationStore {
            translations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a translation data pack from disk.
    ///
    /// Validates the BLAKE3 checksum and format version before accepting.
    /// Returns `EngineError::CorruptDataPack` on checksum mismatch,
    /// `EngineError::UnsupportedPackVersion` on version mismatch.
    pub fn load_translation(
        &self,
        code: &str,
        bible_path: &Path,
        offsets_path: &Path,
    ) -> Result<(), EngineError> {
        // ── 1. Read and validate .bible.bin header ──────────────────
        let bible_file = File::open(bible_path)?;

        // Safety: mmap is safe as long as the file isn't modified externally
        // while mapped. These are immutable data packs.
        let mmap = unsafe { Mmap::map(&bible_file)? };

        if mmap.len() < PACK_HEADER_SIZE {
            return Err(EngineError::CorruptDataPack {
                translation: format!("{}: file too small for header", code),
            });
        }

        // Magic bytes
        if mmap[0..4] != PACK_MAGIC {
            return Err(EngineError::CorruptDataPack {
                translation: format!("{}: invalid magic bytes", code),
            });
        }

        // Format version
        let version = u32::from_le_bytes([mmap[4], mmap[5], mmap[6], mmap[7]]);
        if version != PACK_FORMAT_VERSION {
            return Err(EngineError::UnsupportedPackVersion {
                found: version,
                expected: PACK_FORMAT_VERSION,
            });
        }

        // Verse count
        let verse_count = u32::from_le_bytes([mmap[8], mmap[9], mmap[10], mmap[11]]);

        // ── 2. BLAKE3 checksum verification ─────────────────────────
        let stored_hash: [u8; 32] = {
            let mut h = [0u8; 32];
            h.copy_from_slice(&mmap[12..44]);
            h
        };
        let payload = &mmap[PACK_HEADER_SIZE..];
        let computed = blake3::hash(payload);
        if stored_hash != *computed.as_bytes() {
            return Err(EngineError::CorruptDataPack {
                translation: format!("{}: BLAKE3 checksum mismatch", code),
            });
        }

        // ── 3. Read and validate .offsets.bin ───────────────────────
        let offsets = load_offset_table(code, offsets_path)?;

        // ── 4. Register ─────────────────────────────────────────────
        let translation = LoadedTranslation {
            code: TranslationCode::new(code),
            validated: true,
            format_version: version,
            verse_count,
            mmap: Arc::new(mmap),
            offsets,
        };

        let mut registry = self
            .translations
            .write()
            .map_err(|_| EngineError::TranslationNotFound("lock poisoned".to_string()))?;
        registry.insert(code.to_string(), Arc::new(translation));

        Ok(())
    }

    /// Look up a verse by translation code and global verse index.
    ///
    /// Returns the verse text. The text is copied out of the mmap'd data
    /// to avoid holding the read lock across the return boundary.
    pub fn get_verse(&self, code: &str, index: GlobalVerseIndex) -> Result<String, EngineError> {
        let registry = self
            .translations
            .read()
            .map_err(|_| EngineError::TranslationNotFound("lock poisoned".to_string()))?;

        let translation = registry
            .get(code)
            .ok_or_else(|| EngineError::TranslationNotFound(code.to_string()))?;

        let idx = index.as_u32() as usize;
        if idx >= translation.offsets.len() {
            return Err(EngineError::VerseNotFound {
                book: String::new(),
                chapter: 0,
                verse: 0,
            });
        }

        let entry = &translation.offsets[idx];
        if entry.offset == OFFSET_MISSING {
            return Err(EngineError::VerseNotFound {
                book: String::new(),
                chapter: 0,
                verse: 0,
            });
        }

        // Slice into the mmap'd payload.
        let payload_start = PACK_HEADER_SIZE + entry.offset as usize;
        let payload_end = payload_start + entry.length as usize;

        if payload_end > translation.mmap.len() {
            return Err(EngineError::CorruptDataPack {
                translation: format!(
                    "{}: offset out of bounds for index {}",
                    code,
                    index.as_u32()
                ),
            });
        }

        let record_bytes = &translation.mmap[payload_start..payload_end];

        // Zero-copy access via rkyv.
        let archived =
            rkyv::access::<rkyv::Archived<VerseRecord>, rkyv::rancor::Error>(record_bytes)
                .map_err(|e| EngineError::CorruptDataPack {
                    translation: format!("{}: rkyv access failed: {:?}", code, e),
                })?;

        // Copy the text out so we can release the read lock.
        Ok(archived.text.as_str().to_string())
    }

    /// Check if a translation is loaded.
    pub fn is_loaded(&self, code: &str) -> bool {
        self.translations
            .read()
            .map(|t| t.contains_key(code))
            .unwrap_or(false)
    }

    /// List all loaded translation codes.
    pub fn loaded_translations(&self) -> Vec<String> {
        self.translations
            .read()
            .map(|t| t.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Unload a translation, freeing its memory-mapped resources.
    pub fn unload_translation(&self, code: &str) -> Result<(), EngineError> {
        let mut translations = self
            .translations
            .write()
            .map_err(|_| EngineError::TranslationNotFound("lock poisoned".to_string()))?;

        if translations.remove(code).is_some() {
            Ok(())
        } else {
            Err(EngineError::TranslationNotFound(code.to_string()))
        }
    }
}

impl Default for TranslationStore {
    fn default() -> Self {
        Self::new()
    }
}

// ── Internal helpers ────────────────────────────────────────────────

/// Read and parse the .offsets.bin file.
fn load_offset_table(code: &str, path: &Path) -> Result<Vec<OffsetEntry>, EngineError> {
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data.len() < OFFSETS_HEADER_SIZE {
        return Err(EngineError::CorruptDataPack {
            translation: format!("{}: offsets file too small", code),
        });
    }

    // Magic
    if data[0..4] != OFFSETS_MAGIC {
        return Err(EngineError::CorruptDataPack {
            translation: format!("{}: invalid offsets magic", code),
        });
    }

    // Version
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version != PACK_FORMAT_VERSION {
        return Err(EngineError::UnsupportedPackVersion {
            found: version,
            expected: PACK_FORMAT_VERSION,
        });
    }

    // Entry count
    let entry_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;

    let expected_size = OFFSETS_HEADER_SIZE + entry_count * 8;
    if data.len() < expected_size {
        return Err(EngineError::CorruptDataPack {
            translation: format!(
                "{}: offsets file truncated (expected {} bytes, got {})",
                code,
                expected_size,
                data.len()
            ),
        });
    }

    let mut offsets = Vec::with_capacity(entry_count);
    for i in 0..entry_count {
        let base = OFFSETS_HEADER_SIZE + i * 8;
        let offset =
            u32::from_le_bytes([data[base], data[base + 1], data[base + 2], data[base + 3]]);
        let length = u32::from_le_bytes([
            data[base + 4],
            data[base + 5],
            data[base + 6],
            data[base + 7],
        ]);
        offsets.push(OffsetEntry { offset, length });
    }

    Ok(offsets)
}

/// Validate a pack header's magic bytes and format version.
///
/// Returns the verse count from the header, or an error.
pub fn validate_pack_header(header_bytes: &[u8]) -> Result<u32, EngineError> {
    if header_bytes.len() < 12 {
        return Err(EngineError::CorruptDataPack {
            translation: "header too short".to_string(),
        });
    }

    // Check magic bytes
    if header_bytes[0..4] != PACK_MAGIC {
        return Err(EngineError::CorruptDataPack {
            translation: "invalid magic bytes".to_string(),
        });
    }

    // Check format version
    let version = u32::from_le_bytes([
        header_bytes[4],
        header_bytes[5],
        header_bytes[6],
        header_bytes[7],
    ]);
    if version != PACK_FORMAT_VERSION {
        return Err(EngineError::UnsupportedPackVersion {
            found: version,
            expected: PACK_FORMAT_VERSION,
        });
    }

    // Read verse count
    let verse_count = u32::from_le_bytes([
        header_bytes[8],
        header_bytes[9],
        header_bytes[10],
        header_bytes[11],
    ]);

    Ok(verse_count)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn empty_store() {
        let store = TranslationStore::new();
        assert!(!store.is_loaded("KJV"));
        assert!(store.loaded_translations().is_empty());
    }

    #[test]
    fn valid_header() {
        let mut header = Vec::new();
        header.extend_from_slice(&PACK_MAGIC);
        header.extend_from_slice(&PACK_FORMAT_VERSION.to_le_bytes());
        header.extend_from_slice(&31102u32.to_le_bytes());

        let result = validate_pack_header(&header);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 31102);
    }

    #[test]
    fn invalid_magic() {
        let mut header = vec![0u8; 12];
        header[0..4].copy_from_slice(b"XXXX");
        assert!(validate_pack_header(&header).is_err());
    }

    #[test]
    fn wrong_version() {
        let mut header = Vec::new();
        header.extend_from_slice(&PACK_MAGIC);
        header.extend_from_slice(&99u32.to_le_bytes());
        header.extend_from_slice(&31102u32.to_le_bytes());

        match validate_pack_header(&header) {
            Err(EngineError::UnsupportedPackVersion { found, expected }) => {
                assert_eq!(found, 99);
                assert_eq!(expected, PACK_FORMAT_VERSION);
            }
            other => panic!("expected UnsupportedPackVersion, got {:?}", other),
        }
    }

    #[test]
    fn truncated_header() {
        assert!(validate_pack_header(&[0u8; 4]).is_err());
    }
}
