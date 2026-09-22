//! Normalizes spoken numbers ("three sixteen") and fuzzy book names.

use bible_core::types::BookId;
use std::collections::HashMap;
use std::sync::OnceLock;

static NUMBER_WORDS: OnceLock<HashMap<&'static str, u16>> = OnceLock::new();
static ORDINAL_WORDS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

fn get_number_words() -> &'static HashMap<&'static str, u16> {
    NUMBER_WORDS.get_or_init(|| {
        let mut m = HashMap::new();
        let ones = [
            ("one", 1), ("two", 2), ("three", 3), ("four", 4), ("five", 5),
            ("six", 6), ("seven", 7), ("eight", 8), ("nine", 9),
        ];
        let teens = [
            ("ten", 10), ("eleven", 11), ("twelve", 12), ("thirteen", 13), ("fourteen", 14),
            ("fifteen", 15), ("sixteen", 16), ("seventeen", 17), ("eighteen", 18), ("nineteen", 19),
        ];
        let tens = [
            ("twenty", 20), ("thirty", 30), ("forty", 40), ("fifty", 50),
            ("sixty", 60), ("seventy", 70), ("eighty", 80), ("ninety", 90),
        ];

        for (k, v) in ones.iter().chain(teens.iter()).chain(tens.iter()) {
            m.insert(*k, *v);
        }

        // Add combinations like "twenty one"
        // (Skipped static insertion to avoid allocation; parsed dynamically instead)
        
        m.insert("hundred", 100);
        m
    })
}

fn get_ordinal_words() -> &'static HashMap<&'static str, &'static str> {
    ORDINAL_WORDS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("first", "1");
        m.insert("second", "2");
        m.insert("third", "3");
        m
    })
}

/// Normalizes spoken numbers into digits: "three sixteen" -> "3 16"
pub fn normalize_numbers(text: &str) -> String {
    let num_map = get_number_words();
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut result = Vec::new();
    
    let mut i = 0;
    while i < words.len() {
        let w = words[i];
        
        // Handle ordinals
        if let Some(digit) = get_ordinal_words().get(w) {
            result.push(digit.to_string());
            i += 1;
            continue;
        }
        
        // Handle fuzzy books
        if w == "revelations" {
            result.push("revelation".to_string());
            i += 1;
            continue;
        } else if w == "psalm" {
            result.push("psalms".to_string());
            i += 1;
            continue;
        } else if w == "proverb" {
            result.push("proverbs".to_string());
            i += 1;
            continue;
        }

        if let Some(&val1) = num_map.get(w) {
            let mut total = val1;
            // Check next word for compound numbers like "twenty one" or "hundred"
            if i + 1 < words.len() {
                let w2 = words[i + 1];
                if w2 == "hundred" {
                    total *= 100;
                    i += 1;
                    
                    // check for "hundred and five" or "hundred five"
                    if i + 1 < words.len() {
                        let w3 = words[i + 1];
                        if w3 == "and" && i + 2 < words.len() {
                            if let Some(&val3) = num_map.get(words[i + 2]) {
                                total += val3;
                                i += 2;
                            }
                        } else if let Some(&val3) = num_map.get(w3) {
                            total += val3;
                            i += 1;
                        }
                    }
                } else if val1 >= 20 && val1 <= 90 {
                    if let Some(&val2) = num_map.get(w2) {
                        if val2 < 10 {
                            total += val2;
                            i += 1;
                        }
                    }
                }
            }
            result.push(total.to_string());
        } else {
            result.push(w.to_string());
        }
        i += 1;
    }
    
    result.join(" ")
}
