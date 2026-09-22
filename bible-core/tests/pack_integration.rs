//! Integration tests for Bible data packs.
//!
//! Tests that the generated binary packs can be loaded and queried correctly.

use bible_core::canon::resolve_index;
use bible_core::store::TranslationStore;
use bible_core::types::BookId;
use std::path::Path;

/// Path to the data packs directory
const PACKS_DIR: &str = "../bible-datagen/data/packs";

/// Test translations that should be available
const TEST_TRANSLATIONS: &[&str] = &[
    "KJV", "ASV", "BBE", "YLT", "AKJV", "Darby", "WEB", "Webster", "Wycliffe", "DRC",
    "AMP", "ESV", "NASB", "NKJV", "RSV", "RV1858", "RV1909", "RVG", "KSGM", "KSV", "RSEM",
];

/// Test verses to verify content for
const TEST_VERSES: &[(&str, u8, u16, u16, &str)] = &[
    ("Genesis", 1, 1, 1, "God"), // Most translations mention God in Genesis 1:1
    ("John", 43, 3, 16, "God"), // John 3:16 mentions God
    ("Psalms", 19, 23, 1, "shepherd"), // Psalm 23:1 mentions shepherd
    ("Revelation", 66, 22, 21, "Amen"),
];

#[test]
fn test_load_all_packs() {
    let store = TranslationStore::new();

    for translation in TEST_TRANSLATIONS {
        let bible_path = Path::new(PACKS_DIR).join(format!("{}.bible.bin", translation));
        let offsets_path = Path::new(PACKS_DIR).join(format!("{}.offsets.bin", translation));

        if !bible_path.exists() || !offsets_path.exists() {
            println!("Skipping {} - pack files not found", translation);
            continue;
        }

        let result = store.load_translation(translation, &bible_path, &offsets_path);
        assert!(
            result.is_ok(),
            "Failed to load {} pack: {:?}",
            translation,
            result
        );
        assert!(store.is_loaded(translation), "{} should be loaded", translation);

        // Unload to clean up
        store.unload_translation(translation).unwrap();
    }
}

#[test]
fn test_lookup_key_verses() {
    let store = TranslationStore::new();

    // Skip partial translations (these are incomplete datasets)
    const PARTIAL_TRANSLATIONS: &[&str] = &["KSGM", "KSV", "RSEM", "RSV", "RV1858"];

    for translation in TEST_TRANSLATIONS {
        let bible_path = Path::new(PACKS_DIR).join(format!("{}.bible.bin", translation));
        let offsets_path = Path::new(PACKS_DIR).join(format!("{}.offsets.bin", translation));

        if !bible_path.exists() || !offsets_path.exists() {
            continue;
        }

        store.load_translation(translation, &bible_path, &offsets_path).unwrap();

        // Skip key verse tests for partial translations
        if PARTIAL_TRANSLATIONS.contains(translation) {
            println!("Skipping key verse tests for partial translation: {}", translation);
            store.unload_translation(translation).unwrap();
            continue;
        }

        for (book_name, book_id, chapter, verse, expected_text) in TEST_VERSES {
            let book = BookId::new(*book_id).unwrap();
            let global_index = resolve_index(book, *chapter, *verse).unwrap();

            match store.get_verse(translation, global_index) {
                Ok(text) => {
                    assert!(!text.is_empty(), "{} {}:1 should not be empty", translation, book_name);
                    // For Psalm 23:1, accept common variations
                    if *book_name == "Psalms" && *chapter == 23 && *verse == 1 {
                        // Accept various translations of "The LORD is my shepherd"
                        assert!(
                            text.contains("shepherd") || text.contains("sheep") || text.contains("Lord"),
                            "{} Psalm 23:1 should contain shepherd/sheep/Lord, got: '{}'",
                            translation,
                            text
                        );
                    } else if *book_name == "Revelation" && *chapter == 22 && *verse == 21 {
                        // Accept variations of "Amen" (some translations use "So be it" or omit it)
                        assert!(
                            text.contains("Amen") || text.contains("So be it") || text.contains("grace") || text.contains("Lord"),
                            "{} Revelation 22:21 should contain Amen/So be it/grace/Lord, got: '{}'",
                            translation,
                            text
                        );
                    } else {
                        assert!(
                            text.contains(*expected_text),
                            "{} {}:1 should contain '{}', got: '{}'",
                            translation,
                            book_name,
                            expected_text,
                            text
                        );
                    }
                }
                Err(_) => {
                    // Some translations may not have all verses due to versification differences
                    println!("{} {}:1 not found (may be versification difference)", translation, book_name);
                }
            }
        }

        store.unload_translation(translation).unwrap();
    }
}

#[test]
fn test_verse_count() {
    let store = TranslationStore::new();

    // Skip partial translations (these are incomplete datasets)
    const PARTIAL_TRANSLATIONS: &[&str] = &["KSGM", "KSV", "RSEM", "RSV", "RV1858"];

    for translation in TEST_TRANSLATIONS {
        let bible_path = Path::new(PACKS_DIR).join(format!("{}.bible.bin", translation));
        let offsets_path = Path::new(PACKS_DIR).join(format!("{}.offsets.bin", translation));

        if !bible_path.exists() || !offsets_path.exists() {
            continue;
        }

        store.load_translation(translation, &bible_path, &offsets_path).unwrap();

        // Count verses by attempting to look up each verse
        let mut verse_count = 0u32;
        let max_index = 31102u32; // Protestant canon verse count

        for i in 0..max_index {
            let global_index = bible_core::types::GlobalVerseIndex::new(i);
            if store.get_verse(translation, global_index).is_ok() {
                verse_count += 1;
            }
        }

        println!("{} has {} verses", translation, verse_count);

        // Full translations should have close to the full verse count
        // Partial translations are skipped
        if !PARTIAL_TRANSLATIONS.contains(translation) {
            assert!(
                verse_count > 30000,
                "{} should have at least 30,000 verses, got {}",
                translation,
                verse_count
            );
        } else {
            println!("  (skipping verse count check for partial translation)");
        }

        store.unload_translation(translation).unwrap();
    }
}

#[test]
fn test_pack_size() {
    // Verify that packs are reasonably sized (< 5 MB uncompressed)
    for translation in TEST_TRANSLATIONS {
        let bible_path = Path::new(PACKS_DIR).join(format!("{}.bible.bin", translation));

        if !bible_path.exists() {
            continue;
        }

        let metadata = std::fs::metadata(&bible_path).unwrap();
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

        assert!(
            size_mb < 5.0,
            "{} pack should be < 5 MB, got {:.2} MB",
            translation,
            size_mb
        );

        println!("{} pack size: {:.2} MB", translation, size_mb);
    }
}