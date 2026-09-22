//! Criterion benchmarks for bible-core.
//!
//! Charter targets:
//!   - `resolve_index` (get_verse): < 100 μs on mid-range ARM
//!   - `resolve_text`: < 10 μs on a random 20-char input

use bible_core::types::BookId;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_resolve_index(c: &mut Criterion) {
    c.bench_function("resolve_index (John 3:16)", |b| {
        b.iter(|| {
            let _ = bible_core::canon::resolve_index(
                black_box(BookId(43)),
                black_box(3),
                black_box(16),
            );
        });
    });

    c.bench_function("resolve_index (Rev 22:21 - last verse)", |b| {
        b.iter(|| {
            let _ = bible_core::canon::resolve_index(
                black_box(BookId(66)),
                black_box(22),
                black_box(21),
            );
        });
    });

    c.bench_function("resolve_index (Gen 1:1 - first verse)", |b| {
        b.iter(|| {
            let _ =
                bible_core::canon::resolve_index(black_box(BookId(1)), black_box(1), black_box(1));
        });
    });
}

fn bench_resolve_text(c: &mut Criterion) {
    c.bench_function("resolve_text (\"John 3:16\")", |b| {
        b.iter(|| {
            let _ = bible_core::parser::resolve_text(black_box("John 3:16"));
        });
    });

    c.bench_function("resolve_text (\"Genesis 1 1\")", |b| {
        b.iter(|| {
            let _ = bible_core::parser::resolve_text(black_box("Genesis 1 1"));
        });
    });

    c.bench_function("resolve_text (\"Rev 22:21\")", |b| {
        b.iter(|| {
            let _ = bible_core::parser::resolve_text(black_box("Rev 22:21"));
        });
    });

    c.bench_function("resolve_text (abbreviated \"Psa 119:105\")", |b| {
        b.iter(|| {
            let _ = bible_core::parser::resolve_text(black_box("Psa 119:105"));
        });
    });
}

fn bench_book_alias_lookup(c: &mut Criterion) {
    c.bench_function("resolve_book_alias (\"genesis\")", |b| {
        b.iter(|| {
            let _ = bible_core::canon::resolve_book_alias(black_box("genesis"));
        });
    });

    c.bench_function("resolve_book_alias (\"rev\")", |b| {
        b.iter(|| {
            let _ = bible_core::canon::resolve_book_alias(black_box("rev"));
        });
    });
}

criterion_group!(
    benches,
    bench_resolve_index,
    bench_resolve_text,
    bench_book_alias_lookup
);
criterion_main!(benches);
