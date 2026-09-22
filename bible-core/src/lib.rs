//! bible-core — Pure logic: verse store, resolver, parsing grammar.
//!
//! This crate has NO FFI, NO OS-specific calls, NO async runtime.
//! All failures return `Result<T, E>` — `.unwrap()` and `.expect()`
//! are denied by clippy at compile time.
#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod canon;
pub mod error;
pub mod number_words;
pub mod parser;
pub mod store;
pub mod types;

// Re-exports for convenience.
pub use canon::{book_name, chapter_count, resolve_book_alias, resolve_index, verse_count};
pub use error::{EngineError, ParseError};
pub use parser::{resolve_text, resolve_window};
pub use types::{BookId, GlobalVerseIndex, TranslationCode, VerseRef};
