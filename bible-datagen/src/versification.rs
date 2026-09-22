//! Versification module — runtime access to the canonical verse structure.
//!
//! This mirrors the data in `data/versification.toml` but as Rust constants
//! for use within bible-datagen itself. The same TOML file is also consumed
//! by `bible-core/build.rs` to generate compile-time PHF maps.

use serde::Deserialize;
use std::path::Path;

/// A single book entry from versification.toml.
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // id/name/abbrev used by Phase 2 pack writer; chapters used now for stats
pub struct BookEntry {
    pub id: u8,
    pub name: String,
    pub abbrev: String,
    pub chapters: Vec<u16>,
}

/// Top-level structure of versification.toml.
#[derive(Debug, Deserialize)]
pub struct VersificationData {
    pub book: Vec<BookEntry>,
}

/// Summary statistics for the canonical versification.
pub struct VersificationStats {
    pub book_count: usize,
    pub chapter_count: usize,
    pub verse_count: usize,
}

/// Load versification data from the TOML file.
pub fn load_from_file(path: &Path) -> VersificationData {
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "failed to read versification file at {}: {}",
            path.display(),
            e
        );
    });
    toml::from_str(&content).unwrap_or_else(|e| {
        panic!("failed to parse versification TOML: {}", e);
    })
}

/// Compute statistics from the embedded versification data.
pub fn compute_stats() -> VersificationStats {
    let data_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/versification.toml");
    let data = load_from_file(&data_path);

    let book_count = data.book.len();
    let chapter_count: usize = data.book.iter().map(|b| b.chapters.len()).sum();
    let verse_count: usize = data
        .book
        .iter()
        .map(|b| b.chapters.iter().map(|&v| v as usize).sum::<usize>())
        .sum();

    VersificationStats {
        book_count,
        chapter_count,
        verse_count,
    }
}
