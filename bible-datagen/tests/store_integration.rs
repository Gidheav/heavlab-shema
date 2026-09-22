//! Integration test: full round-trip through the data pipeline.
//!
//! 1. Generate a small data pack via PackWriter
//! 2. Load it into TranslationStore (mmap + BLAKE3 validation)
//! 3. Retrieve verses by GlobalVerseIndex and verify text
#![allow(clippy::unwrap_used, clippy::expect_used)]

use bible_core::canon::resolve_index;
use bible_core::store::TranslationStore;
use bible_core::types::{
    BookId, GlobalVerseIndex, VerseRecord, OFFSETS_MAGIC, OFFSET_MISSING, PACK_FORMAT_VERSION, PACK_MAGIC,
};
use std::fs::{self, File};
use std::io::{BufWriter, Write};

/// Build a tiny data pack directly (bypassing the TSV parsing) and load it.
#[test]
fn round_trip_generate_load_get_verse() {
    let dir = std::env::temp_dir().join("sermon_integration_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let verses = [
        (
            0u32,
            "In the beginning God created the heavens and the earth.",
        ),
        (
            1,
            "And the earth was without form, and void; and darkness was upon the face of the deep.",
        ),
        (2, "And God said, Let there be light: and there was light."),
    ];

    // ── Build .bible.bin ────────────────────────────────────────────
    let mut payload: Vec<u8> = Vec::new();
    let entry_count = 3u32;
    let mut offset_table: Vec<(u32, u32)> = vec![(OFFSET_MISSING, 0); entry_count as usize];

    for &(idx, text) in &verses {
        // Align to 8 bytes.
        let padding = (8 - (payload.len() % 8)) % 8;
        payload.resize(payload.len() + padding, 0u8);

        let record = VerseRecord {
            text: text.to_string(),
        };
        let archived = rkyv::to_bytes::<rkyv::rancor::Error>(&record).unwrap();

        let offset = payload.len() as u32;
        let length = archived.len() as u32;
        offset_table[idx as usize] = (offset, length);

        payload.extend_from_slice(&archived);
    }

    let checksum = blake3::hash(&payload);

    let bible_path = dir.join("TEST.bible.bin");
    {
        let mut w = BufWriter::new(File::create(&bible_path).unwrap());
        w.write_all(&PACK_MAGIC).unwrap();
        w.write_all(&PACK_FORMAT_VERSION.to_le_bytes()).unwrap();
        w.write_all(&(verses.len() as u32).to_le_bytes()).unwrap();
        w.write_all(checksum.as_bytes()).unwrap();
        w.write_all(&payload).unwrap();
        w.flush().unwrap();
    }

    let offsets_path = dir.join("TEST.offsets.bin");
    {
        let mut w = BufWriter::new(File::create(&offsets_path).unwrap());
        w.write_all(&OFFSETS_MAGIC).unwrap();
        w.write_all(&PACK_FORMAT_VERSION.to_le_bytes()).unwrap();
        w.write_all(&entry_count.to_le_bytes()).unwrap();
        for &(offset, length) in &offset_table {
            w.write_all(&offset.to_le_bytes()).unwrap();
            w.write_all(&length.to_le_bytes()).unwrap();
        }
        w.flush().unwrap();
    }

    // ── Load into TranslationStore ──────────────────────────────────
    let store = TranslationStore::new();
    store
        .load_translation("TEST", &bible_path, &offsets_path)
        .expect("load_translation should succeed");

    assert!(store.is_loaded("TEST"));
    assert!(!store.is_loaded("MISSING"));
    assert_eq!(store.loaded_translations(), vec!["TEST".to_string()]);

    // ── Retrieve verses ─────────────────────────────────────────────
    for &(idx, expected_text) in &verses {
        let actual = store
            .get_verse("TEST", GlobalVerseIndex::new(idx))
            .unwrap_or_else(|e| panic!("get_verse({}) failed: {}", idx, e));
        assert_eq!(actual, expected_text, "mismatch at index {}", idx);
    }

    // ── Missing translation errors ──────────────────────────────────
    assert!(store
        .get_verse("MISSING", GlobalVerseIndex::new(0))
        .is_err());

    // ── Unload ──────────────────────────────────────────────────────
    store.unload_translation("TEST").unwrap();
    assert!(!store.is_loaded("TEST"));
    assert!(store.get_verse("TEST", GlobalVerseIndex::new(0)).is_err());

    let _ = fs::remove_dir_all(&dir);
}

/// Corrupt data should be detected: flip a byte in the payload.
#[test]
fn corrupt_pack_detected() {
    let dir = std::env::temp_dir().join("sermon_corrupt_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let record = VerseRecord {
        text: "test".to_string(),
    };
    let archived = rkyv::to_bytes::<rkyv::rancor::Error>(&record).unwrap();
    let mut payload = archived.to_vec();

    // Compute checksum of correct payload.
    let checksum = blake3::hash(&payload);

    // Now corrupt the payload.
    if let Some(b) = payload.last_mut() {
        *b ^= 0xFF;
    }

    let bible_path = dir.join("CORRUPT.bible.bin");
    {
        let mut w = BufWriter::new(File::create(&bible_path).unwrap());
        w.write_all(&PACK_MAGIC).unwrap();
        w.write_all(&PACK_FORMAT_VERSION.to_le_bytes()).unwrap();
        w.write_all(&1u32.to_le_bytes()).unwrap();
        w.write_all(checksum.as_bytes()).unwrap(); // stale checksum
        w.write_all(&payload).unwrap(); // corrupted payload
        w.flush().unwrap();
    }

    let offsets_path = dir.join("CORRUPT.offsets.bin");
    {
        let mut w = BufWriter::new(File::create(&offsets_path).unwrap());
        w.write_all(&OFFSETS_MAGIC).unwrap();
        w.write_all(&PACK_FORMAT_VERSION.to_le_bytes()).unwrap();
        w.write_all(&1u32.to_le_bytes()).unwrap();
        w.write_all(&0u32.to_le_bytes()).unwrap(); // offset
        w.write_all(&(payload.len() as u32).to_le_bytes()).unwrap(); // length
        w.flush().unwrap();
    }

    let store = TranslationStore::new();
    let result = store.load_translation("CORRUPT", &bible_path, &offsets_path);
    assert!(result.is_err(), "should detect corrupt pack");

    let _ = fs::remove_dir_all(&dir);
}

/// Integration test for the KJV data pack.
///
/// This test loads the generated KJV pack and verifies key verses:
/// - Genesis 1:1 (index 0) should start with "In the beginning"
/// - John 3:16 should contain "For God so loved the world"
/// - Revelation 22:21 (index 31101) should contain "grace"
#[test]
fn kjv_pack_integration() {
    let kjv_bible_path = std::path::Path::new("data/packs/KJV.bible.bin");
    let kjv_offsets_path = std::path::Path::new("data/packs/KJV.offsets.bin");
    
    // Skip test if pack files don't exist (e.g., during CI without generation)
    if !kjv_bible_path.exists() || !kjv_offsets_path.exists() {
        println!("Skipping KJV pack integration test - pack files not found");
        return;
    }
    
    let store = TranslationStore::new();
    store
        .load_translation("KJV", kjv_bible_path, kjv_offsets_path)
        .expect("load_translation should succeed for KJV pack");
    
    assert!(store.is_loaded("KJV"));
    
    // Test Genesis 1:1 (should be index 0)
    let genesis_1_1 = store
        .get_verse("KJV", GlobalVerseIndex::new(0))
        .expect("get_verse should succeed for Genesis 1:1");
    assert!(
        genesis_1_1.starts_with("In the beginning"),
        "Genesis 1:1 should start with 'In the beginning', got: {}",
        genesis_1_1
    );
    
    // Test John 3:16
    // Book ID for John is 43 (from versification.toml)
    let john_3_16_idx = resolve_index(BookId::new(43).unwrap(), 3, 16)
        .expect("John 3:16 should resolve to a valid index");
    let john_3_16 = store
        .get_verse("KJV", john_3_16_idx)
        .expect("get_verse should succeed for John 3:16");
    assert!(
        john_3_16.contains("For God so loved the world"),
        "John 3:16 should contain 'For God so loved the world', got: {}",
        john_3_16
    );
    
    // Test Revelation 22:21 (should be index 31102 - the last verse, 0-based)
    let revelation_22_21 = store
        .get_verse("KJV", GlobalVerseIndex::new(31102))
        .expect("get_verse should succeed for Revelation 22:21");
    assert!(
        revelation_22_21.contains("grace"),
        "Revelation 22:21 should contain 'grace', got: {}",
        revelation_22_21
    );
    
    store.unload_translation("KJV").unwrap();
    assert!(!store.is_loaded("KJV"));
}
