//! Bible translation normalization module.
//!
//! Converts various Bible JSON formats into a standardized format
//! compatible with the bible-core canon system.

use anyhow::{Context, Result};
use bible_core::canon::{book_name, chapter_count, resolve_book_alias, verse_count};
use bible_core::types::BookId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, stderr, Write};

/// Standard normalized Bible format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedBible {
    pub translation: String,
    pub books: Vec<NormalizedBook>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedBook {
    pub name: String,
    pub chapters: Vec<NormalizedChapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedChapter {
    pub chapter: u16,
    pub verses: Vec<NormalizedVerse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedVerse {
    pub verse: u16,
    pub text: String,
}

/// Scrollmapper database format (nested structure)
#[derive(Debug, Deserialize)]
struct ScrollmapperBible {
    #[allow(dead_code)]
    translation: Option<String>,
    books: Vec<ScrollmapperBook>,
}

#[derive(Debug, Deserialize)]
struct ScrollmapperBook {
    name: String,
    chapters: Vec<ScrollmapperChapter>,
}

#[derive(Debug, Deserialize)]
struct ScrollmapperChapter {
    #[serde(deserialize_with = "deserialize_string_or_number")]
    chapter: u16,
    verses: Vec<ScrollmapperVerse>,
}

#[derive(Debug, Deserialize)]
struct ScrollmapperVerse {
    #[serde(deserialize_with = "deserialize_string_or_number")]
    verse: u16,
    text: String,
}

/// Deserialize either a string or a number into u16
fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(u16),
    }

    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => s.parse::<u16>().map_err(D::Error::custom),
        StringOrNumber::Number(n) => Ok(n),
    }
}

/// WEB format (flat array)
#[derive(Debug, Deserialize)]
struct WebVerse {
    #[allow(dead_code)]
    translation: Option<String>,
    book: String,
    #[serde(deserialize_with = "deserialize_string_or_number")]
    chapter: u16,
    #[serde(deserialize_with = "deserialize_string_or_number")]
    verse: u16,
    text: String,
}

/// Normalize a Bible JSON file to the standard format
pub fn normalize_bible(json_str: &str, translation_code: &str) -> Result<NormalizedBible> {
    // Fix common encoding issues (replace â€œ with proper quotes)
    let cleaned_str = json_str
        .replace("â€œ", "\"")
        .replace("â€", "\"")
        .replace("â€\"", "\"")
        .replace("â€˜", "\"");

    // Try to parse as scrollmapper format first
    if let Ok(scrollmapper) = serde_json::from_str::<ScrollmapperBible>(&cleaned_str) {
        return normalize_scrollmapper(scrollmapper, translation_code);
    }

    // Try to parse as scrollmapper format with relaxed parsing (some sources have different formatting)
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&cleaned_str) {
        if value.is_object() {
            if let Ok(scrollmapper) = serde_json::from_value::<ScrollmapperBible>(value.clone()) {
                return normalize_scrollmapper(scrollmapper, translation_code);
            }
        }
    }

    // Try to parse as WEB format (flat array)
    if let Ok(web_verses) = serde_json::from_str::<Vec<WebVerse>>(&cleaned_str) {
        return normalize_web(web_verses, translation_code);
    }

    // Try to parse as array format (some sources use arrays instead of objects)
    if let Ok(array_data) = serde_json::from_str::<serde_json::Value>(&cleaned_str) {
        if array_data.is_array() {
            // Try to parse as array of WebVerse
            if let Ok(web_verses) = serde_json::from_str::<Vec<WebVerse>>(&cleaned_str) {
                return normalize_web(web_verses, translation_code);
            }
        }
    }

    // For debugging: try to show structure
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&cleaned_str) {
        println!("JSON structure: {}", value.get("translation").and_then(|v| v.as_str()).unwrap_or("no translation field"));
        println!("Has books: {}", value.get("books").is_some());
        println!("First 500 chars: {}", &cleaned_str.chars().take(500).collect::<String>());
        println!("Full JSON keys: {:?}", value.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    }

    anyhow::bail!("Unknown Bible JSON format");
}

fn normalize_scrollmapper(bible: ScrollmapperBible, translation_code: &str) -> Result<NormalizedBible> {
    let mut normalized_books = Vec::new();

    for book in &bible.books {
        // Handle numeric book names (like "1" instead of "Genesis")
        let book_name = if book.name.chars().all(|c| c.is_numeric()) {
            // Convert numeric ID to book name using canon
            let book_id: u8 = book.name.parse().unwrap_or(1);
            if let Some(id) = BookId::new(book_id) {
                book_name(id).to_string()
            } else {
                book.name.clone()
            }
        } else {
            book.name.clone()
        };

        // Skip books that aren't in the Protestant canon (e.g., Apocrypha)
        let _book_id = match resolve_book_alias(&book_name) {
            Ok(id) => id,
            Err(_) => {
                println!("Skipping non-canonical book: {}", book_name);
                continue;
            }
        };

        let mut normalized_chapters = Vec::new();

        for chapter in &book.chapters {
            let mut normalized_verses = Vec::new();

            for verse in &chapter.verses {
                normalized_verses.push(NormalizedVerse {
                    verse: verse.verse,
                    text: verse.text.clone(),
                });
            }

            normalized_chapters.push(NormalizedChapter {
                chapter: chapter.chapter,
                verses: normalized_verses,
            });
        }

        normalized_books.push(NormalizedBook {
            name: book_name,
            chapters: normalized_chapters,
        });
    }

    Ok(NormalizedBible {
        translation: translation_code.to_string(),
        books: normalized_books,
    })
}

fn normalize_web(verses: Vec<WebVerse>, translation_code: &str) -> Result<NormalizedBible> {
    let mut books_map: HashMap<String, Vec<(u16, u16, String)>> = HashMap::new();

    // Group verses by book
    for verse in verses {
        books_map
            .entry(verse.book.clone())
            .or_insert_with(Vec::new)
            .push((verse.chapter, verse.verse, verse.text));
    }

    let mut normalized_books = Vec::new();

    for (book_name, mut verses_list) in books_map {
        // Skip books that aren't in the Protestant canon (e.g., Apocrypha)
        let _book_id = match resolve_book_alias(&book_name) {
            Ok(id) => id,
            Err(_) => {
                println!("Skipping non-canonical book: {}", book_name);
                continue;
            }
        };

        // Sort by chapter, then verse
        verses_list.sort_by_key(|(ch, v, _)| (*ch, *v));

        // Group by chapter
        let mut chapters_map: HashMap<u16, Vec<(u16, String)>> = HashMap::new();
        for (chapter, verse, text) in verses_list {
            chapters_map
                .entry(chapter)
                .or_insert_with(Vec::new)
                .push((verse, text));
        }

        let mut normalized_chapters = Vec::new();
        for (chapter_num, mut verse_entries) in chapters_map {
            verse_entries.sort_by_key(|(v, _)| *v);

            let normalized_verses: Vec<NormalizedVerse> = verse_entries
                .into_iter()
                .map(|(verse_num, text)| NormalizedVerse {
                    verse: verse_num,
                    text,
                })
                .collect();

            normalized_chapters.push(NormalizedChapter {
                chapter: chapter_num,
                verses: normalized_verses,
            });
        }

        normalized_chapters.sort_by_key(|ch| ch.chapter);

        normalized_books.push(NormalizedBook {
            name: book_name,
            chapters: normalized_chapters,
        });
    }

    // Sort books by canonical order
    normalized_books.sort_by_key(|book| {
        resolve_book_alias(&book.name)
            .map(|id| id.as_u8())
            .unwrap_or(255)
    });

    Ok(NormalizedBible {
        translation: translation_code.to_string(),
        books: normalized_books,
    })
}

/// Validate a normalized Bible against the canon
pub fn validate_against_canon(bible: &NormalizedBible) -> Result<ValidationReport> {
    let mut report = ValidationReport {
        translation: bible.translation.clone(),
        total_books: 0,
        expected_books: 66,
        total_chapters: 0,
        total_verses: 0,
        expected_verses: 31102,
        missing_books: Vec::new(),
        missing_chapters: Vec::new(),
        missing_verses: Vec::new(),
        extra_verses: Vec::new(),
    };

    // Build a map of what we have
    let mut book_map: HashMap<u8, HashMap<u16, Vec<u16>>> = HashMap::new();

    for book in &bible.books {
        let book_id = resolve_book_alias(&book.name)
            .context(format!("Unknown book name: {}", book.name))?;

        let mut chapter_map: HashMap<u16, Vec<u16>> = HashMap::new();
        for chapter in &book.chapters {
            let mut verse_nums: Vec<u16> = chapter.verses.iter().map(|v| v.verse).collect();
            verse_nums.sort();
            chapter_map.insert(chapter.chapter, verse_nums);
        }

        book_map.insert(book_id.as_u8(), chapter_map);
    }

    report.total_books = book_map.len() as u32;

    // Check each book in the canon
    for book_id in 1..=66u8 {
        let book = BookId::new(book_id).unwrap();
        let book_name_str = book_name(book);

        if let Some(chapter_map) = book_map.get(&book_id) {
            report.total_chapters += chapter_map.len() as u32;

            // Check each chapter
            if let Some(expected_chapters) = chapter_count(book) {
                for chapter_num in 1..=expected_chapters {
                    if let Some(verse_nums) = chapter_map.get(&chapter_num) {
                        report.total_verses += verse_nums.len() as u32;

                        // Check each verse
                        if let Some(expected_verses) = verse_count(book, chapter_num) {
                            for verse_num in 1..=expected_verses {
                                if !verse_nums.contains(&verse_num) {
                                    report.missing_verses.push(format!(
                                        "{} {}:{}",
                                        book_name_str, chapter_num, verse_num
                                    ));
                                }
                            }

                            // Check for extra verses
                            for &verse_num in verse_nums {
                                if verse_num > expected_verses {
                                    report.extra_verses.push(format!(
                                        "{} {}:{}",
                                        book_name_str, chapter_num, verse_num
                                    ));
                                }
                            }
                        }
                    } else {
                        report.missing_chapters.push(format!("{} {}", book_name_str, chapter_num));
                    }
                }
            }
        } else {
            report.missing_books.push(book_name_str.to_string());
        }
    }

    Ok(report)
}

#[derive(Debug)]
pub struct ValidationReport {
    pub translation: String,
    pub total_books: u32,
    pub expected_books: u32,
    pub total_chapters: u32,
    pub total_verses: u32,
    pub expected_verses: u32,
    pub missing_books: Vec<String>,
    pub missing_chapters: Vec<String>,
    pub missing_verses: Vec<String>,
    pub extra_verses: Vec<String>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.missing_books.is_empty()
            && self.missing_chapters.is_empty()
            && self.missing_verses.is_empty()
            && self.extra_verses.is_empty()
            && self.total_verses == self.expected_verses
    }

    pub fn print_summary(&self) {
        println!("Validation Report for {}", self.translation);
        println!("═══════════════════════════════════════════════════════════");
        println!("Books: {}/{}", self.total_books, self.expected_books);
        println!("Chapters: {}", self.total_chapters);
        println!("Verses: {}/{}", self.total_verses, self.expected_verses);

        if !self.missing_books.is_empty() {
            println!("Missing books: {}", self.missing_books.join(", "));
        }
        if !self.missing_chapters.is_empty() {
            println!("Missing chapters ({}): {}", self.missing_chapters.len(), self.missing_chapters[0]);
            if self.missing_chapters.len() > 1 {
                println!("... and {} more", self.missing_chapters.len() - 1);
            }
        }
        if !self.missing_verses.is_empty() {
            println!("Missing verses ({}): {}", self.missing_verses.len(), self.missing_verses[0]);
            if self.missing_verses.len() > 1 {
                println!("... and {} more", self.missing_verses.len() - 1);
            }
        }
        if !self.extra_verses.is_empty() {
            println!("Extra verses ({}): {}", self.extra_verses.len(), self.extra_verses[0]);
            if self.extra_verses.len() > 1 {
                println!("... and {} more", self.extra_verses.len() - 1);
            }
        }

        if self.is_valid() {
            println!("✓ VALID");
        } else {
            println!("✗ INVALID");
        }
        println!();
    }
}