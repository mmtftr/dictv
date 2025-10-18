use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::models::DictionaryEntry;
use crate::parser;
use crate::search::SearchEngine;

/// Index manager for dictionaries
pub struct IndexManager {
    data_dir: PathBuf,
    index_dir: PathBuf,
}

impl IndexManager {
    /// Create a new index manager
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
        let base_path = base_dir.as_ref();
        let data_dir = base_path.join("data");
        let index_dir = base_path.join("index");

        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(&index_dir)?;

        Ok(Self {
            data_dir,
            index_dir,
        })
    }

    /// Get the default index manager using system directories
    pub fn default() -> Result<Self> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let base_dir = home.join(".dictv");
        Self::new(base_dir)
    }

    /// Import dictionary from local files
    pub fn import_local<P: AsRef<Path>>(
        &self,
        dict_path: P,
        index_path: P,
        language: &str,
    ) -> Result<()> {
        info!(
            "Importing dictionary from {:?} and {:?}",
            dict_path.as_ref(),
            index_path.as_ref()
        );

        let entries = parser::parse_dict(dict_path, index_path, language)?;
        info!("Parsed {} entries", entries.len());

        self.add_entries_to_index(entries)?;

        Ok(())
    }

    /// Download and import FreeDict dictionary
    pub fn import_freedict(&self, dict_name: &str) -> Result<()> {
        let (url, language, base_name) = match dict_name {
            "freedict-eng-deu" => (
                "https://download.freedict.org/dictionaries/eng-deu/1.9-fd1/freedict-eng-deu-1.9-fd1.dictd.tar.xz",
                "en-de",
                "eng-deu",
            ),
            "freedict-deu-eng" => (
                "https://download.freedict.org/dictionaries/deu-eng/1.9-fd1/freedict-deu-eng-1.9-fd1.dictd.tar.xz",
                "de-en",
                "deu-eng",
            ),
            _ => anyhow::bail!("Unknown dictionary: {}", dict_name),
        };

        info!("Downloading {} from FreeDict", dict_name);

        // Download tar.xz archive
        let tar_path = self.data_dir.join(format!("{}.tar.xz", dict_name));
        download_file(url, &tar_path)?;

        info!("Extracting archive...");

        // Extract the tar.xz archive
        extract_tar_xz(&tar_path, &self.data_dir)?;

        // Find the extracted .dict.dz and .index files by searching recursively
        let (dict_path, index_path) = find_dict_files(&self.data_dir, base_name)?;

        info!("Downloaded successfully, parsing...");

        // Parse and import
        self.import_local(&dict_path, &index_path, language)?;

        // Clean up tar archive
        let _ = fs::remove_file(&tar_path);

        Ok(())
    }

    /// Add entries to the index
    fn add_entries_to_index(&self, entries: Vec<DictionaryEntry>) -> Result<()> {
        // Check if index exists
        let index_exists = self.index_dir.join("meta.json").exists();

        if index_exists {
            // Load existing index and merge
            info!("Existing index found, merging entries");
            // For simplicity, we'll rebuild the entire index
            // In production, you might want to merge incrementally
        }

        SearchEngine::build_index(&self.index_dir, entries)?;

        Ok(())
    }

    /// Rebuild the index from all dictionary files
    pub fn rebuild(&self) -> Result<()> {
        info!("Rebuilding index from all dictionary files");

        // Remove existing index
        if self.index_dir.exists() {
            fs::remove_dir_all(&self.index_dir)?;
            fs::create_dir_all(&self.index_dir)?;
        }

        // Find all dictionary files
        let mut all_entries = Vec::new();

        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("dz") {
                let dict_name = path.file_stem().unwrap().to_str().unwrap();
                let index_path = self.data_dir.join(format!("{}.index", dict_name));

                if index_path.exists() {
                    // Determine language from filename
                    let language = if dict_name.contains("eng-deu") {
                        "en-de"
                    } else if dict_name.contains("deu-eng") {
                        "de-en"
                    } else {
                        "unknown"
                    };

                    info!("Processing {} ({})", dict_name, language);
                    let entries = parser::parse_dict(&path, &index_path, language)?;
                    all_entries.extend(entries);
                }
            }
        }

        info!("Rebuilding index with {} total entries", all_entries.len());
        SearchEngine::build_index(&self.index_dir, all_entries)?;

        Ok(())
    }

    /// Get index statistics
    pub fn stats(&self) -> Result<(usize, usize, usize, u64)> {
        let engine = SearchEngine::new(&self.index_dir)?;
        let (total, en_de, de_en) = engine.get_stats()?;

        let index_size = get_dir_size(&self.index_dir)?;

        Ok((total, en_de, de_en, index_size))
    }

    /// Get the index directory path
    pub fn index_dir(&self) -> &Path {
        &self.index_dir
    }
}

/// Download a file from a URL
fn download_file<P: AsRef<Path>>(url: &str, dest: P) -> Result<()> {
    let response = reqwest::blocking::get(url)?;
    let mut file = fs::File::create(dest)?;
    let content = response.bytes()?;
    std::io::copy(&mut content.as_ref(), &mut file)?;
    Ok(())
}

/// Extract a tar.xz archive
fn extract_tar_xz<P: AsRef<Path>>(archive_path: P, dest_dir: P) -> Result<()> {
    use std::process::Command;

    // Use system tar command for .tar.xz extraction
    let output = Command::new("tar")
        .arg("-xJf")
        .arg(archive_path.as_ref())
        .arg("-C")
        .arg(dest_dir.as_ref())
        .output()
        .context("Failed to execute tar command. Make sure 'tar' is installed.")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to extract archive: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Recursively find .dict.dz and .index files in a directory
fn find_dict_files<P: AsRef<Path>>(base_dir: P, base_name: &str) -> Result<(PathBuf, PathBuf)> {
    let mut dict_file = None;
    let mut index_file = None;

    // Walk through the directory tree
    fn walk_dir(
        dir: &Path,
        base_name: &str,
        dict_file: &mut Option<PathBuf>,
        index_file: &mut Option<PathBuf>,
    ) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    walk_dir(&path, base_name, dict_file, index_file)?;
                } else if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Look for files matching the pattern
                    if file_name.ends_with(".dict.dz") && file_name.contains(base_name) {
                        *dict_file = Some(path.clone());
                    } else if file_name.ends_with(".index") && file_name.contains(base_name) {
                        *index_file = Some(path.clone());
                    }
                }

                // Early exit if we found both files
                if dict_file.is_some() && index_file.is_some() {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    walk_dir(base_dir.as_ref(), base_name, &mut dict_file, &mut index_file)?;

    match (dict_file, index_file) {
        (Some(dict), Some(index)) => Ok((dict, index)),
        (None, Some(_)) => anyhow::bail!(
            "Found .index file but not .dict.dz file for '{}' in {:?}",
            base_name,
            base_dir.as_ref()
        ),
        (Some(_), None) => anyhow::bail!(
            "Found .dict.dz file but not .index file for '{}' in {:?}",
            base_name,
            base_dir.as_ref()
        ),
        (None, None) => anyhow::bail!(
            "Could not find dictionary files for '{}' in {:?}. Files may not have been extracted properly.",
            base_name,
            base_dir.as_ref()
        ),
    }
}

/// Get the total size of a directory
fn get_dir_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let mut total_size = 0u64;

    if path.as_ref().is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += get_dir_size(entry.path())?;
            }
        }
    }

    Ok(total_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_index_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = IndexManager::new(temp_dir.path()).unwrap();

        assert!(manager.data_dir.exists());
        assert!(manager.index_dir.exists());
    }
}
