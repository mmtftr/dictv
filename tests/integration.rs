use dictv::models::{DictionaryEntry, Language, SearchMode};
use dictv::search::SearchEngine;
use tempfile::TempDir;

#[test]
fn test_end_to_end_exact_search() {
    let temp_dir = TempDir::new().unwrap();

    // Create test data
    let entries = vec![
        DictionaryEntry::new(
            "Haus".to_string(),
            "house, building, home".to_string(),
            "de-en".to_string(),
        ),
        DictionaryEntry::new(
            "Auto".to_string(),
            "car, automobile".to_string(),
            "de-en".to_string(),
        ),
    ];

    // Build index
    SearchEngine::build_index(temp_dir.path(), entries).unwrap();

    // Create search engine
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    // Test exact search
    let results = engine
        .search("Haus", SearchMode::Exact, Language::DeEn, 2, 10)
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].word, "haus");
    assert!(results[0].definition.contains("house"));
}

#[test]
fn test_fuzzy_search_single_typo() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![
        DictionaryEntry::new(
            "Haus".to_string(),
            "house, building, home".to_string(),
            "de-en".to_string(),
        ),
        DictionaryEntry::new(
            "Maus".to_string(),
            "mouse".to_string(),
            "de-en".to_string(),
        ),
    ];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    // Search with single character typo
    let results = engine
        .search("Hauss", SearchMode::Fuzzy, Language::DeEn, 1, 10)
        .unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].word, "haus");
    assert_eq!(results[0].edit_distance, Some(1));
}

#[test]
fn test_fuzzy_search_double_typo() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![DictionaryEntry::new(
        "Haus".to_string(),
        "house, building, home".to_string(),
        "de-en".to_string(),
    )];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    // Search with double typo
    let results = engine
        .search("Haaus", SearchMode::Fuzzy, Language::DeEn, 2, 10)
        .unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].word, "haus");
}

#[test]
fn test_case_insensitive_search() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![DictionaryEntry::new(
        "Haus".to_string(),
        "house, building, home".to_string(),
        "de-en".to_string(),
    )];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    // Search with lowercase
    let results = engine
        .search("haus", SearchMode::Exact, Language::DeEn, 2, 10)
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].word, "haus");
}

#[test]
fn test_prefix_search() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![
        DictionaryEntry::new(
            "Haus".to_string(),
            "house".to_string(),
            "de-en".to_string(),
        ),
        DictionaryEntry::new(
            "Haustür".to_string(),
            "front door".to_string(),
            "de-en".to_string(),
        ),
        DictionaryEntry::new(
            "Auto".to_string(),
            "car".to_string(),
            "de-en".to_string(),
        ),
    ];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    let results = engine
        .search("Haus", SearchMode::Prefix, Language::DeEn, 2, 10)
        .unwrap();

    assert!(results.len() >= 1);
    assert!(results.iter().any(|r| r.word == "haus"));
}

#[test]
fn test_language_filtering() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![
        DictionaryEntry::new(
            "Haus".to_string(),
            "house".to_string(),
            "de-en".to_string(),
        ),
        DictionaryEntry::new(
            "house".to_string(),
            "Haus".to_string(),
            "en-de".to_string(),
        ),
    ];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    // Search in German-English
    let results = engine
        .search("Haus", SearchMode::Exact, Language::DeEn, 2, 10)
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].language, "de-en");

    // Search in English-German
    let results = engine
        .search("house", SearchMode::Exact, Language::EnDe, 2, 10)
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].language, "en-de");
}

#[test]
fn test_empty_query() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![DictionaryEntry::new(
        "Haus".to_string(),
        "house".to_string(),
        "de-en".to_string(),
    )];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    let results = engine
        .search("", SearchMode::Exact, Language::DeEn, 2, 10)
        .unwrap();

    // Empty query should return no results
    assert!(results.is_empty());
}

#[test]
fn test_no_matches() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![DictionaryEntry::new(
        "Haus".to_string(),
        "house".to_string(),
        "de-en".to_string(),
    )];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    let results = engine
        .search("xyz", SearchMode::Exact, Language::DeEn, 2, 10)
        .unwrap();

    assert!(results.is_empty());
}

#[test]
fn test_limit_results() {
    let temp_dir = TempDir::new().unwrap();

    let mut entries = Vec::new();
    for i in 0..100 {
        entries.push(DictionaryEntry::new(
            format!("word{}", i),
            format!("definition {}", i),
            "de-en".to_string(),
        ));
    }

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    let results = engine
        .search("word", SearchMode::Prefix, Language::DeEn, 2, 5)
        .unwrap();

    assert!(results.len() <= 5);
}
