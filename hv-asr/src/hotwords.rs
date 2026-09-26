//! Hotwords generation for Bible vocabulary biasing.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::collections::HashSet;

/// Generate a hotwords file containing all Bible books and common terms.
pub fn generate_compatible_hotwords(
    hotwords_path: impl AsRef<Path>,
    tokens_path: impl AsRef<Path>,
) -> io::Result<usize> {
    let mut valid_tokens = HashSet::new();
    if let Ok(file) = File::open(tokens_path) {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Some(token) = line.split_whitespace().next() {
                    let cleaned = token.replace(' ', "").replace('▁', "");
                    if !cleaned.is_empty() {
                        valid_tokens.insert(cleaned);
                    }
                }
            }
        }
    }

    let mut file = File::create(hotwords_path)?;
    let mut count = 0;

    // Core religious vocabulary
    let core_terms = [
        "bible", "scripture", "testament", "gospel", "epistle", "psalm", "proverb", "prophet",
        "chapter", "verse", "lord", "god", "jesus", "christ", "holy", "spirit",
    ];

    // Bible book names
    let books = [
        "genesis", "exodus", "leviticus", "numbers", "deuteronomy", "joshua", "judges", "ruth",
        "samuel", "kings", "chronicles", "ezra", "nehemiah", "esther", "job", "psalms",
        "proverbs", "ecclesiastes", "song of solomon", "isaiah", "jeremiah", "lamentations",
        "ezekiel", "daniel", "hosea", "joel", "amos", "obadiah", "jonah", "micah", "nahum",
        "habakkuk", "zephaniah", "haggai", "zechariah", "malachi", "matthew", "mark", "luke",
        "john", "acts", "romans", "corinthians", "galatians", "ephesians", "philippians",
        "colossians", "thessalonians", "timothy", "titus", "philemon", "hebrews", "james",
        "peter", "john", "jude", "revelation",
    ];

    let modifiers = [
        "first", "second", "third",
    ];

    for term in core_terms {
        let is_valid = valid_tokens.is_empty() || valid_tokens.contains(term);
        if is_valid {
            writeln!(file, "{}", term)?;
            count += 1;
        }
    }

    for book in books {
        if !book.contains(' ') {
            let is_valid = valid_tokens.is_empty() || valid_tokens.contains(book);
            if is_valid {
                writeln!(file, "{}", book)?;
                count += 1;
            }
            for modifier in modifiers {
                let modifier_valid = valid_tokens.is_empty() || valid_tokens.contains(modifier);
                let book_valid = valid_tokens.is_empty() || valid_tokens.contains(book);
                if modifier_valid && book_valid {
                    writeln!(file, "{} {}", modifier, book)?;
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}


