#![no_main]
use libfuzzer_sys::fuzz_target;
use bible_core::parser;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignoring invalid UTF-8
    if let Ok(input) = std::str::from_utf8(data) {
        // The parser should never panic on any input
        let _ = parser::resolve_text(input);
        
        // Test window parsing with word-split input
        let words: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
        let _ = parser::resolve_window(&words);
    }
});