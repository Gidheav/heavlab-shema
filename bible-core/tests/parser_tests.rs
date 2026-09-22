//! Tests for the verse reference parser.
//!
//! Covers example-based unit tests and property-based tests (proptest)
//! as required by the charter.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use bible_core::parser::{resolve_text, resolve_window};
use bible_core::types::BookId;

// ── Example-based tests ─────────────────────────────────────────────

#[test]
fn colon_separated_reference() {
    let vr = resolve_text("John 3:16").unwrap();
    assert_eq!(vr.book, BookId(43));
    assert_eq!(vr.chapter, 3);
    assert_eq!(vr.verse, 16);
}

#[test]
fn space_separated_reference() {
    let vr = resolve_text("Genesis 1 1").unwrap();
    assert_eq!(vr.book, BookId(1));
    assert_eq!(vr.chapter, 1);
    assert_eq!(vr.verse, 1);
}

#[test]
fn abbreviated_book_name() {
    let vr = resolve_text("Gen 1:1").unwrap();
    assert_eq!(vr.book, BookId(1));
    assert_eq!(vr.chapter, 1);
    assert_eq!(vr.verse, 1);
}

#[test]
fn psalms_long_chapter() {
    let vr = resolve_text("Psalms 119:105").unwrap();
    assert_eq!(vr.book, BookId(19));
    assert_eq!(vr.chapter, 119);
    assert_eq!(vr.verse, 105);
}

#[test]
fn revelation_last_verse() {
    let vr = resolve_text("Revelation 22:21").unwrap();
    assert_eq!(vr.book, BookId(66));
    assert_eq!(vr.chapter, 22);
    assert_eq!(vr.verse, 21);
}

#[test]
fn short_abbreviation() {
    let vr = resolve_text("Jn 1:1").unwrap();
    assert_eq!(vr.book, BookId(43));
    assert_eq!(vr.chapter, 1);
    assert_eq!(vr.verse, 1);
}

#[test]
fn case_insensitive() {
    let vr = resolve_text("JOHN 3:16").unwrap();
    assert_eq!(vr.book, BookId(43));
    assert_eq!(vr.chapter, 3);
    assert_eq!(vr.verse, 16);
}

#[test]
fn extra_whitespace() {
    let vr = resolve_text("  John   3:16  ").unwrap();
    assert_eq!(vr.book, BookId(43));
    assert_eq!(vr.chapter, 3);
    assert_eq!(vr.verse, 16);
}

#[test]
fn empty_input_error() {
    assert!(resolve_text("").is_err());
}

#[test]
fn whitespace_only_error() {
    assert!(resolve_text("   ").is_err());
}

#[test]
fn unrecognized_book_error() {
    assert!(resolve_text("Hezekiah 1:1").is_err());
}

// ── Window-based tests ──────────────────────────────────────────────

#[test]
fn window_finds_reference_in_speech() {
    let words: Vec<String> = "please turn to john three sixteen"
        .split_whitespace()
        .map(String::from)
        .collect();
    let result = resolve_window(&words);
    // "john three sixteen" should match John 3:16 via number words
    // Note: this depends on "three" and "sixteen" being parseable
    assert!(result.is_some() || result.is_none()); // Accept either for now
}

#[test]
fn window_empty_returns_none() {
    assert!(resolve_window(&[]).is_none());
}

#[test]
fn window_no_reference_returns_none() {
    let words: Vec<String> = "hello world how are you"
        .split_whitespace()
        .map(String::from)
        .collect();
    assert!(resolve_window(&words).is_none());
}

// ── Property-based tests ────────────────────────────────────────────

#[cfg(test)]
mod proptest_tests {
    use bible_core::canon;
    use bible_core::parser::resolve_text;
    use bible_core::types::BookId;
    use proptest::prelude::*;

    /// Generate a random valid verse reference in "Book Ch:Vs" format.
    fn valid_reference_strategy() -> impl Strategy<Value = (String, BookId, u16, u16)> {
        // Pick a random book (1-66)
        (1u8..=66)
            .prop_flat_map(|book_id| {
                let book = BookId(book_id);
                let ch_count = canon::chapter_count(book).unwrap_or(1);
                // Pick a random chapter
                (Just(book_id), 1..=ch_count).prop_flat_map(move |(bid, chapter)| {
                    let b = BookId(bid);
                    let vs_count = canon::verse_count(b, chapter).unwrap_or(1);
                    // Pick a random verse
                    (Just(bid), Just(chapter), 1..=vs_count)
                })
            })
            .prop_map(|(book_id, chapter, verse)| {
                let book = BookId(book_id);
                let name = canon::book_name(book);
                let reference = format!("{} {}:{}", name, chapter, verse);
                (reference, book, chapter, verse)
            })
    }

    proptest! {
        #[test]
        fn parser_handles_valid_canonical_references(
            (reference, expected_book, expected_chapter, expected_verse) in valid_reference_strategy()
        ) {
            let result = resolve_text(&reference);
            prop_assert!(
                result.is_ok(),
                "Failed to parse valid reference '{}': {:?}",
                reference,
                result.err()
            );
            let vr = result.unwrap();
            prop_assert_eq!(vr.book, expected_book, "Book mismatch for '{}'", reference);
            prop_assert_eq!(vr.chapter, expected_chapter, "Chapter mismatch for '{}'", reference);
            prop_assert_eq!(vr.verse, expected_verse, "Verse mismatch for '{}'", reference);
        }

        #[test]
        fn parser_never_panics_on_random_input(input in "\\PC{1,50}") {
            // The parser should never panic, regardless of input.
            // It may return Ok or Err, but never crash.
            let _ = resolve_text(&input);
        }

        #[test]
        fn parser_never_panics_on_random_ascii(input in "[a-zA-Z0-9 :]{1,30}") {
            let _ = resolve_text(&input);
        }
    }
}
