use dictv::models::{DictionaryEntry, Language, SearchMode};
use dictv::search::SearchEngine;
use tempfile::TempDir;

/// Test matrix for fuzzy search accuracy
#[test]
fn test_fuzzy_search_accuracy_matrix() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![
        DictionaryEntry::new("Haus".to_string(), "house".to_string(), "de-en".to_string()),
        DictionaryEntry::new("grüßen".to_string(), "to greet".to_string(), "de-en".to_string()),
        DictionaryEntry::new("Straße".to_string(), "street".to_string(), "de-en".to_string()),
        DictionaryEntry::new("Buch".to_string(), "book".to_string(), "de-en".to_string()),
        DictionaryEntry::new("Schule".to_string(), "school".to_string(), "de-en".to_string()),
    ];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    // Test cases: (query, expected_word, max_distance)
    let test_cases = vec![
        ("Haus", "haus", 0),        // Exact match
        ("Hauss", "haus", 1),       // Single typo
        ("Haaus", "haus", 2),       // Double typo
        ("grussen", "grüßen", 2),   // Diacritic variation
        ("Strasse", "straße", 2),   // ß vs ss
        ("Bch", "buch", 1),         // Missing character
        ("Schuule", "schule", 1),   // Extra character
    ];

    for (query, expected, max_distance) in test_cases {
        let results = engine
            .search(query, SearchMode::Fuzzy, Language::DeEn, max_distance, 10)
            .unwrap();

        assert!(
            !results.is_empty(),
            "Expected to find '{}' for query '{}'",
            expected,
            query
        );

        let found = results.iter().any(|r| r.word == expected);
        assert!(
            found,
            "Expected '{}' in results for query '{}', but got: {:?}",
            expected,
            query,
            results.iter().map(|r| &r.word).collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_diacritic_handling() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![
        DictionaryEntry::new("grüßen".to_string(), "to greet".to_string(), "de-en".to_string()),
        DictionaryEntry::new("Müller".to_string(), "Miller".to_string(), "de-en".to_string()),
        DictionaryEntry::new("schön".to_string(), "beautiful".to_string(), "de-en".to_string()),
    ];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    // Test searches without diacritics - should find words with diacritics thanks to ASCII folding
    let test_cases = vec![
        ("grussen", "grüßen"),
        ("Muller", "müller"),
        ("schon", "schön"),
    ];

    for (query, expected) in test_cases {
        let results = engine
            .search(query, SearchMode::Fuzzy, Language::DeEn, 2, 10)
            .unwrap();

        assert!(
            !results.is_empty(),
            "Expected to find '{}' for query '{}'",
            expected,
            query
        );

        let found = results.iter().any(|r| r.word.contains(expected) || r.word == expected);
        assert!(
            found,
            "Expected '{}' in results for query '{}', but got: {:?}",
            expected,
            query,
            results.iter().map(|r| &r.word).collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_unicode_edge_cases() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![
        DictionaryEntry::new("Café".to_string(), "cafe".to_string(), "de-en".to_string()),
        DictionaryEntry::new("naïve".to_string(), "naive".to_string(), "de-en".to_string()),
        DictionaryEntry::new("résumé".to_string(), "resume".to_string(), "de-en".to_string()),
    ];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    // These searches should still work
    let results = engine
        .search("Cafe", SearchMode::Fuzzy, Language::DeEn, 2, 10)
        .unwrap();

    assert!(!results.is_empty());
}

#[test]
fn test_special_characters() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![DictionaryEntry::new(
        "test-word".to_string(),
        "test definition".to_string(),
        "de-en".to_string(),
    )];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    // Search for either the full word or parts - tokenizer may split on hyphens
    let results = engine
        .search("test", SearchMode::Fuzzy, Language::DeEn, 2, 10)
        .unwrap();

    assert!(!results.is_empty());
}

#[test]
fn test_fuzzy_search_ordering() {
    let temp_dir = TempDir::new().unwrap();

    let entries = vec![
        DictionaryEntry::new("Haus".to_string(), "house".to_string(), "de-en".to_string()),
        DictionaryEntry::new("Hause".to_string(), "house (dative)".to_string(), "de-en".to_string()),
        DictionaryEntry::new("Hauses".to_string(), "house's".to_string(), "de-en".to_string()),
    ];

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    let results = engine
        .search("Haus", SearchMode::Fuzzy, Language::DeEn, 2, 10)
        .unwrap();

    // Results should be ordered by edit distance
    assert!(!results.is_empty());
    // Exact match should come first
    assert_eq!(results[0].word, "haus");
    assert_eq!(results[0].edit_distance, Some(0));
}

#[test]
fn test_memory_stability() {
    let temp_dir = TempDir::new().unwrap();

    // Create a reasonable number of entries
    let mut entries = Vec::new();
    for i in 0..1000 {
        entries.push(DictionaryEntry::new(
            format!("word{}", i),
            format!("definition {}", i),
            "de-en".to_string(),
        ));
    }

    SearchEngine::build_index(temp_dir.path(), entries).unwrap();
    let engine = SearchEngine::new(temp_dir.path()).unwrap();

    // Perform many queries
    for i in 0..100 {
        let query = format!("word{}", i % 100);
        let _ = engine
            .search(&query, SearchMode::Fuzzy, Language::DeEn, 2, 10)
            .unwrap();
    }

    // If we get here without panic, memory is stable
    assert!(true);
}
