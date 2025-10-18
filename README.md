# dictv - German-English Dictionary Server

High-performance, self-hosted German-English dictionary server with fuzzy search capabilities. Designed for instant lookups via Raycast and other clients.

## Features

- **Fast Full-Text Search**: Powered by Tantivy search engine
- **Fuzzy Matching**: Handles typos and spelling variations (Levenshtein distance)
- **Diacritics Support**: Automatically normalizes diacritics (ü→u, ß→ss, etc.) using ASCII folding
- **Multiple Search Modes**: Exact, fuzzy, and prefix matching with regex support
- **DICTD Format Support**: Import from FreeDict and other DICTD-compatible sources
- **HTTP API**: RESTful API for easy integration
- **CLI Interface**: Complete command-line tool for management and queries
- **Bidirectional**: Supports both German→English and English→German lookups

## Performance

- Exact queries: <5ms
- Fuzzy queries: <30ms
- Memory usage: <150MB
- Startup time: <3s

## Installation

### Prerequisites

- Rust 2024 edition or later
- ~200MB disk space for dictionaries and index

### Build from Source

```bash
git clone https://github.com/yourusername/dictv.git
cd dictv
cargo build --release
```

The binary will be available at `target/release/dictv`.

### Install

```bash
cargo install --path .
```

## Quick Start

### 1. Import Dictionary Data

Import FreeDict German-English dictionary:

```bash
dictv import --download freedict-deu-eng
```

Import FreeDict English-German dictionary:

```bash
dictv import --download freedict-eng-deu
```

Or import from local DICTD files:

```bash
dictv import --local dict.dict.dz --index dict.index --lang de-en
```

### 2. Start the Server

```bash
dictv serve --port 3000
```

The server will start at `http://localhost:3000`.

### 3. Query the Dictionary

Via CLI:

```bash
dictv query Haus
dictv query "Hauss" --mode fuzzy --max-distance 2
dictv query "Ha" --mode prefix
```

Via HTTP API:

```bash
curl "http://localhost:3000/search?q=Haus&mode=fuzzy&lang=de-en"
```

## CLI Commands

### Import Dictionary

```bash
# Download from FreeDict
dictv import --download freedict-deu-eng
dictv import --download freedict-eng-deu

# Import local files
dictv import --local path/to/dict.dict.dz --index path/to/dict.index --lang de-en
```

### Rebuild Index

Rebuild the search index from all imported dictionaries:

```bash
dictv rebuild
```

### View Statistics

```bash
dictv stats
```

### Start HTTP Server

```bash
dictv serve --port 3000
```

### Direct Query

```bash
dictv query "Haus" --mode fuzzy --lang de-en --max-distance 2 --limit 10
```

## HTTP API

### Search

```
GET /search?q={query}&mode={exact|fuzzy|prefix}&lang={en-de|de-en}&max_distance={1-2}&limit={n}
```

**Parameters:**
- `q` (required): Search query
- `mode` (optional): Search mode - `exact`, `fuzzy`, or `prefix` (default: `fuzzy`)
- `lang` (optional): Language direction - `de-en` or `en-de` (default: `de-en`)
- `max_distance` (optional): Maximum edit distance for fuzzy search, 1-2 (default: `2`)
- `limit` (optional): Maximum number of results (default: `20`)

**Response:**

```json
{
  "results": [
    {
      "word": "haus",
      "definition": "house, building, home",
      "language": "de-en",
      "edit_distance": 0,
      "score": 1.5
    }
  ],
  "query_time_ms": 4.2,
  "total_results": 1
}
```

### Health Check

```
GET /health
```

**Response:**

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### Statistics

```
GET /stats
```

**Response:**

```json
{
  "total_entries": 977000,
  "en_de_entries": 460000,
  "de_en_entries": 517000,
  "index_size_bytes": 85000000
}
```

## Raycast Integration

Create a new Raycast Script Command:

```bash
#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Dictionary Lookup
# @raycast.mode compact

# Optional parameters:
# @raycast.icon 📖
# @raycast.argument1 { "type": "text", "placeholder": "Word" }

query="$1"

if [ -z "$query" ]; then
    echo "Please provide a word to look up"
    exit 1
fi

response=$(curl -s "http://localhost:3000/search?q=$query&mode=fuzzy&max_distance=2&limit=5")

# Parse and format the response
echo "$response" | jq -r '.results[] | "• \(.word): \(.definition)"'
```

Save this as `dictionary-lookup.sh` in your Raycast scripts directory and make it executable:

```bash
chmod +x dictionary-lookup.sh
```

## Development

### Run Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test integration
cargo test --test fuzzy_search

# Run with output
cargo test -- --nocapture
```

### Run Benchmarks

```bash
cargo bench
```

Benchmark results will be saved to `target/criterion/`.

### Enable Debug Logging

```bash
RUST_LOG=debug dictv serve
```

## Project Structure

```
dictv/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library exports
│   ├── models.rs        # Data structures
│   ├── parser.rs        # DICTD format parser
│   ├── search.rs        # Tantivy search engine
│   ├── index.rs         # Index management
│   └── server.rs        # HTTP API server
├── tests/
│   ├── integration.rs   # Integration tests
│   └── fuzzy_search.rs  # Fuzzy search tests
├── benches/
│   └── search.rs        # Performance benchmarks
└── Cargo.toml
```

## Data Sources

### FreeDict

- German→English: ~517,000 entries
- English→German: ~460,000 entries
- License: GPL
- Source: https://github.com/freedict/fd-dictionaries

## Performance Tuning

### Index Location

By default, dictv stores data in `~/.dictv/`. To use a different location, modify the `IndexManager::default()` implementation.

### Memory Usage

To reduce memory usage, adjust the Tantivy index settings in `search.rs`:

```rust
let mut writer: IndexWriter = index.writer(50_000_000)?; // Reduce from 100MB
```

### Search Performance

For better fuzzy search performance, reduce `max_distance`:

```bash
dictv query "word" --max-distance 1  # Faster than 2
```

## Troubleshooting

### Server won't start

Check if port 3000 is already in use:

```bash
lsof -i :3000
```

Use a different port:

```bash
dictv serve --port 3001
```

### Import fails

Ensure you have enough disk space (~200MB) and network connectivity.

### Slow queries

- Rebuild the index: `dictv rebuild`
- Reduce `max_distance` for fuzzy searches
- Use exact search when possible

### Memory issues

If memory usage is too high, try:
- Reducing the index writer buffer size
- Limiting result count
- Using exact search mode instead of fuzzy

## License

MIT License - See LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Roadmap

- [ ] Additional dictionary formats (StarDict, MDX)
- [ ] Web UI
- [ ] Multiple language pair support
- [ ] Word frequency ranking
- [ ] Caching layer
- [ ] System service integration (launchd/systemd)
- [ ] Daemon mode with background service

## Acknowledgments

- [Tantivy](https://github.com/tantivy-search/tantivy) - Full-text search engine
- [FreeDict](https://freedict.org/) - Free dictionary data
- Inspired by [jpv](https://github.com/yourusername/jpv)
