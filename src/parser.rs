use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use crate::models::DictionaryEntry;

/// DICTD index entry
#[derive(Debug, Clone)]
struct IndexEntry {
    word: String,
    offset: u64,
    length: u64,
}

/// Parse DICTD .index file
pub fn parse_index<P: AsRef<Path>>(path: P) -> Result<Vec<IndexEntry>> {
    let file = File::open(path.as_ref())
        .context(format!("Failed to open index file: {:?}", path.as_ref()))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split('\t').collect();

        if parts.len() >= 3 {
            let word = parts[0].to_string();
            let offset = parts[1]
                .parse::<u64>()
                .context(format!("Invalid offset in index: {}", parts[1]))?;
            let length = parts[2]
                .parse::<u64>()
                .context(format!("Invalid length in index: {}", parts[2]))?;

            entries.push(IndexEntry {
                word,
                offset,
                length,
            });
        }
    }

    Ok(entries)
}

/// Parse DICTD .dict.dz (gzipped dictionary file)
pub fn parse_dict<P: AsRef<Path>>(
    dict_path: P,
    index_path: P,
    language: &str,
) -> Result<Vec<DictionaryEntry>> {
    let index_entries = parse_index(index_path)?;

    // Open and decompress the dictionary file
    let file = File::open(dict_path.as_ref())
        .context(format!("Failed to open dict file: {:?}", dict_path.as_ref()))?;

    let mut decoder = GzDecoder::new(file);
    let mut content = Vec::new();
    decoder.read_to_end(&mut content)?;

    let mut entries = Vec::with_capacity(index_entries.len());

    for index_entry in index_entries {
        let start = index_entry.offset as usize;
        let end = (index_entry.offset + index_entry.length) as usize;

        if end <= content.len() {
            let definition_bytes = &content[start..end];
            let definition = String::from_utf8_lossy(definition_bytes)
                .trim()
                .to_string();

            entries.push(DictionaryEntry::new(
                index_entry.word.clone(),
                clean_definition(&definition),
                language.to_string(),
            ));
        }
    }

    Ok(entries)
}

/// Clean up DICTD definition formatting
fn clean_definition(def: &str) -> String {
    // Remove excessive whitespace and newlines
    let cleaned = def
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    // Remove common DICTD markup
    cleaned
        .replace("\\n", " ")
        .replace("  ", " ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_definition() {
        let input = "  house, building  \n  home  \n\n";
        let expected = "house, building home";
        assert_eq!(clean_definition(input), expected);
    }

    #[test]
    fn test_clean_definition_with_markup() {
        let input = "house\\nbuilding\\n\\nhome";
        let expected = "house building home";
        assert_eq!(clean_definition(input), expected);
    }
}
