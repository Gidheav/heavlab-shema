//! Integration tests for the canonical index.
//!
//! Verifies that the compile-time PHF map correctly covers all 66 books,
//! boundary verses, and known verse counts.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use bible_core::canon;
use bible_core::types::BookId;

#[test]
fn total_verse_count_is_correct() {
    // The canonical data yields 31,103 verses.
    assert_eq!(canon::TOTAL_VERSE_COUNT, 31103);
}

#[test]
fn book_count_is_66() {
    assert_eq!(canon::BOOK_COUNT, 66);
}

#[test]
fn first_verse_resolves() {
    // Genesis 1:1 should be global index 0
    let result = canon::resolve_index(BookId(1), 1, 1);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_u32(), 0);
}

#[test]
fn last_verse_resolves() {
    // Revelation 22:21 should be the last verse (index 31,102)
    let result = canon::resolve_index(BookId(66), 22, 21);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_u32(), 31102);
}

#[test]
fn john_3_16_resolves() {
    let result = canon::resolve_index(BookId(43), 3, 16);
    assert!(result.is_ok());
}

#[test]
fn psalm_119_176_resolves() {
    // Psalm 119 is the longest chapter (176 verses)
    let result = canon::resolve_index(BookId(19), 119, 176);
    assert!(result.is_ok());
}

#[test]
fn invalid_book_fails() {
    let result = canon::resolve_index(BookId(0), 1, 1);
    assert!(result.is_err());

    let result = canon::resolve_index(BookId(67), 1, 1);
    assert!(result.is_err());
}

#[test]
fn invalid_chapter_fails() {
    // Genesis only has 50 chapters
    let result = canon::resolve_index(BookId(1), 51, 1);
    assert!(result.is_err());
}

#[test]
fn invalid_verse_fails() {
    // Genesis 1 only has 31 verses
    let result = canon::resolve_index(BookId(1), 1, 32);
    assert!(result.is_err());
}

#[test]
fn all_book_names_present() {
    for id in 1..=66u8 {
        let name = canon::book_name(BookId(id));
        assert!(!name.is_empty(), "Book {} has no name", id);
        assert_ne!(name, "Unknown", "Book {} returned 'Unknown'", id);
    }
}

#[test]
fn chapter_counts_reasonable() {
    // Spot checks for well-known chapter counts
    assert_eq!(canon::chapter_count(BookId(1)), Some(50)); // Genesis: 50
    assert_eq!(canon::chapter_count(BookId(19)), Some(150)); // Psalms: 150
    assert_eq!(canon::chapter_count(BookId(43)), Some(21)); // John: 21
    assert_eq!(canon::chapter_count(BookId(66)), Some(22)); // Revelation: 22
    assert_eq!(canon::chapter_count(BookId(31)), Some(1)); // Obadiah: 1
    assert_eq!(canon::chapter_count(BookId(57)), Some(1)); // Philemon: 1
}

#[test]
fn verse_counts_spot_check() {
    assert_eq!(canon::verse_count(BookId(1), 1), Some(31)); // Genesis 1: 31 verses
    assert_eq!(canon::verse_count(BookId(19), 119), Some(176)); // Psalm 119: 176 verses
    assert_eq!(canon::verse_count(BookId(19), 117), Some(2)); // Psalm 117: 2 verses (shortest chapter)
}

#[test]
fn reverse_lookup_round_trip() {
    // Forward then reverse for a few known verses
    let test_cases = [
        (BookId(1), 1u16, 1u16), // Genesis 1:1
        (BookId(43), 3, 16),     // John 3:16
        (BookId(66), 22, 21),    // Revelation 22:21
        (BookId(19), 23, 1),     // Psalm 23:1
    ];

    for (book, ch, vs) in test_cases {
        let idx = canon::resolve_index(book, ch, vs).unwrap();
        let (rb, rc, rv) = canon::reverse_lookup(idx).unwrap();
        assert_eq!(rb, book, "book mismatch for {:?}", idx);
        assert_eq!(rc, ch, "chapter mismatch for {:?}", idx);
        assert_eq!(rv, vs, "verse mismatch for {:?}", idx);
    }
}

#[test]
fn all_66_books_have_at_least_one_chapter() {
    for id in 1..=66u8 {
        let count = canon::chapter_count(BookId(id));
        assert!(
            count.is_some() && count.unwrap() > 0,
            "Book {} has no chapters",
            id
        );
    }
}

#[test]
fn alias_genesis() {
    let book = canon::resolve_book_alias("genesis");
    assert!(book.is_ok());
    assert_eq!(book.unwrap(), BookId(1));

    assert_eq!(canon::resolve_book_alias("gen").unwrap(), BookId(1));
    assert_eq!(canon::resolve_book_alias("Gen").unwrap(), BookId(1));
}

#[test]
fn alias_revelation() {
    assert_eq!(canon::resolve_book_alias("revelation").unwrap(), BookId(66));
    assert_eq!(canon::resolve_book_alias("rev").unwrap(), BookId(66));
    assert_eq!(
        canon::resolve_book_alias("Revelations").unwrap(),
        BookId(66)
    );
}

#[test]
fn alias_numbered_books() {
    assert_eq!(canon::resolve_book_alias("1 samuel").unwrap(), BookId(9));
    assert_eq!(canon::resolve_book_alias("2 samuel").unwrap(), BookId(10));
    assert_eq!(
        canon::resolve_book_alias("first samuel").unwrap(),
        BookId(9)
    );
    assert_eq!(
        canon::resolve_book_alias("1 corinthians").unwrap(),
        BookId(46)
    );
    assert_eq!(
        canon::resolve_book_alias("first corinthians").unwrap(),
        BookId(46)
    );
}

#[test]
fn alias_unknown_fails() {
    assert!(canon::resolve_book_alias("notabook").is_err());
    assert!(canon::resolve_book_alias("").is_err());
}
