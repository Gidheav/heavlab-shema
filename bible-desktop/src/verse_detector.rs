// Allow dead code - these will be used in Task 7 when wired into audio pipeline
#![allow(dead_code)]

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

/// Verse detector with rolling word buffer and debounce.
///
/// This sits between ASR transcript output and the UI, implementing
/// the spec's rolling window matching with 3-second debounce.
pub struct VerseDetector {
    /// Rolling buffer of words (max 20, FIFO)
    word_buffer: Vec<String>,
    /// Time of last detection for debounce
    last_detection_time: Option<Instant>,
    /// Bible translation store for verse lookup
    store: Arc<TranslationStore>,
    /// Active translation code (e.g., "KJV")
    active_translation: String,
    /// Debounce duration in seconds
    debounce_duration: std::time::Duration,
}

impl VerseDetector {
    /// Create a new verse detector.
    pub fn new(store: Arc<TranslationStore>, active_translation: String) -> Self {
        Self {
            word_buffer: Vec::with_capacity(20),
            last_detection_time: None,
            store,
            active_translation,
            debounce_duration: std::time::Duration::from_secs(3),
        }
    }

    /// Push transcript text and check for verse detection.
    ///
    /// Tokenizes the text, updates the rolling buffer, and checks
    /// for matches using resolve_window. Returns Some(DetectedVerse)
    /// if a match is found and debounce has expired.
    pub fn push_transcript(&mut self, text: &str) -> Option<DetectedVerse> {
        // Tokenize on whitespace
        let words: Vec<String> = text
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();

        // Push each word into buffer, evicting oldest if > 20
        for word in words {
            if self.word_buffer.len() >= 20 {
                self.word_buffer.remove(0);
            }
            self.word_buffer.push(word);
        }

        // Check for match using resolve_window
        if let Some(verse_ref) = bible_core::parser::resolve_window(&self.word_buffer) {
            // Check debounce: only trigger if 3 seconds have passed since last detection
            if let Some(last_time) = self.last_detection_time {
                if last_time.elapsed() < self.debounce_duration {
                    return None; // Still in debounce period
                }
            }

            // Look up verse text
            let book_id = verse_ref.book;
            if let Ok(index) = bible_core::canon::resolve_index(book_id, verse_ref.chapter, verse_ref.verse) {
                if let Ok(text) = self.store.get_verse(&self.active_translation, index) {
                    let reference = format!("{} {}:{}", 
                        bible_core::canon::book_name(book_id), 
                        verse_ref.chapter, 
                        verse_ref.verse
                    );

                    // Record detection time and clear buffer
                    self.last_detection_time = Some(Instant::now());
                    self.word_buffer.clear();

                    return Some(DetectedVerse {
                        reference,
                        text,
                        book: book_id,
                        chapter: verse_ref.chapter,
                        verse: verse_ref.verse,
                    });
                }
            }
        }

        None
    }

    /// Reset the detector state (clear buffer and debounce timer).
    pub fn reset(&mut self) {
        self.word_buffer.clear();
        self.last_detection_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bible_core::store::TranslationStore;
    use std::sync::Arc;

    fn create_test_detector() -> VerseDetector {
        let store = Arc::new(TranslationStore::new());
        VerseDetector::new(store, "KJV".to_string())
    }

    #[test]
    fn test_word_buffer_capacity() {
        let mut detector = create_test_detector();
        
        // Push 25 words
        for i in 1..=25 {
            detector.push_transcript(&format!("word{}", i));
        }
        
        // Buffer should be capped at 20
        assert_eq!(detector.word_buffer.len(), 20);
        // Should contain the most recent 20 words (word6..word25)
        assert_eq!(detector.word_buffer[0], "word6");
        assert_eq!(detector.word_buffer[19], "word25");
    }

    #[test]
    fn test_word_buffer_fifo_eviction() {
        let mut detector = create_test_detector();
        
        // Push words one by one
        detector.push_transcript("first");
        assert_eq!(detector.word_buffer, vec!["first"]);
        
        detector.push_transcript("second");
        assert_eq!(detector.word_buffer, vec!["first", "second"]);
        
        // Fill to capacity
        for i in 3..=20 {
            detector.push_transcript(&format!("word{}", i));
        }
        assert_eq!(detector.word_buffer.len(), 20);
        assert_eq!(detector.word_buffer[0], "first");
        
        // Add one more - should evict first
        detector.push_transcript("twentyfirst");
        assert_eq!(detector.word_buffer.len(), 20);
        assert_eq!(detector.word_buffer[0], "second");
        assert_eq!(detector.word_buffer[19], "twentyfirst");
    }

    #[test]
    fn test_debounce_logic() {
        let mut detector = create_test_detector();
        
        // Set debounce to a very short duration for testing
        detector.debounce_duration = std::time::Duration::from_millis(10);
        
        // Simulate a detection by setting the last_detection_time directly
        detector.last_detection_time = Some(Instant::now());
        
        // Immediate check should still be in debounce
        assert!(detector.last_detection_time.is_some());
        assert!(detector.last_detection_time.unwrap().elapsed() < detector.debounce_duration);
        
        // Wait for debounce to expire
        std::thread::sleep(detector.debounce_duration + std::time::Duration::from_millis(5));
        
        // Should now be past debounce
        assert!(detector.last_detection_time.unwrap().elapsed() >= detector.debounce_duration);
    }

    #[test]
    fn test_incomplete_reference_no_detection() {
        let mut detector = create_test_detector();
        
        // "John" alone should not trigger detection (no verse data loaded)
        let result = detector.push_transcript("john");
        assert!(result.is_none(), "Incomplete reference should not trigger detection");
    }

    #[test]
    fn test_reset_clears_state() {
        let mut detector = create_test_detector();
        
        // Add some words
        detector.push_transcript("john 3 16");
        assert!(!detector.word_buffer.is_empty());
        
        // Manually set detection time (since we can't detect without data)
        detector.last_detection_time = Some(Instant::now());
        assert!(detector.last_detection_time.is_some());
        
        // Reset
        detector.reset();
        
        // State should be cleared
        assert!(detector.word_buffer.is_empty());
        assert!(detector.last_detection_time.is_none());
    }

    #[test]
    fn test_tokenization_and_case_insensitivity() {
        let mut detector = create_test_detector();
        
        // Test tokenization and case conversion
        detector.push_transcript("JOHN Three SIXTEEN");
        
        // Words should be lowercase
        assert_eq!(detector.word_buffer, vec!["john", "three", "sixteen"]);
    }

    #[test]
    fn test_multiple_pushes_accumulate() {
        let mut detector = create_test_detector();
        
        // Multiple pushes should accumulate
        detector.push_transcript("john");
        assert_eq!(detector.word_buffer, vec!["john"]);
        
        detector.push_transcript("3");
        assert_eq!(detector.word_buffer, vec!["john", "3"]);
        
        detector.push_transcript("16");
        assert_eq!(detector.word_buffer, vec!["john", "3", "16"]);
    }
}