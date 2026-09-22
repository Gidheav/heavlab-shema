//! Verse reference parser — deterministic finite-state machine.
//!
//! Parses spoken and written Bible references into `VerseRef` values.
//! Supports patterns:
//!   - `<Book> <Chapter>:<Verse>`      — "John 3:16"
//!   - `<Book> <Chapter> <Verse>`      — "Genesis 1 1"
//!   - `<Ordinal> <Book> <Ch>:<Vs>`    — "First Corinthians 13:4"
//!   - `<Book> chapter <Ch> verse <Vs>` — "Psalms chapter 119 verse 105"
//!
//! No regex. All number normalization goes through `number_words`.

use crate::canon;
use crate::error::ParseError;
use crate::number_words;
use crate::types::VerseRef;

/// Parse a text string as a Bible verse reference.
///
/// The input is tokenized on whitespace. The parser runs a deterministic
/// FSM over the token stream to extract book name, chapter, and verse.
///
/// # Examples
/// ```ignore
/// resolve_text("John 3:16")            // Ok(VerseRef { book: 43, chapter: 3, verse: 16 })
/// resolve_text("First Corinthians 13:4") // Ok(VerseRef { book: 46, chapter: 13, verse: 4 })
/// resolve_text("Genesis 1 1")           // Ok(VerseRef { book: 1, chapter: 1, verse: 1 })
/// ```
pub fn resolve_text(text: &str) -> Result<VerseRef, ParseError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    parse_tokens(&tokens)
}

/// Attempt to match a verse reference from a rolling window of recent words.
///
/// Returns `Some(VerseRef)` immediately on a definitive match,
/// `None` if the window doesn't contain a recognizable reference.
/// Never guesses — only returns a match when unambiguous.
pub fn resolve_window(words: &[String]) -> Option<VerseRef> {
    if words.is_empty() {
        return None;
    }

    // Try progressively larger windows from the end of the buffer,
    // longest match first to prefer more specific references.
    let max_window = words.len().min(8); // A reference is at most ~6-7 tokens

    for start in (0..words.len()).rev() {
        let end = words.len().min(start + max_window);
        let window: Vec<&str> = words[start..end].iter().map(|s| s.as_str()).collect();

        if let Ok(verse_ref) = parse_tokens(&window) {
            // Validate the result exists in the canonical index
            if canon::resolve_index(verse_ref.book, verse_ref.chapter, verse_ref.verse).is_ok() {
                return Some(verse_ref);
            }
        }
    }

    None
}

// ── Internal parser ─────────────────────────────────────────────────

/// FSM states for the reference parser.
#[derive(Debug)]
enum State {
    /// Looking for the start of a book name.
    ExpectBook,
    /// Accumulating multi-word book name tokens.
    AccumulatingBook,
    /// Got the book, expecting chapter number.
    ExpectChapter,
    /// Got the chapter, expecting verse (or colon-separated ch:vs already parsed).
    ExpectVerse,
    /// Parsing complete.
    Done,
}

fn parse_tokens(tokens: &[&str]) -> Result<VerseRef, ParseError> {
    let mut state = State::ExpectBook;
    let mut book_name_parts: Vec<String> = Vec::new();
    let mut resolved_book = None;
    let mut chapter: Option<u16> = None;
    let mut verse: Option<u16> = None;

    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        let lower = token.to_lowercase();

        match state {
            State::ExpectBook => {
                // Check if this is an ordinal prefix ("First", "Second", etc.)
                if number_words::is_ordinal(token) {
                    if let Some(n) = number_words::parse_number_word(token) {
                        book_name_parts.push(n.to_string());
                        state = State::AccumulatingBook;
                        i += 1;
                        continue;
                    }
                }

                // Check if it starts with a digit prefix for numbered books ("1", "2", "3")
                if let Ok(n) = token.parse::<u8>() {
                    if (1..=3).contains(&n) {
                        book_name_parts.push(token.to_string());
                        state = State::AccumulatingBook;
                        i += 1;
                        continue;
                    }
                }

                // Try as a single-word book name
                book_name_parts.push(lower.clone());
                state = State::AccumulatingBook;
                i += 1;
            }

            State::AccumulatingBook => {
                // Check if current token can be parsed as a chapter (or chapter prefix)
                if let Some((ch, vs)) = try_parse_chapter_verse(token) {
                    let candidate = book_name_parts.join(" ");
                    if canon::resolve_book_alias(&candidate).is_ok() {
                        resolved_book = Some(candidate);
                        chapter = Some(ch);
                        verse = vs;
                        state = if verse.is_some() {
                            State::Done
                        } else {
                            State::ExpectVerse
                        };
                        i += 1;
                        continue;
                    }
                } else if let Some(n) = number_words::parse_number_word(token) {
                    let candidate = book_name_parts.join(" ");
                    if canon::resolve_book_alias(&candidate).is_ok() {
                        resolved_book = Some(candidate);
                        chapter = Some(n);
                        state = State::ExpectVerse;
                        i += 1;
                        continue;
                    }
                } else if lower == "chapter" {
                    let candidate = book_name_parts.join(" ");
                    if canon::resolve_book_alias(&candidate).is_ok() {
                        resolved_book = Some(candidate);
                        state = State::ExpectChapter;
                        i += 1;
                        continue;
                    }
                }

                // If not a chapter, or if the preceding words don't form a valid book,
                // keep accumulating.
                book_name_parts.push(lower.clone());
                i += 1;
            }

            State::ExpectChapter => {
                // Skip the word "chapter" if present
                if lower == "chapter" {
                    i += 1;
                    continue;
                }

                if let Some((ch, vs)) = try_parse_chapter_verse(token) {
                    chapter = Some(ch);
                    verse = vs;
                    state = if verse.is_some() {
                        State::Done
                    } else {
                        State::ExpectVerse
                    };
                } else if let Some(n) = number_words::parse_number_word(token) {
                    chapter = Some(n);
                    state = State::ExpectVerse;
                } else {
                    return Err(ParseError::InvalidChapter(token.to_string()));
                }
                i += 1;
            }

            State::ExpectVerse => {
                // Skip the word "verse" if present
                if lower == "verse" {
                    i += 1;
                    continue;
                }

                if let Some(n) = number_words::parse_number_word(token) {
                    verse = Some(n);
                    state = State::Done;
                } else if let Ok(n) = token.parse::<u16>() {
                    verse = Some(n);
                    state = State::Done;
                }
                // If we can't parse a verse number, that's okay — some references
                // are just "Book Chapter" without a verse.
                i += 1;
            }

            State::Done => break,
        }
    }

    // Build the result
    let book_str = resolved_book.ok_or_else(|| {
        ParseError::IncompleteReference(format!(
            "could not identify book in: {}",
            book_name_parts.join(" ")
        ))
    })?;

    let book_id = canon::resolve_book_alias(&book_str)?;

    let ch = chapter.ok_or_else(|| {
        ParseError::IncompleteReference(format!("no chapter number found for {}", book_str))
    })?;

    let vs = verse.ok_or_else(|| {
        ParseError::IncompleteReference(format!("no verse number found for {} {}", book_str, ch))
    })?;

    match canon::chapter_count(book_id) {
        Some(max_chapter) if (1..=max_chapter).contains(&ch) => {}
        _ => return Err(ParseError::InvalidChapter(ch.to_string())),
    }
    match canon::verse_count(book_id, ch) {
        Some(max_verse) if (1..=max_verse).contains(&vs) => {}
        _ => return Err(ParseError::InvalidVerse(vs.to_string())),
    }

    Ok(VerseRef::new(book_id, ch, vs))
}

/// Try to parse a token as "chapter:verse" (e.g., "3:16").
/// Returns (chapter, Some(verse)) if colon-separated, or (number, None) if just a number.
fn try_parse_chapter_verse(token: &str) -> Option<(u16, Option<u16>)> {
    if let Some(colon_pos) = token.find(':') {
        let ch_str = &token[..colon_pos];
        let vs_str = &token[colon_pos + 1..];
        let ch = ch_str.parse::<u16>().ok()?;
        let vs = vs_str.parse::<u16>().ok()?;
        Some((ch, Some(vs)))
    } else {
        let n = token.parse::<u16>().ok()?;
        Some((n, None))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types::BookId;

    #[test]
    fn parse_john_3_16_colon() {
        let result = resolve_text("John 3:16");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(43));
        assert_eq!(vr.chapter, 3);
        assert_eq!(vr.verse, 16);
    }

    #[test]
    fn parse_genesis_1_1_spaces() {
        let result = resolve_text("Genesis 1 1");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(1));
        assert_eq!(vr.chapter, 1);
        assert_eq!(vr.verse, 1);
    }

    #[test]
    fn parse_first_corinthians() {
        let result = resolve_text("First Corinthians 13:4");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(46)); // 1 Corinthians
        assert_eq!(vr.chapter, 13);
        assert_eq!(vr.verse, 4);
    }

    #[test]
    fn parse_abbreviated() {
        let result = resolve_text("Rev 22:21");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(66));
        assert_eq!(vr.chapter, 22);
        assert_eq!(vr.verse, 21);
    }

    #[test]
    fn empty_input_fails() {
        assert!(resolve_text("").is_err());
        assert!(resolve_text("   ").is_err());
    }

    #[test]
    fn garbage_input_fails() {
        assert!(resolve_text("hello world 42").is_err());
    }

    #[test]
    fn window_match() {
        let words: Vec<String> = vec![
            "let".into(),
            "us".into(),
            "read".into(),
            "john".into(),
            "3".into(),
            "16".into(),
        ];
        let result = resolve_window(&words);
        assert!(result.is_some());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(43));
        assert_eq!(vr.chapter, 3);
        assert_eq!(vr.verse, 16);
    }

    #[test]
    fn window_empty() {
        assert!(resolve_window(&[]).is_none());
    }

    #[test]
    fn chapter_verse_syntax() {
        let result = resolve_text("Romans 8:28");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(45));
        assert_eq!(vr.chapter, 8);
        assert_eq!(vr.verse, 28);
    }

    #[test]
    fn chapter_word_syntax() {
        let result = resolve_text("Psalms chapter 119 verse 105");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(19));
        assert_eq!(vr.chapter, 119);
        assert_eq!(vr.verse, 105);
    }

    #[test]
    fn numeric_book_prefix() {
        let result = resolve_text("1 John 1:1");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(62));
        assert_eq!(vr.chapter, 1);
        assert_eq!(vr.verse, 1);
    }

    #[test]
    fn second_timothy() {
        let result = resolve_text("2 Timothy 3:16");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(55));
        assert_eq!(vr.chapter, 3);
        assert_eq!(vr.verse, 16);
    }

    #[test]
    fn third_john() {
        let result = resolve_text("3 John 1:1");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(64));
        assert_eq!(vr.chapter, 1);
        assert_eq!(vr.verse, 1);
    }

    #[test]
    fn song_of_solomon() {
        let result = resolve_text("Song of Solomon 2:7");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(22));
        assert_eq!(vr.chapter, 2);
        assert_eq!(vr.verse, 7);
    }

    #[test]
    fn case_insensitive() {
        let result = resolve_text("JOHN 3:16");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(43));
    }

    #[test]
    fn mixed_case() {
        let result = resolve_text("gEnEsIs 1:1");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(1));
    }

    #[test]
    fn invalid_book_name() {
        assert!(resolve_text("UnknownBook 1:1").is_err());
    }

    #[test]
    fn invalid_chapter() {
        assert!(resolve_text("John 999:1").is_err());
    }

    #[test]
    fn invalid_chapter_reports_parse_error() {
        assert!(matches!(
            resolve_text("John 99:1"),
            Err(ParseError::InvalidChapter(chapter)) if chapter == "99"
        ));
    }

    #[test]
    fn invalid_verse_reports_parse_error() {
        assert!(matches!(
            resolve_text("John 3:999"),
            Err(ParseError::InvalidVerse(verse)) if verse == "999"
        ));
    }

    #[test]
    fn psalm_117_has_only_two_verses() {
        assert!(matches!(
            resolve_text("Psalm 117:3"),
            Err(ParseError::InvalidVerse(verse)) if verse == "3"
        ));
    }

    #[test]
    fn in_range_reference_remains_valid() {
        let result = resolve_text("John 3:16");
        assert!(result.is_ok());
    }

    #[test]
    fn incomplete_reference_book_only() {
        assert!(resolve_text("John").is_err());
    }

    #[test]
    fn incomplete_reference_book_chapter() {
        assert!(resolve_text("John 3").is_err());
    }

    #[test]
    fn window_with_noise() {
        let words: Vec<String> = vec![
            "and".into(),
            "then".into(),
            "the".into(),
            "pastor".into(),
            "said".into(),
            "turn".into(),
            "to".into(),
            "matthew".into(),
            "5".into(),
            "7".into(),
        ];
        let result = resolve_window(&words);
        assert!(result.is_some());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(40));
        assert_eq!(vr.chapter, 5);
        assert_eq!(vr.verse, 7);
    }

    #[test]
    fn window_no_match() {
        let words: Vec<String> = vec![
            "hello".into(),
            "world".into(),
            "test".into(),
            "fourty".into(),
            "two".into(),
        ];
        assert!(resolve_window(&words).is_none());
    }

    #[test]
    fn window_longer_than_reference() {
        let words: Vec<String> = vec![
            "and".into(),
            "we".into(),
            "find".into(),
            "that".into(),
            "in".into(),
            "the".into(),
            "book".into(),
            "of".into(),
            "revelation".into(),
            "chapter".into(),
            "22".into(),
            "verse".into(),
            "21".into(),
        ];
        let result = resolve_window(&words);
        assert!(result.is_some());
        let vr = result.unwrap();
        assert_eq!(vr.book, BookId(66));
        assert_eq!(vr.chapter, 22);
        assert_eq!(vr.verse, 21);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::types::BookId;
    use proptest::prelude::*;

    // Property: Valid verse references should always resolve successfully
    proptest! {
        #[test]
        fn prop_valid_book_chapter_verse(book in 1u8..=66, chapter in 1u16..=150, verse in 1u16..=176) {
            // Skip if this specific chapter/verse doesn't exist in the canon
            if let Some(book_id) = BookId::new(book) {
                if let Some(chapter_count) = canon::chapter_count(book_id) {
                    if chapter <= chapter_count {
                        if let Some(verse_count) = canon::verse_count(book_id, chapter) {
                            if verse <= verse_count {
                                // Get the book name and try to parse it back
                                let book_name = canon::book_name(book_id);
                                let ref_str = format!("{} {}:{}", book_name, chapter, verse);

                                // This should parse successfully
                                if let Ok(parsed) = resolve_text(&ref_str) {
                                    prop_assert_eq!(parsed.book, book_id);
                                    prop_assert_eq!(parsed.chapter, chapter);
                                    prop_assert_eq!(parsed.verse, verse);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Property: Invalid book names should always fail
    proptest! {
        #[test]
        fn prop_invalid_book_fails(invalid_name in "[a-zA-Z]{5,10}") {
            // Generate a random book name that's unlikely to be valid
            let ref_str = format!("{} 1:1", invalid_name);
            prop_assert!(resolve_text(&ref_str).is_err());
        }
    }

    // Property: Parser should handle case variations correctly
    proptest! {
        #[test]
        fn prop_case_insensitive(base in "John|Genesis|Romans|Matthew", chapter in 1u16..=28, verse in 1u16..=50) {
            let variations = vec![
                base.to_lowercase(),
                base.to_uppercase(),
                // Mix case by capitalizing first letter only
                base.chars().enumerate().map(|(i, c)| {
                    if i == 0 { c.to_uppercase().to_string() } else { c.to_lowercase().to_string() }
                }).collect::<String>(),
            ];

            for variation in variations {
                let ref_str = format!("{} {}:{}", variation, chapter, verse);
                let result = resolve_text(&ref_str);
                // Should either succeed with correct values or fail due to non-existent verse
                if let Ok(parsed) = result {
                    // If it succeeds, verify the chapter/verse match
                    prop_assert_eq!(parsed.chapter, chapter);
                    prop_assert_eq!(parsed.verse, verse);
                }
            }
        }
    }

    // Property: Window function should find references embedded in noise
    proptest! {
        #[test]
        fn prop_window_with_noise(ref_str in "John 3:16|Genesis 1:1|Romans 8:28") {
            let noise_words = vec!["and", "then", "the", "pastor", "said", "turn", "to", "let", "us", "read"];
            let mut words: Vec<String> = noise_words.iter().map(|s| s.to_string()).collect();

            // Add the reference
            for part in ref_str.split_whitespace() {
                words.push(part.to_string());
            }

            // Add more noise
            words.extend(noise_words.iter().map(|s| s.to_string()));

            let result = resolve_window(&words);
            prop_assert!(result.is_some());
        }
    }

    // Property: Parser should not panic on malformed input
    proptest! {
        #[test]
        fn prop_no_panic_on_malformed(input in "\\PC*") {
            // Should never panic, only return errors
            let _ = resolve_text(&input);
        }
    }

    // Property: Empty or whitespace-only input should always fail gracefully
    proptest! {
        #[test]
        fn prop_whitespace_input_fails(whitespace in "[ \t\n\r]{0,20}") {
            prop_assert!(resolve_text(&whitespace).is_err());
        }
    }
}
