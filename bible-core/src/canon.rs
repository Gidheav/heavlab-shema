//! Canonical index — compile-time-generated verse structure.
//!
//! This module provides the canonical mapping between (BookId, Chapter, Verse)
//! and GlobalVerseIndex, along with book names, chapter counts, and verse counts.
//! All data is baked into the binary at compile time via `phf_codegen` in build.rs.

use crate::error::{EngineError, ParseError};
use crate::types::{BookId, GlobalVerseIndex};

// ── Generated code from build.rs ────────────────────────────────────
// These files are created at compile time in $OUT_DIR.
include!(concat!(env!("OUT_DIR"), "/canonical_index.rs"));
include!(concat!(env!("OUT_DIR"), "/aliases_en.rs"));

/// Resolve a (BookId, Chapter, Verse) triple to a GlobalVerseIndex.
///
/// Returns `EngineError::InvalidBookId` if the book is out of range,
/// or `EngineError::VerseNotFound` if the chapter/verse doesn't exist.
pub fn resolve_index(
    book: BookId,
    chapter: u16,
    verse: u16,
) -> Result<GlobalVerseIndex, EngineError> {
    let id = book.as_u8();
    if !(BookId::MIN..=BookId::MAX).contains(&id) {
        return Err(EngineError::InvalidBookId(id));
    }

    // Format the key for PHF lookup
    let key = format!("{}:{}:{}", id, chapter, verse);

    match CANONICAL_INDEX.get(key.as_str()) {
        Some(&index) => Ok(GlobalVerseIndex::new(index)),
        None => Err(EngineError::VerseNotFound {
            book: book_name(book).to_string(),
            chapter,
            verse,
        }),
    }
}

/// Resolve a lowercase book alias to a BookId.
///
/// The input is converted to lowercase before lookup.
/// Returns `ParseError::UnrecognizedBook` if no match is found.
pub fn resolve_book_alias(alias: &str) -> Result<BookId, ParseError> {
    let lower = alias.to_lowercase();
    match ENGLISH_ALIASES.get(lower.as_str()) {
        Some(&id) => match BookId::new(id) {
            Some(book_id) => Ok(book_id),
            None => Err(ParseError::UnrecognizedBook(alias.to_string())),
        },
        None => Err(ParseError::UnrecognizedBook(alias.to_string())),
    }
}

/// Get the canonical English name of a book by its ID.
///
/// Returns "Unknown" for invalid IDs rather than panicking.
pub fn book_name(id: BookId) -> &'static str {
    let idx = id.as_u8() as usize;
    if idx < BOOK_NAMES.len() {
        BOOK_NAMES[idx]
    } else {
        "Unknown"
    }
}

/// Get the number of chapters in a book.
///
/// Returns `None` for invalid book IDs.
pub fn chapter_count(book: BookId) -> Option<u16> {
    let idx = book.as_u8() as usize;
    if idx < CHAPTER_COUNTS.len() {
        Some(CHAPTER_COUNTS[idx])
    } else {
        None
    }
}

/// Get the number of verses in a specific chapter of a book.
///
/// Returns `None` if the book or chapter is invalid.
pub fn verse_count(book: BookId, chapter: u16) -> Option<u16> {
    let book_idx = book.as_u8() as usize;
    if book_idx >= VERSE_COUNTS.len() {
        return None;
    }
    let chapters = VERSE_COUNTS[book_idx];
    let ch_idx = chapter.checked_sub(1)? as usize;
    chapters.get(ch_idx).copied()
}

/// Reverse-lookup: convert a GlobalVerseIndex back to (BookId, Chapter, Verse).
///
/// Returns `None` if the index is out of range.
pub fn reverse_lookup(index: GlobalVerseIndex) -> Option<(BookId, u16, u16)> {
    let idx = index.as_u32() as usize;
    if idx < REVERSE_INDEX.len() {
        let (book_id, chapter, verse) = REVERSE_INDEX[idx];
        Some((BookId(book_id), chapter, verse))
    } else {
        None
    }
}
