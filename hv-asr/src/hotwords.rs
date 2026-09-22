//! Hotwords generation for Bible vocabulary biasing.

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Generate a hotwords file containing all Bible books and common terms.
pub fn generate_hotwords_file(path: impl AsRef<Path>) -> io::Result<()> {
    let mut file = File::create(path)?;
    
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
        writeln!(file, "{}", term)?;
    }
    
    for book in books {
        writeln!(file, "{}", book)?;
        for modifier in modifiers {
            writeln!(file, "{} {}", modifier, book)?;
        }
    }

    Ok(())
}
