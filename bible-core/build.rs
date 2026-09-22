// build.rs — exempt from the library's panic-free constraints.
// Build scripts use panic! to abort the build with a clear error message,
// which is the correct and idiomatic pattern for Rust build scripts.
#![allow(clippy::unwrap_used)]
//! build.rs for bible-core
//!
//! Reads the canonical versification data and English alias table from
//! bible-datagen/data/ at compile time and generates:
//!
//! 1. `canonical_index.rs` — a `phf::Map` mapping `"BookId:Chapter:Verse"`
//!    string keys to `GlobalVerseIndex` (u32), plus a const reverse-lookup array,
//!    book name array, chapter count array, and verse count tables.
//!
//! 2. `aliases_en.rs` — a `phf::Map` mapping lowercase alias strings to BookId (u8).
//!
//! Both are written to `$OUT_DIR` and pulled in via `include!` in the
//! corresponding source modules.

use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

// ── TOML structures ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct VersificationFile {
    book: Vec<BookEntry>,
}

#[derive(Debug, Deserialize)]
struct BookEntry {
    id: u8,
    name: String,
    #[allow(dead_code)]
    abbrev: String,
    chapters: Vec<u16>,
}

#[derive(Debug, Deserialize)]
struct AliasFile {
    alias: Vec<AliasEntry>,
}

#[derive(Debug, Deserialize)]
struct AliasEntry {
    book_id: u8,
    aliases: Vec<String>,
}

// ── Main ────────────────────────────────────────────────────────────

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let datagen_data = Path::new(&manifest_dir)
        .join("..")
        .join("bible-datagen")
        .join("data");

    // Tell Cargo to rerun if the data files change.
    let versification_path = datagen_data.join("versification.toml");
    let aliases_path = datagen_data.join("en_aliases.toml");
    println!("cargo:rerun-if-changed={}", versification_path.display());
    println!("cargo:rerun-if-changed={}", aliases_path.display());

    generate_canonical_index(&versification_path, &out_dir);
    generate_aliases(&aliases_path, &out_dir);
}

// ── Canonical Index Generation ──────────────────────────────────────

fn generate_canonical_index(versification_path: &Path, out_dir: &str) {
    let content = fs::read_to_string(versification_path).unwrap_or_else(|e| {
        panic!(
            "bible-core build.rs: failed to read {}: {}",
            versification_path.display(),
            e
        );
    });
    let data: VersificationFile = toml::from_str(&content).unwrap_or_else(|e| {
        panic!(
            "bible-core build.rs: failed to parse versification TOML: {}",
            e
        );
    });

    let dest_path = Path::new(out_dir).join("canonical_index.rs");
    let file = fs::File::create(&dest_path).unwrap();
    let mut w = BufWriter::new(file);

    // ── 1. PHF Map: "book_id:chapter:verse" → GlobalVerseIndex ──

    let mut phf_map = phf_codegen::Map::new();
    let mut total_verses: u32 = 0;

    // First pass: count total verses for the reverse table
    for book in &data.book {
        for verse_count in &book.chapters {
            total_verses += *verse_count as u32;
        }
    }

    // Collect entries for both PHF and reverse table
    #[allow(dead_code)] // global_index stored for documentation clarity; ordering is implicit
    struct VerseEntry {
        book_id: u8,
        chapter: u16,
        verse: u16,
        global_index: u32,
    }
    let mut entries: Vec<VerseEntry> = Vec::with_capacity(total_verses as usize);

    let mut global_index: u32 = 0;
    for book in &data.book {
        for (ch_idx, &verse_count) in book.chapters.iter().enumerate() {
            let chapter = (ch_idx + 1) as u16;
            for verse in 1..=verse_count {
                let key = format!("{}:{}:{}", book.id, chapter, verse);
                phf_map.entry(key, &format!("{}u32", global_index));
                entries.push(VerseEntry {
                    book_id: book.id,
                    chapter,
                    verse,
                    global_index,
                });
                global_index += 1;
            }
        }
    }

    writeln!(
        w,
        "/// Total number of verses in the canonical versification."
    )
    .unwrap();
    writeln!(w, "pub const TOTAL_VERSE_COUNT: u32 = {};", total_verses).unwrap();
    writeln!(w).unwrap();

    writeln!(
        w,
        "/// Compile-time PHF map: \"book_id:chapter:verse\" → GlobalVerseIndex."
    )
    .unwrap();
    writeln!(
        w,
        "/// Keys are formatted as \"{{book_id}}:{{chapter}}:{{verse}}\" (e.g., \"43:3:16\" for John 3:16)."
    )
    .unwrap();
    write!(
        w,
        "pub static CANONICAL_INDEX: phf::Map<&'static str, u32> = "
    )
    .unwrap();
    write!(w, "{}", phf_map.build()).unwrap();
    writeln!(w, ";").unwrap();
    writeln!(w).unwrap();

    // ── 2. Reverse lookup: GlobalVerseIndex → (BookId, Chapter, Verse) ──

    writeln!(
        w,
        "/// Reverse lookup: index into this array with GlobalVerseIndex to get (BookId, Chapter, Verse)."
    )
    .unwrap();
    writeln!(
        w,
        "pub static REVERSE_INDEX: [(u8, u16, u16); {} ] = [",
        total_verses
    )
    .unwrap();

    for entry in &entries {
        writeln!(
            w,
            "    ({}, {}, {}),",
            entry.book_id, entry.chapter, entry.verse
        )
        .unwrap();
    }
    writeln!(w, "];").unwrap();
    writeln!(w).unwrap();

    // ── 3. Book names ──

    writeln!(
        w,
        "/// Canonical book names indexed by BookId (1-based; index 0 is empty)."
    )
    .unwrap();
    writeln!(
        w,
        "pub static BOOK_NAMES: [&str; {}] = [",
        data.book.len() + 1
    )
    .unwrap();
    writeln!(w, "    \"\",  // index 0 unused (BookId is 1-based)").unwrap();
    for book in &data.book {
        writeln!(w, "    \"{}\",", book.name).unwrap();
    }
    writeln!(w, "];").unwrap();
    writeln!(w).unwrap();

    // ── 4. Chapter counts per book ──

    writeln!(
        w,
        "/// Number of chapters per book, indexed by BookId (1-based; index 0 is 0)."
    )
    .unwrap();
    writeln!(
        w,
        "pub static CHAPTER_COUNTS: [u16; {}] = [",
        data.book.len() + 1
    )
    .unwrap();
    writeln!(w, "    0,  // index 0 unused").unwrap();
    for book in &data.book {
        writeln!(w, "    {},", book.chapters.len()).unwrap();
    }
    writeln!(w, "];").unwrap();
    writeln!(w).unwrap();

    // ── 5. Verse counts per chapter ──
    // Stored as a flat array of arrays: VERSE_COUNTS[book_id][chapter_index]
    // Since array sizes vary, we use a Vec-of-slices approach via const references.

    writeln!(
        w,
        "/// Verse counts per chapter for each book. Indexed by BookId (1-based)."
    )
    .unwrap();
    writeln!(
        w,
        "/// Each inner slice is 0-indexed by chapter (chapter 1 is index 0)."
    )
    .unwrap();

    // Generate individual const arrays per book
    for book in &data.book {
        writeln!(
            w,
            "const VERSES_BOOK_{}: [u16; {}] = {:?};",
            book.id,
            book.chapters.len(),
            book.chapters
        )
        .unwrap();
    }
    writeln!(w).unwrap();

    // Generate the lookup table
    writeln!(
        w,
        "pub static VERSE_COUNTS: [&[u16]; {}] = [",
        data.book.len() + 1
    )
    .unwrap();
    writeln!(w, "    &[],  // index 0 unused").unwrap();
    for book in &data.book {
        writeln!(w, "    &VERSES_BOOK_{},", book.id).unwrap();
    }
    writeln!(w, "];").unwrap();

    // ── 6. Book count constant ──

    writeln!(w).unwrap();
    writeln!(w, "/// Total number of books in the canon.").unwrap();
    writeln!(w, "pub const BOOK_COUNT: u8 = {};", data.book.len()).unwrap();
}

// ── Alias Generation ────────────────────────────────────────────────

fn generate_aliases(aliases_path: &Path, out_dir: &str) {
    let content = fs::read_to_string(aliases_path).unwrap_or_else(|e| {
        panic!(
            "bible-core build.rs: failed to read {}: {}",
            aliases_path.display(),
            e
        );
    });
    let data: AliasFile = toml::from_str(&content).unwrap_or_else(|e| {
        panic!("bible-core build.rs: failed to parse aliases TOML: {}", e);
    });

    let dest_path = Path::new(out_dir).join("aliases_en.rs");
    let file = fs::File::create(&dest_path).unwrap();
    let mut w = BufWriter::new(file);

    let mut phf_map = phf_codegen::Map::new();

    for entry in &data.alias {
        for alias in &entry.aliases {
            // Store all aliases lowercase for case-insensitive matching
            let key = alias.to_lowercase();
            phf_map.entry(key, &format!("{}u8", entry.book_id));
        }
    }

    writeln!(
        w,
        "/// Compile-time PHF map: lowercase English alias → BookId (u8)."
    )
    .unwrap();
    writeln!(
        w,
        "/// Includes canonical names, abbreviations, ordinal forms, and spoken variants."
    )
    .unwrap();
    write!(
        w,
        "pub static ENGLISH_ALIASES: phf::Map<&'static str, u8> = "
    )
    .unwrap();
    write!(w, "{}", phf_map.build()).unwrap();
    writeln!(w, ";").unwrap();
}
