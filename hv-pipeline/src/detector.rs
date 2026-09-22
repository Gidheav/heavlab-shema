//! Rolling-window verse radar.

use std::sync::Arc;
use std::time::Instant;

use bible_core::store::TranslationStore;
use bible_core::types::BookId;

/// A detected verse with its reference and text.
#[derive(Debug, Clone)]
pub struct DetectedVerse {
    pub reference: String,
    pub text: String,
    pub book: BookId,
    pub chapter: u16,
    pub verse: u16,
}

/// Watches the ASR transcript stream and matches Bible verse references.
pub struct VerseDetector {
    pub(crate) word_buffer: Vec<String>,
    pub(crate) last_detection_time: Option<Instant>,
    store: Arc<TranslationStore>,
    active_translation: String,
    pub(crate) debounce_duration: std::time::Duration,
    pub(crate) last_context: Option<(BookId, u16)>,
}

impl VerseDetector {
    pub fn new(store: Arc<TranslationStore>, active_translation: String) -> Self {
        Self {
            word_buffer: Vec::with_capacity(30),
            last_detection_time: None,
            store,
            active_translation,
            debounce_duration: std::time::Duration::from_secs(3),
            last_context: None,
        }
    }

    pub fn set_translation(&mut self, translation: String) {
        self.active_translation = translation;
    }

    /// Tokenize, update the 30-word window, and return a match after debounce.
    pub fn push_transcript(&mut self, text: &str) -> Option<DetectedVerse> {
        let normalized = crate::normalizer::normalize_numbers(text);
        for word in normalized.split_whitespace().map(str::to_lowercase) {
            if self.word_buffer.len() >= 30 {
                self.word_buffer.remove(0);
            }
            self.word_buffer.push(word);
        }

        let verse_ref = bible_core::parser::resolve_window(&self.word_buffer)?;
        if let Some(last_time) = self.last_detection_time {
            if last_time.elapsed() < self.debounce_duration {
                return None;
            }
        }

        let book_id = verse_ref.book;
        let chapter = verse_ref.chapter;
        let verse = verse_ref.verse;

        // If it's a "lone verse" (usually chapter 1 if book is missing, but bible_core might fail earlier. 
        // Assuming bible_core returns Genesis 1:X if only a verse is spoken? We'd need to adapt bible_core 
        // to return partial matches. For now, let's just save context for future use).
        self.last_context = Some((book_id, chapter));

        let reference = format!(
            "{} {}:{}",
            bible_core::canon::book_name(book_id),
            verse_ref.chapter,
            verse_ref.verse
        );

        let text = bible_core::canon::resolve_index(book_id, verse_ref.chapter, verse_ref.verse)
            .ok()
            .and_then(|index| {
                self.store
                    .get_verse(&self.active_translation, index)
                    .ok()
            })
            .unwrap_or_default();

        self.last_detection_time = Some(Instant::now());
        self.word_buffer.clear();

        Some(DetectedVerse {
            reference,
            text,
            book: book_id,
            chapter: verse_ref.chapter,
            verse: verse_ref.verse,
        })
    }

    pub fn reset(&mut self) {
        self.word_buffer.clear();
        self.last_detection_time = None;
        self.last_context = None;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use bible_core::store::TranslationStore;
    use std::sync::Arc;

    fn create_test_detector() -> VerseDetector {
        VerseDetector::new(Arc::new(TranslationStore::new()), "KJV".to_string())
    }

    #[test]
    fn word_buffer_capacity() {
        let mut detector = create_test_detector();
        for i in 1..=35 {
            detector.push_transcript(&format!("word{i}"));
        }
        assert_eq!(detector.word_buffer.len(), 30);
        assert_eq!(detector.word_buffer[0], "word6");
        assert_eq!(detector.word_buffer[29], "word35");
    }

    #[test]
    fn john_3_16_emits_without_pack() {
        let mut detector = create_test_detector();
        let hit = detector
            .push_transcript("john 3:16")
            .expect("parser should match John 3:16");
        assert_eq!(hit.book.as_u8(), 43);
        assert_eq!(hit.chapter, 3);
        assert_eq!(hit.verse, 16);
        assert!(hit.reference.to_lowercase().contains("john"));
    }

    #[test]
    fn incomplete_reference_no_detection() {
        let mut detector = create_test_detector();
        assert!(detector.push_transcript("john").is_none());
    }

    #[test]
    fn reset_clears_state() {
        let mut detector = create_test_detector();
        detector.push_transcript("and the preacher said");
        detector.last_detection_time = Some(Instant::now());
        detector.reset();
        assert!(detector.word_buffer.is_empty());
        assert!(detector.last_detection_time.is_none());
    }

    #[test]
    fn tokenization_lowercases() {
        let mut detector = create_test_detector();
        detector.push_transcript("Hello THERE preacher");
        assert_eq!(
            detector.word_buffer,
            vec!["hello", "there", "preacher"]
        );
    }
}
