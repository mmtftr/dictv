use dictv::index::IndexManager;
use dictv::models::{DictionaryEntry, Language, SearchMode};
use dictv::search::SearchEngine;
use tempfile::TempDir;

#[test]
fn test_cli_import_and_search() {
    let temp_dir = TempDir::new().unwrap();
    let manager = IndexManager::new(temp_dir.path()).unwrap();

    // Simulate importing a small dictionary
    let entries = vec![
        DictionaryEntry::new(
            "Haus".to_string(),
            "house, building".to_string(),
            "de-en".to_string(),
        ),
        DictionaryEntry::new(
            "Auto".to_string(),
            "car".to_string(),
            "de-en".to_string(),
        ),
        DictionaryEntry::new(
            "Buch".to_string(),
            "book".to_string(),
            "de-en".to_string(),
        ),
    ];

    // Build index
    SearchEngine::build_index(manager.index_dir(), entries).unwrap();

    // Verify search works
    let engine = SearchEngine::new(manager.index_dir()).unwrap();
    let results = engine
        .search("Haus", SearchMode::Exact, Language::DeEn, 2, 10)
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].word, "haus");
}

#[test]
fn test_cli_rebuild() {
    let temp_dir = TempDir::new().unwrap();
    let manager = IndexManager::new(temp_dir.path()).unwrap();

    // Import initial data
    let entries = vec![DictionaryEntry::new(
        "Test".to_string(),
        "test".to_string(),
        "de-en".to_string(),
    )];

    SearchEngine::build_index(manager.index_dir(), entries).unwrap();

    // Rebuild with no dictionary files should succeed (empty rebuild)
    // Note: In real usage, dictionary files would be in data/ directory
    manager.rebuild().unwrap();

    // After rebuilding with no data files, index should be empty
    let engine = SearchEngine::new(manager.index_dir()).unwrap();
    let results = engine
        .search("Test", SearchMode::Exact, Language::DeEn, 2, 10)
        .unwrap();

    // Empty index is expected when no dictionary files are present
    assert_eq!(results.len(), 0);
}

#[test]
fn test_cli_stats() {
    let temp_dir = TempDir::new().unwrap();
    let manager = IndexManager::new(temp_dir.path()).unwrap();

    // Import test data with both language directions
    let entries = vec![
        DictionaryEntry::new("Haus".to_string(), "house".to_string(), "de-en".to_string()),
        DictionaryEntry::new("Auto".to_string(), "car".to_string(), "de-en".to_string()),
        DictionaryEntry::new("house".to_string(), "Haus".to_string(), "en-de".to_string()),
    ];

    SearchEngine::build_index(manager.index_dir(), entries).unwrap();

    // Get stats
    let (total, _en_de, _de_en, size) = manager.stats().unwrap();

    assert_eq!(total, 3);
    assert!(size > 0);
}

#[test]
fn test_data_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let manager = IndexManager::new(temp_dir.path()).unwrap();

    // Verify directory structure is created
    assert!(temp_dir.path().join("data").exists());
    assert!(temp_dir.path().join("index").exists());

    // Verify we can get the index directory
    assert_eq!(manager.index_dir(), temp_dir.path().join("index"));
}

#[test]
fn test_fuzzy_search_via_cli() {
    let temp_dir = TempDir::new().unwrap();
    let manager = IndexManager::new(temp_dir.path()).unwrap();

    let entries = vec![
        DictionaryEntry::new("Haus".to_string(), "house".to_string(), "de-en".to_string()),
        DictionaryEntry::new("grüßen".to_string(), "to greet".to_string(), "de-en".to_string()),
    ];

    SearchEngine::build_index(manager.index_dir(), entries).unwrap();

    let engine = SearchEngine::new(manager.index_dir()).unwrap();

    // Test fuzzy with typo
    let results = engine
        .search("Hauss", SearchMode::Fuzzy, Language::DeEn, 1, 10)
        .unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].word, "haus");

    // Test fuzzy with diacritic variation
    let results = engine
        .search("grussen", SearchMode::Fuzzy, Language::DeEn, 2, 10)
        .unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_prefix_search_via_cli() {
    let temp_dir = TempDir::new().unwrap();
    let manager = IndexManager::new(temp_dir.path()).unwrap();

    let entries = vec![
        DictionaryEntry::new("Haus".to_string(), "house".to_string(), "de-en".to_string()),
        DictionaryEntry::new("Haustür".to_string(), "front door".to_string(), "de-en".to_string()),
        DictionaryEntry::new("Auto".to_string(), "car".to_string(), "de-en".to_string()),
    ];

    SearchEngine::build_index(manager.index_dir(), entries).unwrap();

    let engine = SearchEngine::new(manager.index_dir()).unwrap();

    // Test prefix search
    let results = engine
        .search("Haus", SearchMode::Prefix, Language::DeEn, 2, 10)
        .unwrap();

    assert!(results.len() >= 1);
    assert!(results.iter().any(|r| r.word == "haus"));
}

#[test]
fn test_language_filtering_via_cli() {
    let temp_dir = TempDir::new().unwrap();
    let manager = IndexManager::new(temp_dir.path()).unwrap();

    let entries = vec![
        DictionaryEntry::new("Haus".to_string(), "house".to_string(), "de-en".to_string()),
        DictionaryEntry::new("house".to_string(), "Haus".to_string(), "en-de".to_string()),
    ];

    SearchEngine::build_index(manager.index_dir(), entries).unwrap();

    let engine = SearchEngine::new(manager.index_dir()).unwrap();

    // Search de-en
    let results = engine
        .search("Haus", SearchMode::Exact, Language::DeEn, 2, 10)
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].language, "de-en");

    // Search en-de
    let results = engine
        .search("house", SearchMode::Exact, Language::EnDe, 2, 10)
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].language, "en-de");
}
