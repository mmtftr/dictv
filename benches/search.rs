use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use dictv::index::IndexManager;
use dictv::models::{Language, SearchMode};
use dictv::search::SearchEngine;
use std::sync::{Once, OnceLock};
use tempfile::TempDir;

static INDEX_PATH: OnceLock<TempDir> = OnceLock::new();

/// Download and build index with full dictionary data (once)
fn setup_full_dictionary() -> &'static TempDir {
    INDEX_PATH.get_or_init(|| {
        println!("Setting up full dictionary for benchmarks...");
        println!("This may take a few minutes on first run.");

        let temp_dir = TempDir::new().unwrap();
        let manager = IndexManager::new(temp_dir.path()).unwrap();

        // Try to download both dictionaries
        println!("Downloading freedict-deu-eng...");
        match manager.import_freedict("freedict-deu-eng") {
            Ok(_) => println!("✓ freedict-deu-eng imported"),
            Err(e) => println!("⚠ Failed to download deu-eng ({}), using sample data", e),
        }

        println!("Downloading freedict-eng-deu...");
        match manager.import_freedict("freedict-eng-deu") {
            Ok(_) => println!("✓ freedict-eng-deu imported"),
            Err(e) => println!("⚠ Failed to download eng-deu ({}), using sample data", e),
        }

        println!("✓ Benchmark index ready at {:?}", temp_dir.path());
        temp_dir
    })
}

fn create_benchmark_index() -> SearchEngine {
    let temp_dir = setup_full_dictionary();
    let manager = IndexManager::new(temp_dir.path()).unwrap();
    SearchEngine::new(manager.index_dir()).expect("Failed to open benchmark index")
}

fn bench_exact_search(c: &mut Criterion) {
    let engine = create_benchmark_index();

    c.bench_function("exact_search_haus", |b| {
        b.iter(|| {
            engine
                .search(black_box("Haus"), SearchMode::Exact, Language::DeEn, 2, 10)
                .unwrap()
        })
    });
}

fn bench_fuzzy_search(c: &mut Criterion) {
    let engine = create_benchmark_index();

    c.bench_function("fuzzy_search_hauss_distance_1", |b| {
        b.iter(|| {
            engine
                .search(black_box("Hauss"), SearchMode::Fuzzy, Language::DeEn, 1, 10)
                .unwrap()
        })
    });

    c.bench_function("fuzzy_search_haaus_distance_2", |b| {
        b.iter(|| {
            engine
                .search(black_box("Haaus"), SearchMode::Fuzzy, Language::DeEn, 2, 10)
                .unwrap()
        })
    });
}

fn bench_prefix_search(c: &mut Criterion) {
    let engine = create_benchmark_index();

    c.bench_function("prefix_search_ha", |b| {
        b.iter(|| {
            engine
                .search(black_box("Ha"), SearchMode::Prefix, Language::DeEn, 2, 10)
                .unwrap()
        })
    });
}

fn bench_search_modes(c: &mut Criterion) {
    let engine = create_benchmark_index();
    let mut group = c.benchmark_group("search_modes");

    for mode in [SearchMode::Exact, SearchMode::Fuzzy, SearchMode::Prefix].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", mode)),
            mode,
            |b, &mode| {
                b.iter(|| {
                    engine
                        .search(black_box("Haus"), mode, Language::DeEn, 2, 10)
                        .unwrap()
                })
            },
        );
    }

    group.finish();
}

fn bench_varying_query_lengths(c: &mut Criterion) {
    let engine = create_benchmark_index();
    let mut group = c.benchmark_group("query_lengths");

    for query in ["H", "Ha", "Hau", "Haus"].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(query), query, |b, &query| {
            b.iter(|| {
                engine
                    .search(black_box(query), SearchMode::Fuzzy, Language::DeEn, 2, 10)
                    .unwrap()
            })
        });
    }

    group.finish();
}

fn bench_diacritic_search(c: &mut Criterion) {
    let engine = create_benchmark_index();

    c.bench_function("fuzzy_search_diacritics_grussen", |b| {
        b.iter(|| {
            engine
                .search(
                    black_box("grussen"),
                    SearchMode::Fuzzy,
                    Language::DeEn,
                    2,
                    10,
                )
                .unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_exact_search,
    bench_fuzzy_search,
    bench_prefix_search,
    bench_search_modes,
    bench_varying_query_lengths,
    bench_diacritic_search,
);
criterion_main!(benches);
