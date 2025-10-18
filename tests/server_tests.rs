use dictv::index::IndexManager;
use dictv::models::DictionaryEntry;
use dictv::search::SearchEngine;
use dictv::server;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

use std::sync::atomic::{AtomicU16, Ordering};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(14000);

/// Helper to start server in background
async fn setup_test_server() -> (TempDir, u16) {
    let temp_dir = TempDir::new().unwrap();
    let manager = IndexManager::new(temp_dir.path()).unwrap();

    // Create test data
    let entries = vec![
        DictionaryEntry::new("Haus".to_string(), "house, building".to_string(), "de-en".to_string()),
        DictionaryEntry::new("Auto".to_string(), "car, automobile".to_string(), "de-en".to_string()),
        DictionaryEntry::new("Buch".to_string(), "book".to_string(), "de-en".to_string()),
        DictionaryEntry::new("grüßen".to_string(), "to greet".to_string(), "de-en".to_string()),
        DictionaryEntry::new("house".to_string(), "Haus, Gebäude".to_string(), "en-de".to_string()),
    ];

    SearchEngine::build_index(manager.index_dir(), entries).unwrap();

    let engine = SearchEngine::new(manager.index_dir()).unwrap();

    // Use unique port for each test
    let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Start server in background
    tokio::spawn(async move {
        let _ = server::serve(engine, port).await;
    });

    // Give server time to start
    sleep(Duration::from_millis(1000)).await;

    (temp_dir, port)
}

#[tokio::test]
async fn test_server_health_endpoint() {
    let (_temp_dir, port) = setup_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/health", port))
        .send()
        .await
        .expect("Failed to connect to server");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_server_exact_search() {
    let (_temp_dir, port) = setup_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://localhost:{}/search?q=Haus&mode=exact&lang=de-en",
            port
        ))
        .send()
        .await
        .expect("Failed to search");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["total_results"], 1);

    let results = json["results"].as_array().unwrap();
    assert_eq!(results[0]["word"], "haus");
}

#[tokio::test]
async fn test_server_fuzzy_search() {
    let (_temp_dir, port) = setup_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://localhost:{}/search?q=Hauss&mode=fuzzy&lang=de-en&max_distance=1",
            port
        ))
        .send()
        .await
        .expect("Failed to search");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    let results = json["results"].as_array().unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0]["word"], "haus");
}

#[tokio::test]
async fn test_server_diacritic_search() {
    let (_temp_dir, port) = setup_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://localhost:{}/search?q=grussen&mode=fuzzy&lang=de-en&max_distance=2",
            port
        ))
        .send()
        .await
        .expect("Failed to search");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    let results = json["results"].as_array().unwrap();

    assert!(!results.is_empty());
    // Should find grüßen
}

#[tokio::test]
async fn test_server_prefix_search() {
    let (_temp_dir, port) = setup_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://localhost:{}/search?q=Ha&mode=prefix&lang=de-en",
            port
        ))
        .send()
        .await
        .expect("Failed to search");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    let results = json["results"].as_array().unwrap();

    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_server_language_filtering() {
    let (_temp_dir, port) = setup_test_server().await;

    let client = reqwest::Client::new();

    // Test de-en
    let response = client
        .get(format!(
            "http://localhost:{}/search?q=Haus&mode=exact&lang=de-en",
            port
        ))
        .send()
        .await
        .expect("Failed to search");

    let json: serde_json::Value = response.json().await.unwrap();
    let results = json["results"].as_array().unwrap();
    assert_eq!(results[0]["language"], "de-en");

    // Test en-de
    let response = client
        .get(format!(
            "http://localhost:{}/search?q=house&mode=exact&lang=en-de",
            port
        ))
        .send()
        .await
        .expect("Failed to search");

    let json: serde_json::Value = response.json().await.unwrap();
    let results = json["results"].as_array().unwrap();
    assert_eq!(results[0]["language"], "en-de");
}

#[tokio::test]
async fn test_server_empty_query() {
    let (_temp_dir, port) = setup_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/search?q=", port))
        .send()
        .await
        .expect("Failed to send request");

    // Should return 400 Bad Request
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_server_query_performance() {
    let (_temp_dir, port) = setup_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://localhost:{}/search?q=Haus&mode=exact&lang=de-en",
            port
        ))
        .send()
        .await
        .expect("Failed to search");

    let json: serde_json::Value = response.json().await.unwrap();
    let query_time = json["query_time_ms"].as_f64().unwrap();

    // Query should be fast (< 100ms for small dataset)
    assert!(
        query_time < 100.0,
        "Query took {}ms, expected < 100ms",
        query_time
    );
}

#[tokio::test]
async fn test_server_stats_endpoint() {
    let (_temp_dir, port) = setup_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/stats", port))
        .send()
        .await
        .expect("Failed to get stats");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["total_entries"], 5);
}
