//! Core type definitions for the Bible engine.
//!
//! All types here are value types — cheap to copy, compare, and hash.
//! They carry no allocations on the hot path.

use std::fmt;

/// A book identifier in the canonical 66-book Protestant scheme.
/// Valid range: 1–66 inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BookId(pub u8);

impl BookId {
    /// The minimum valid book ID (Genesis).
    pub const MIN: u8 = 1;
    /// The maximum valid book ID (Revelation).
    pub const MAX: u8 = 66;

    /// Create a new BookId, returning None if out of range.
    pub fn new(id: u8) -> Option<Self> {
        if (Self::MIN..=Self::MAX).contains(&id) {
            Some(BookId(id))
        } else {
            None
        }
    }

    /// Get the raw u8 value.
    pub fn as_u8(self) -> u8 {
        self.0
    }
}

impl fmt::Display for BookId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A global verse index in the canonical versification (0-based).
/// Range: 0..31,101 (31,102 total verses).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GlobalVerseIndex(pub u32);

impl GlobalVerseIndex {
    /// Create a new GlobalVerseIndex.
    pub fn new(index: u32) -> Self {
        GlobalVerseIndex(index)
    }

    /// Get the raw u32 value.
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Display for GlobalVerseIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A resolved verse reference: book + chapter + verse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct VerseRef {
    pub book: BookId,
    pub chapter: u16,
    pub verse: u16,
}

impl VerseRef {
    /// Create a new VerseRef.
    pub fn new(book: BookId, chapter: u16, verse: u16) -> Self {
        VerseRef {
            book,
            chapter,
            verse,
        }
    }
}

impl fmt::Display for VerseRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Uses the canon module to get the book name if available,
        // but falls back to the numeric ID to avoid circular dependency issues.
        write!(f, "Book{}:{}:{}", self.book, self.chapter, self.verse)
    }
}

/// A translation code identifying a Bible translation (e.g., "KJV", "NIV", "ESV").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TranslationCode(pub String);

impl TranslationCode {
    /// Create a new TranslationCode.
    pub fn new(code: impl Into<String>) -> Self {
        TranslationCode(code.into())
    }

    /// Get the code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TranslationCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Current data pack format version. Must match the version in the pack header.
pub const PACK_FORMAT_VERSION: u32 = 1;

/// Magic bytes identifying a valid .bible.bin file.
pub const PACK_MAGIC: [u8; 4] = *b"HVBB";

/// Size of the .bible.bin header: magic(4) + version(4) + verse_count(4) + blake3(32) = 44.
pub const PACK_HEADER_SIZE: usize = 4 + 4 + 4 + 32;

/// Magic bytes identifying a valid .offsets.bin file.
pub const OFFSETS_MAGIC: [u8; 4] = *b"HVBO";

/// Size of the .offsets.bin header: magic(4) + version(4) + entry_count(4) = 12.
pub const OFFSETS_HEADER_SIZE: usize = 4 + 4 + 4;

/// Sentinel value for missing offset entries.
pub const OFFSET_MISSING: u32 = u32::MAX;

/// A single verse record stored in the data pack.
///
/// This is the unit of serialization — each verse is independently
/// archived so individual records can be accessed via offset table
/// without deserializing the entire file.
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize)]
#[rkyv(derive(Debug))]
pub struct VerseRecord {
    /// The verse text content.
    pub text: String,
}
