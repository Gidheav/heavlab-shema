use clap::{Parser, Subcommand};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

use bible_core::types::{PACK_FORMAT_VERSION, PACK_HEADER_SIZE, PACK_MAGIC};

mod kjv_ingest;
mod pack_writer;
mod versification;
mod download_model;
mod normalize;

/// bible-datagen: Build-time tool for generating Bible data packs.
///
/// This binary is never shipped in the app. It generates:
/// - `*.bible.bin`   — rkyv-archived verse text with format-version header
/// - `*.offsets.bin` — GlobalVerseIndex → byte-offset table per translation
#[derive(Parser)]
#[command(name = "bible-datagen", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a data pack from a tab-separated verse file.
    ///
    /// Input format: one line per verse, `global_index\tverse_text`.
    /// Lines starting with '#' are treated as comments and skipped.
    GeneratePack {
        /// Path to the TSV file containing verse text.
        #[arg(long)]
        input: String,

        /// Output directory for the generated .bible.bin and .offsets.bin files.
        #[arg(long)]
        output_dir: String,

        /// Translation code (e.g., "KJV", "NIV").
        #[arg(long)]
        translation: String,
    },

    /// Validate an existing data pack's BLAKE3 checksum.
    ValidatePack {
        /// Path to the .bible.bin file to validate.
        #[arg(long)]
        pack_path: String,
    },

    /// Generate a sample TSV file with placeholder text for all canonical verses.
    ///
    /// Useful for testing the full pipeline without real translation data.
    GenerateSample {
        /// Output path for the generated TSV file.
        #[arg(long)]
        output: String,
    },

    /// Print versification statistics (book count, chapter count, total verses).
    Stats,

    /// Ingest KJV Bible text from bible-api.com and convert to TSV format.
    IngestKjv {
        /// Output path for the generated TSV file.
        #[arg(long)]
        output: String,
    },

    /// Download the Sherpa-ONNX streaming ASR model.
    DownloadModel,

    /// Process normalized JSON Bible files and generate binary packs.
    ProcessJson {
        /// Path to the JSON file containing the Bible translation.
        #[arg(long)]
        input: String,

        /// Output directory for the generated .bible.bin and .offsets.bin files.
        #[arg(long)]
        output_dir: String,

        /// Translation code (e.g., "KJV", "NIV").
        #[arg(long)]
        translation: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::GeneratePack {
            input,
            output_dir,
            translation,
        } => {
            if let Err(e) = run_generate_pack(&input, &output_dir, &translation) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::ValidatePack { pack_path } => {
            if let Err(e) = run_validate_pack(&pack_path) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::GenerateSample { output } => {
            if let Err(e) = run_generate_sample(&output) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Stats => {
            let stats = versification::compute_stats();
            println!("Versification Statistics (Protestant Canon)");
            println!("───────────────────────────────────────────");
            println!("  Books:    {}", stats.book_count);
            println!("  Chapters: {}", stats.chapter_count);
            println!("  Verses:   {}", stats.verse_count);
        }
        Commands::IngestKjv { output } => {
            if let Err(e) = run_ingest_kjv(&output) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::DownloadModel => {
            if let Err(e) = download_model::run_download_model() {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::ProcessJson {
            input,
            output_dir,
            translation,
        } => {
            if let Err(e) = run_process_json(&input, &output_dir, &translation) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

// ── generate-pack ───────────────────────────────────────────────────

fn run_generate_pack(input: &str, output_dir: &str, translation: &str) -> Result<(), io::Error> {
    let input_path = Path::new(input);
    if !input_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("input file not found: {}", input),
        ));
    }

    let reader = BufReader::new(File::open(input_path)?);
    let mut writer = pack_writer::PackWriter::new(translation, Path::new(output_dir));
    let mut line_count = 0u64;
    let mut verse_count = 0u64;

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let trimmed = line.trim();
        line_count += 1;

        // Skip empty lines and comments.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse: global_index<TAB>verse_text
        let (idx_str, text) = trimmed.split_once('\t').ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "line {}: expected tab-separated 'index\\ttext', got: {}",
                    line_num + 1,
                    if trimmed.len() > 60 {
                        &trimmed[..60]
                    } else {
                        trimmed
                    }
                ),
            )
        })?;

        let global_index: u32 = idx_str.parse().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("line {}: invalid index '{}'", line_num + 1, idx_str),
            )
        })?;

        writer.add_verse(global_index, text);
        verse_count += 1;
    }

    writer.finalize()?;

    println!(
        "✓ Generated {}.bible.bin + {}.offsets.bin",
        translation, translation
    );
    println!(
        "  {} lines read, {} verses packed → {}",
        line_count, verse_count, output_dir
    );

    Ok(())
}

// ── validate-pack ───────────────────────────────────────────────────

fn run_validate_pack(pack_path: &str) -> Result<(), io::Error> {
    let path = Path::new(pack_path);
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("pack file not found: {}", pack_path),
        ));
    }

    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data.len() < PACK_HEADER_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file too small for pack header ({} bytes, need {})",
                data.len(),
                PACK_HEADER_SIZE
            ),
        ));
    }

    // Check magic.
    if data[0..4] != PACK_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "invalid magic: expected {:?}, got {:?}",
                PACK_MAGIC,
                &data[0..4]
            ),
        ));
    }

    // Check version.
    let version = u32::from_le_bytes(
        data[4..8]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "failed to read version"))?,
    );
    if version != PACK_FORMAT_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "version mismatch: expected {}, found {}",
                PACK_FORMAT_VERSION, version
            ),
        ));
    }

    // Read verse count.
    let verse_count =
        u32::from_le_bytes(data[8..12].try_into().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "failed to read verse count")
        })?);

    // Extract stored checksum and recompute.
    let stored_hash: [u8; 32] = data[12..44]
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "failed to read checksum"))?;

    let payload = &data[PACK_HEADER_SIZE..];
    let computed = blake3::hash(payload);

    if stored_hash != *computed.as_bytes() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "BLAKE3 checksum mismatch — pack is corrupt",
        ));
    }

    println!("✓ Pack valid: {}", pack_path);
    println!("  Version:     {}", version);
    println!("  Verses:      {}", verse_count);
    println!("  Payload:     {} bytes", data.len() - PACK_HEADER_SIZE);
    println!("  BLAKE3:      {}", computed.to_hex());

    Ok(())
}

// ── generate-sample ─────────────────────────────────────────────────

fn run_generate_sample(output: &str) -> Result<(), io::Error> {
    let data_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/versification.toml");
    let data = versification::load_from_file(&data_path);

    let output_path = Path::new(output);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut w = io::BufWriter::new(File::create(output_path)?);
    writeln!(w, "# Sample verse data — auto-generated by bible-datagen")?;
    writeln!(w, "# Format: global_index<TAB>verse_text")?;

    let mut global_index: u32 = 0;
    for book in &data.book {
        for (ch_idx, &verse_count) in book.chapters.iter().enumerate() {
            let chapter = ch_idx + 1;
            for verse in 1..=verse_count {
                writeln!(
                    w,
                    "{}\t[{} {}:{}] Placeholder verse text.",
                    global_index, book.name, chapter, verse
                )?;
                global_index += 1;
            }
        }
    }

    w.flush()?;
    println!(
        "✓ Generated sample TSV: {} ({} verses)",
        output, global_index
    );

    Ok(())
}

// ── ingest-kjv ───────────────────────────────────────────────────────

fn run_ingest_kjv(output: &str) -> Result<(), io::Error> {
    let data_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/versification.toml");
    let output_path = Path::new(output);

    match kjv_ingest::ingest_kjv(&data_path, output_path) {
        Ok(()) => Ok(()),
        Err(e) => Err(io::Error::other(
            format!("KJV ingestion failed: {}", e),
        )),
    }
}

// ── process-json ─────────────────────────────────────────────────────

fn run_process_json(input: &str, output_dir: &str, translation: &str) -> Result<(), io::Error> {
    use bible_core::canon::{resolve_index, resolve_book_alias};
    use normalize::{normalize_bible, validate_against_canon};

    let input_path = Path::new(input);
    if !input_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("input file not found: {}", input),
        ));
    }

    println!("Reading JSON file: {}", input);
    let json_content = fs::read_to_string(input_path)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read JSON: {}", e)))?;

    println!("Normalizing to standard format...");
    let normalized = normalize_bible(&json_content, translation)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Normalization failed: {}", e)))?;

    println!("Validating against canon...");
    let validation = validate_against_canon(&normalized)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Validation failed: {}", e)))?;

    validation.print_summary();

    // For now, we'll be lenient and proceed even with minor mismatches
    // Some Bible translations have versification differences from the KJV standard
    if !validation.is_valid() {
        println!("⚠ Warning: Translation has versification differences from canonical structure");
        println!("Proceeding with available verses...");
    }

    println!("Generating binary pack...");
    let mut writer = pack_writer::PackWriter::new(translation, Path::new(output_dir));
    let mut verse_count = 0u64;

    for book in &normalized.books {
        let book_id = resolve_book_alias(&book.name)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Unknown book: {}", e)))?;

        for chapter in &book.chapters {
            for verse in &chapter.verses {
                match resolve_index(book_id, chapter.chapter, verse.verse) {
                    Ok(global_index) => {
                        writer.add_verse(global_index.as_u32(), &verse.text);
                        verse_count += 1;
                    }
                    Err(_) => {
                        // Skip verses that don't exist in the canonical structure
                        // This handles versification differences between translations
                    }
                }
            }
        }
    }

    writer.finalize()?;

    println!(
        "✓ Generated {}.bible.bin + {}.offsets.bin",
        translation, translation
    );
    println!("  {} verses packed → {}", verse_count, output_dir);

    Ok(())
}
