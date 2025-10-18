# dictv - German-English Dictionary Server

## Project Overview
High-performance, self-hosted German-English dictionary server with fuzzy search capabilities. Designed for instant lookups via Raycast and other clients. Architecture inspired by jpv.

**Language**: Rust (edition 2024)

## Performance Targets
| Metric | Target | Stretch Goal |
|--------|--------|--------------|
| Exact Query Latency | <10ms | <5ms |
| Fuzzy Query Latency | <30ms | <15ms |
| Memory Usage | <150MB | <100MB |
| Startup Time | <3s | <2s |
| Index Build | <60s | <30s |
| Index Size | <100MB | <50MB |
| Concurrent Requests | 100 req/s @ <30ms p95 | 500 req/s |

## Core Requirements

### 1. Search Engine
- **Implementation**: Tantivy full-text search
- **Fuzzy Matching**: Levenshtein distance (edit distance 0-2)
- **Features**: Case-insensitive, diacritic folding, prefix matching
- **Search Modes**: exact, fuzzy, prefix

### 2. Dictionary Management

**Supported Format**: DICTD (.dict.dz + .index files)

**Data Sources**:
- FreeDict eng-deu (English→German, ~460k entries)
- FreeDict deu-eng (German→English, ~517k entries)

**CLI Commands**:
```bash
dictv import --download freedict-eng-deu
dictv import --local path/to/dict.dict.dz path/to/dict.index
dictv rebuild
dictv stats
dictv serve [--daemon] [--port 3000]
dictv query "Haus"
```

### 3. HTTP API

**Server**: Axum (localhost:3000)

**Endpoints**:
- `GET /search?q={query}&mode={exact|fuzzy|prefix}&lang={en-de|de-en}&max_distance={1-2}`
- `GET /health`
- `GET /stats`

**Response Format**: JSON with results array, query time, match metadata

### 4. Testing Requirements

**Must Handle**:
- Single-character typos: "Hauss" → "Haus"
- Double typos: "Haaus" → "Haus"
- Case variations: "haus" → "Haus"
- Diacritics: "Grussen" → "grüßen"
- Unicode edge cases
- Empty/special character queries

**Test Coverage**:
- Unit tests: parser, search logic, indexer
- Integration tests: end-to-end API
- Performance benchmarks: criterion.rs
- Load testing: 100 concurrent requests
- Fuzzy search accuracy matrix

**Required Test Cases**:
```
assert_finds("Haus", "Hauss", distance=1)
assert_finds("grüßen", "grussen", distance=2)
assert_latency("Haus", exact, <5ms)
assert_latency("Hauss", fuzzy, <30ms)
assert_memory_stable(10_000_queries)
```

## Tech Stack

**Core Dependencies**:
- `tantivy` - Full-text search
- `axum` - HTTP server
- `tokio` - Async runtime
- `serde` + `serde_json` - Serialization
- `flate2` - Gzip decompression
- `clap` - CLI parser
- `anyhow` - Error handling

**Dev Dependencies**:
- `criterion` - Benchmarking
- `tokio-test` - Async testing
- `reqwest` - Integration tests

## Project Structure
```
dictv/
├── src/
│   ├── main.rs           # CLI
│   ├── server.rs         # HTTP API
│   ├── search.rs         # Search engine
│   ├── parser.rs         # DICTD parser
│   ├── index.rs          # Index management
│   └── models.rs         # Types
├── tests/
│   ├── integration.rs
│   └── fuzzy_search.rs
├── benches/
│   └── search.rs
└── Cargo.toml
```

## Architecture

**Data Flow**:
```
DICTD Files → Parser → Tantivy Index → Memory-mapped Index → HTTP Server → Client
```

**Tantivy Schema**: word (TEXT|STORED), definition (TEXT|STORED), language (STRING|STORED)

## Raycast Integration

Script should query `http://localhost:3000/search?q={word}&mode=fuzzy&max_distance=2` and format results as:
```
• Haus: house, building, home
• Häuser: houses, buildings
```

## Success Criteria
- [ ] All performance targets met
- [ ] FreeDict import works automatically
- [ ] Fuzzy search handles common misspellings (tested)
- [ ] <30ms p95 latency maintained under load
- [ ] Memory usage stable <150MB
- [ ] >80% test coverage
- [ ] Raycast integration working
- [ ] Complete documentation

## Deliverables
1. Working Rust implementation
2. Comprehensive test suite with benchmarks
3. README with setup instructions
4. Raycast integration example
5. Performance benchmark report

## Optional Future Enhancements
- Additional dictionary formats (StarDict, MDX)
- Web UI
- Multiple language pair support
- Word frequency ranking
- Caching layer
- System service integration (launchd/systemd)
