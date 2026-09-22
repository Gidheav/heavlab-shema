//! KJV Bible text ingestion module.
//!
//! Downloads the public-domain King James Version text from a reliable source
//! and converts it to the TSV format required by the pack generator.

use crate::versification::{load_from_file, VersificationData};
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

/// Compute the global verse index for a given position in the versification.
fn compute_global_index(
    versification: &VersificationData,
    book_id: u8,
    chapter: u16,
    verse: u16,
) -> Option<u32> {
    let mut global_index: u32 = 0;

    for book in &versification.book {
        if book.id == book_id {
            for ch in 0..chapter as usize - 1 {
                if ch < book.chapters.len() {
                    global_index += book.chapters[ch] as u32;
                }
            }
            global_index += verse as u32 - 1;
            return Some(global_index);
        }
        for &chapter_verses in &book.chapters {
            global_index += chapter_verses as u32;
        }
    }

    None
}

/// Ingest KJV text and write to TSV format.
pub fn ingest_kjv(
    versification_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let versification = load_from_file(versification_path);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create output directory")?;
    }

    println!("Downloading KJV text...");
    let url = "https://raw.githubusercontent.com/scrollmapper/bible_databases/2024/csv/t_kjv.csv";
    
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent("HV-Bible-Datagen/1.0")
        .build();
        
    let response = agent.get(url).call().context("Failed to download KJV CSV")?;
    let body = response.into_string().context("Failed to read response body")?;
    
    println!("Successfully downloaded {} bytes", body.len());

    let output_file = File::create(output_path)
        .context("Failed to create output file")?;
    let mut writer = BufWriter::new(output_file);

    writeln!(writer, "# KJV Bible text")?;
    writeln!(writer, "# Format: global_index<TAB>verse_text")?;
    writeln!(writer, "# Public domain - King James Version")?;

    let mut total_verses = 0u32;
    let mut missing_verses = 0u32;

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("id,") {
            continue;
        }
        
        let parts: Vec<&str> = line.splitn(5, ',').collect();
        if parts.len() < 5 {
            continue;
        }
        
        let book_id: u8 = match parts[1].trim().parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        let chapter: u16 = match parts[2].trim().parse() {
            Ok(c) => c,
            Err(_) => continue,
        };
        let verse: u16 = match parts[3].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let mut text = parts[4].trim();
        
        if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
            text = &text[1..text.len()-1];
        }
        let text = text.replace("\"\"", "\"");

        let global_index = match compute_global_index(
            &versification,
            book_id,
            chapter,
            verse,
        ) {
            Some(idx) => idx,
            None => {
                missing_verses += 1;
                continue;
            }
        };

        writeln!(writer, "{}\t{}", global_index, text)
            .context("Failed to write verse")?;
        total_verses += 1;
    }

    writer.flush().context("Failed to flush output file")?;

    println!("\nIngestion complete:");
    println!("  Total verses written: {}", total_verses);
    println!("  Missing verses: {}", missing_verses);
    println!("  Output: {}", output_path.display());

    if total_verses == 0 {
        anyhow::bail!("No verses were written to output");
    }

    Ok(())
}
