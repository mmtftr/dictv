use anyhow::{Context, Result};
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::{FuzzyTermQuery, Query, QueryParser, RegexQuery};
use tantivy::schema::{STORED, STRING, Schema, TEXT, TextFieldIndexing, TextOptions, Value};
use tantivy::tokenizer::{AsciiFoldingFilter, LowerCaser, SimpleTokenizer, TextAnalyzer};
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term, doc};
use tracing::info;

use crate::models::{DictionaryEntry, Language, SearchMode, SearchResult};

/// Search engine powered by Tantivy
pub struct SearchEngine {
    index: Index,
    reader: IndexReader,
    schema: Schema,
}

impl SearchEngine {
    /// Create a new search engine with the given index directory
    pub fn new<P: AsRef<Path>>(index_path: P) -> Result<Self> {
        let schema = build_schema();
        let mut index = Index::open_in_dir(index_path)?;

        // Register custom tokenizer with ASCII folding for diacritic support
        register_tokenizer(&mut index);

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            index,
            reader,
            schema,
        })
    }

    /// Create a new index at the given path
    pub fn create_index<P: AsRef<Path>>(index_path: P) -> Result<Index> {
        let schema = build_schema();
        std::fs::create_dir_all(index_path.as_ref())?;
        let index = Index::create_in_dir(index_path, schema)?;
        Ok(index)
    }

    /// Build the index from dictionary entries
    pub fn build_index<P: AsRef<Path>>(index_path: P, entries: Vec<DictionaryEntry>) -> Result<()> {
        info!("Building index with {} entries", entries.len());

        let schema = build_schema();
        std::fs::create_dir_all(index_path.as_ref())?;
        let mut index = Index::create_in_dir(index_path, schema.clone())?;

        // Register custom tokenizer with ASCII folding for diacritic support
        register_tokenizer(&mut index);

        let word_field = schema.get_field("word").unwrap();
        let definition_field = schema.get_field("definition").unwrap();
        let language_field = schema.get_field("language").unwrap();

        let mut writer: IndexWriter = index.writer(100_000_000)?;

        for entry in entries {
            writer.add_document(doc!(
                word_field => entry.word.to_lowercase(),
                definition_field => entry.definition,
                language_field => entry.language,
            ))?;
        }

        writer.commit()?;
        info!("Index built successfully");

        Ok(())
    }

    /// Search for a query
    pub fn search(
        &self,
        query: &str,
        mode: SearchMode,
        language: Language,
        max_distance: u8,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();

        let word_field = self.schema.get_field("word").unwrap();
        let definition_field = self.schema.get_field("definition").unwrap();
        let language_field = self.schema.get_field("language").unwrap();

        let normalized_query = query.to_lowercase();
        let lang_str = language.as_str();

        let query: Box<dyn Query> = match mode {
            SearchMode::Exact => {
                // Exact match query
                let term = Term::from_field_text(word_field, &normalized_query);
                Box::new(tantivy::query::TermQuery::new(
                    term,
                    tantivy::schema::IndexRecordOption::Basic,
                ))
            }
            SearchMode::Fuzzy => {
                // Fuzzy match query
                let term = Term::from_field_text(word_field, &normalized_query);
                Box::new(FuzzyTermQuery::new(term, max_distance, true))
            }
            SearchMode::Prefix => {
                // Prefix query using regex
                let regex_pattern = format!("{}.*", regex::escape(&normalized_query));
                Box::new(
                    RegexQuery::from_pattern(&regex_pattern, word_field)
                        .context("Failed to create prefix regex query")?,
                )
            }
        };

        // Execute search
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit * 2))?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            let word = retrieved_doc
                .get_first(word_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let definition = retrieved_doc
                .get_first(definition_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let doc_language = retrieved_doc
                .get_first(language_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Filter by language
            if doc_language != lang_str {
                continue;
            }

            // Calculate edit distance for fuzzy search
            let edit_distance = if mode == SearchMode::Fuzzy {
                Some(strsim::levenshtein(&normalized_query, &word) as u8)
            } else {
                None
            };

            results.push(SearchResult {
                word,
                definition,
                language: doc_language,
                edit_distance,
                score: Some(_score),
            });

            if results.len() >= limit {
                break;
            }
        }

        // Sort by edit distance for fuzzy search
        if mode == SearchMode::Fuzzy {
            results.sort_by_key(|r| r.edit_distance.unwrap_or(255));
        }

        Ok(results)
    }

    /// Get index statistics
    pub fn get_stats(&self) -> Result<(usize, usize, usize)> {
        let searcher = self.reader.searcher();
        let language_field = self.schema.get_field("language").unwrap();

        // Count total documents
        let total = searcher.num_docs() as usize;

        // Count by language (approximate)
        let en_de_query = Term::from_field_text(language_field, "en-de");
        let de_en_query = Term::from_field_text(language_field, "de-en");

        let en_de_count = searcher
            .search(
                &tantivy::query::TermQuery::new(
                    en_de_query,
                    tantivy::schema::IndexRecordOption::Basic,
                ),
                &TopDocs::with_limit(1),
            )?
            .len();

        let de_en_count = searcher
            .search(
                &tantivy::query::TermQuery::new(
                    de_en_query,
                    tantivy::schema::IndexRecordOption::Basic,
                ),
                &TopDocs::with_limit(1),
            )?
            .len();

        Ok((total, en_de_count, de_en_count))
    }
}

/// Register custom tokenizer with ASCII folding for diacritic support
fn register_tokenizer(index: &mut Index) {
    let tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(LowerCaser)
        .filter(AsciiFoldingFilter)
        .build();

    index.tokenizers().register("custom_tokenizer", tokenizer);
}

/// Build the Tantivy schema
fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    // Word field: searchable and stored with custom tokenizer
    let text_field_indexing = TextFieldIndexing::default()
        .set_tokenizer("custom_tokenizer")
        .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions);

    let text_options = TextOptions::default()
        .set_indexing_options(text_field_indexing)
        .set_stored();

    schema_builder.add_text_field("word", text_options.clone());

    // Definition field: searchable and stored with custom tokenizer
    schema_builder.add_text_field("definition", text_options);

    // Language field: filterable and stored
    schema_builder.add_text_field("language", STRING | STORED);

    schema_builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_entries() -> Vec<DictionaryEntry> {
        vec![
            DictionaryEntry::new(
                "Haus".to_string(),
                "house, building, home".to_string(),
                "de-en".to_string(),
            ),
            DictionaryEntry::new(
                "Häuser".to_string(),
                "houses, buildings".to_string(),
                "de-en".to_string(),
            ),
            DictionaryEntry::new(
                "Auto".to_string(),
                "car, automobile".to_string(),
                "de-en".to_string(),
            ),
            DictionaryEntry::new(
                "house".to_string(),
                "Haus, Gebäude".to_string(),
                "en-de".to_string(),
            ),
        ]
    }

    #[test]
    fn test_build_and_search_exact() {
        let temp_dir = TempDir::new().unwrap();
        let entries = create_test_entries();

        SearchEngine::build_index(temp_dir.path(), entries).unwrap();
        let engine = SearchEngine::new(temp_dir.path()).unwrap();

        let results = engine
            .search("Haus", SearchMode::Exact, Language::DeEn, 2, 10)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].word, "haus");
        assert!(results[0].definition.contains("house"));
    }

    #[test]
    fn test_search_fuzzy() {
        let temp_dir = TempDir::new().unwrap();
        let entries = create_test_entries();

        SearchEngine::build_index(temp_dir.path(), entries).unwrap();
        let engine = SearchEngine::new(temp_dir.path()).unwrap();

        // Search with a typo
        let results = engine
            .search("Hauss", SearchMode::Fuzzy, Language::DeEn, 2, 10)
            .unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].word, "haus");
    }

    #[test]
    fn test_search_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let entries = create_test_entries();

        SearchEngine::build_index(temp_dir.path(), entries).unwrap();
        let engine = SearchEngine::new(temp_dir.path()).unwrap();

        let results = engine
            .search("Ha", SearchMode::Prefix, Language::DeEn, 2, 10)
            .unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.word == "haus"));
    }
}
