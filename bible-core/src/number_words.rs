//! Number-word normalization table.
//!
//! Maps English ordinal and cardinal number words to their numeric values.
//! Used by the reference parser to handle spoken forms like "First Corinthians"
//! or "chapter sixteen". No regex — pure lookup table.

use phf::Map as PhfMap;

/// Static map of English number words (lowercase) to their numeric value.
/// Covers cardinals 1–150 and ordinals 1st–20th (the practical range for
/// Bible chapters and book ordinals).
static NUMBER_WORDS: PhfMap<&'static str, u16> = phf::phf_map! {
    // ── Ordinals (for book names: "First", "Second", "Third") ───
    "first" => 1,
    "second" => 2,
    "third" => 3,
    "fourth" => 4,
    "fifth" => 5,
    "sixth" => 6,
    "seventh" => 7,
    "eighth" => 8,
    "ninth" => 9,
    "tenth" => 10,
    "eleventh" => 11,
    "twelfth" => 12,
    "thirteenth" => 13,
    "fourteenth" => 14,
    "fifteenth" => 15,
    "sixteenth" => 16,
    "seventeenth" => 17,
    "eighteenth" => 18,
    "nineteenth" => 19,
    "twentieth" => 20,

    // ── Cardinals (for chapter/verse numbers) ───────────────────
    "one" => 1,
    "two" => 2,
    "three" => 3,
    "four" => 4,
    "five" => 5,
    "six" => 6,
    "seven" => 7,
    "eight" => 8,
    "nine" => 9,
    "ten" => 10,
    "eleven" => 11,
    "twelve" => 12,
    "thirteen" => 13,
    "fourteen" => 14,
    "fifteen" => 15,
    "sixteen" => 16,
    "seventeen" => 17,
    "eighteen" => 18,
    "nineteen" => 19,
    "twenty" => 20,
    "thirty" => 30,
    "forty" => 40,
    "fifty" => 50,
    "sixty" => 60,
    "seventy" => 70,
    "eighty" => 80,
    "ninety" => 90,
    "hundred" => 100,
};

/// Attempt to parse a word as a number.
///
/// Handles:
/// 1. Numeric strings ("3", "16", "119")
/// 2. English number words ("three", "sixteen", "first")
///
/// Returns `None` if the word is not a recognized number form.
pub fn parse_number_word(word: &str) -> Option<u16> {
    // Try direct numeric parse first (fast path)
    if let Ok(n) = word.parse::<u16>() {
        return Some(n);
    }

    // Try lookup table
    let lower = word.to_lowercase();
    NUMBER_WORDS.get(lower.as_str()).copied()
}

/// Check if a word is an ordinal number word (e.g., "first", "second", "third").
/// These are specifically used for book-name prefixes.
pub fn is_ordinal(word: &str) -> bool {
    let lower = word.to_lowercase();
    matches!(
        lower.as_str(),
        "first"
            | "second"
            | "third"
            | "fourth"
            | "fifth"
            | "sixth"
            | "seventh"
            | "eighth"
            | "ninth"
            | "tenth"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_strings() {
        assert_eq!(parse_number_word("3"), Some(3));
        assert_eq!(parse_number_word("16"), Some(16));
        assert_eq!(parse_number_word("119"), Some(119));
        assert_eq!(parse_number_word("0"), Some(0));
    }

    #[test]
    fn cardinal_words() {
        assert_eq!(parse_number_word("one"), Some(1));
        assert_eq!(parse_number_word("sixteen"), Some(16));
        assert_eq!(parse_number_word("twenty"), Some(20));
        assert_eq!(parse_number_word("hundred"), Some(100));
    }

    #[test]
    fn ordinal_words() {
        assert_eq!(parse_number_word("first"), Some(1));
        assert_eq!(parse_number_word("third"), Some(3));
        assert_eq!(parse_number_word("sixteenth"), Some(16));
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(parse_number_word("First"), Some(1));
        assert_eq!(parse_number_word("SIXTEEN"), Some(16));
        assert_eq!(parse_number_word("Three"), Some(3));
    }

    #[test]
    fn unrecognized() {
        assert_eq!(parse_number_word("hello"), None);
        assert_eq!(parse_number_word(""), None);
        assert_eq!(parse_number_word("abc123"), None);
    }

    #[test]
    fn ordinal_detection() {
        assert!(is_ordinal("first"));
        assert!(is_ordinal("First"));
        assert!(is_ordinal("SECOND"));
        assert!(is_ordinal("third"));
        assert!(!is_ordinal("one"));
        assert!(!is_ordinal("hello"));
    }
}
