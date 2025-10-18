use serde::{Deserialize, Serialize};

/// Language direction for dictionary lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    EnDe, // English to German
    DeEn, // German to English
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::EnDe => "en-de",
            Language::DeEn => "de-en",
        }
    }
}

impl std::str::FromStr for Language {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en-de" => Ok(Language::EnDe),
            "de-en" => Ok(Language::DeEn),
            _ => Err(anyhow::anyhow!("Invalid language: {}", s)),
        }
    }
}

/// Search mode for dictionary queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    Exact,  // Exact word match
    Fuzzy,  // Fuzzy match with edit distance
    Prefix, // Prefix matching
}

impl std::str::FromStr for SearchMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exact" => Ok(SearchMode::Exact),
            "fuzzy" => Ok(SearchMode::Fuzzy),
            "prefix" => Ok(SearchMode::Prefix),
            _ => Err(anyhow::anyhow!("Invalid search mode: {}", s)),
        }
    }
}

/// Dictionary entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntry {
    pub word: String,
    pub definition: String,
    pub language: String,
}

impl DictionaryEntry {
    pub fn new(word: String, definition: String, language: String) -> Self {
        Self {
            word,
            definition,
            language,
        }
    }
}

/// Search result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub word: String,
    pub definition: String,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_distance: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}

/// Search response
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub query_time_ms: f64,
    pub total_results: usize,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_entries: usize,
    pub en_de_entries: usize,
    pub de_en_entries: usize,
    pub index_size_bytes: u64,
}

/// Search query parameters
#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_search_mode")]
    pub mode: SearchMode,
    #[serde(default = "default_language")]
    pub lang: Language,
    #[serde(default = "default_max_distance")]
    pub max_distance: u8,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_search_mode() -> SearchMode {
    SearchMode::Fuzzy
}

fn default_language() -> Language {
    Language::DeEn
}

fn default_max_distance() -> u8 {
    2
}

fn default_limit() -> usize {
    20
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!("en-de".parse::<Language>().unwrap(), Language::EnDe);
        assert_eq!("de-en".parse::<Language>().unwrap(), Language::DeEn);
        assert!("invalid".parse::<Language>().is_err());
    }

    #[test]
    fn test_search_mode_from_str() {
        assert_eq!("exact".parse::<SearchMode>().unwrap(), SearchMode::Exact);
        assert_eq!("fuzzy".parse::<SearchMode>().unwrap(), SearchMode::Fuzzy);
        assert_eq!("prefix".parse::<SearchMode>().unwrap(), SearchMode::Prefix);
        assert!("invalid".parse::<SearchMode>().is_err());
    }
}
