# Laverna

[![CI](https://github.com/nutypebuddha/Laverna/actions/workflows/ci.yml/badge.svg)](https://github.com/nutypebuddha/Laverna/actions/workflows/ci.yml)
[![Release](https://github.com/nutypebuddha/Laverna/actions/workflows/release.yml/badge.svg)](https://github.com/nutypebuddha/Laverna/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Deterministic Vedic reasoning engine — 9-graha wheel, NAND-to-bankai compute.

Laverna maps natural language queries onto a 9-graha (planetary) wheel and reasons downward through a 4-layer pipeline to provable NAND-gate truth. It never guesses — out-of-scope input fails loudly rather than hallucinating.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Layer 3 — Bankai      │ Expression verification,       │
│                         │ diagnostics, feedback protocol │
├─────────────────────────────────────────────────────────┤
│  Layer 2 — Zanpakuto   │ NLP tokenization, query intent │
│                         │ parsing, domain classification │
├─────────────────────────────────────────────────────────┤
│  Layer 1 — Asauchi     │ Formula registry, entity       │
│                         │ registry, ephemeris, charts    │
├─────────────────────────────────────────────────────────┤
│  Layer 0 — Primitive   │ NAND gates, descent engine,    │
│                         │ gyro router                    │
└─────────────────────────────────────────────────────────┘
```

**Pipeline:** `query` → `zanpakuto_nlp` → `descent_engine` → `shikai_process` → `bankai_solve`

### Subsystems

| Module | Purpose |
|--------|---------|
| `primitive` | Boolean NAND gates (discrete + continuous f64), adders, NAND DAG evaluator |
| `descent` | 7-layer descent engine with provenance chains |
| `gyro` | Spinning-wheel router: state tracking, dynamics, token processing |
| `astrology` | 7-axis classification (signs, elements, modalities, rulers, houses, aspects, vedic) |
| `wheel` | 9-graha Vedic graph with shortest-path traversal |
| `formula` | Formula registry (TOML), search with synonyms, glyph rendering |
| `entity` | Entity registry (TOML), seed entities, dynamic generation |
| `chart` | Vedic birth chart (lagna + 12 bhavas), personality derivation |
| `ephemeris` | Julian day, VSOP87 positions, Lahiri ayanamsa |
| `tanto` | Math expression parser/evaluator, NL expression evaluation |
| `bankai` | Verifier, diagnostics, feedback protocol, confidence scoring |
| `validation` | 5-gate validation pipeline with pachinko ball mechanics |
| `pachinko` | Pachinko validation metaphor: balls, pins, pockets |
| `mcp` | MCP JSON-RPC server (feature-gated) |

## Design Principles

- **Determinism-first:** Every computation is pure. Identical inputs always produce identical outputs.
- **NAND-to-bankai:** All logic derives from NAND gates (the Sheffer stroke).
- **Formulas, not facts:** Encode relationships, not static lookups.
- **Cross-domain by default:** New formulas reference ≥2 grahas.
- **No ML in core:** The classifier, router, and validator are deterministic.
- **Embedded corpus:** All seed data compiled into the binary via `build.rs`.

## Quick Start

### Prerequisites

- Rust stable toolchain (see `rust-toolchain.toml`)
- `cargo-deny` and `cargo-audit` (for CI checks)

### Build

```bash
# Native build (default: no features)
cargo build --release

# With MCP server
cargo build --release --features mcp

# Cross-compile for musl (static binary)
cargo build --release --target x86_64-unknown-linux-musl --no-default-features
```

### Run

```bash
# Basic commands
./target/release/laverna ping
./target/release/laverna info
./target/release/laverna entities

# Query
./target/release/laverna query "energy efficiency in code"
```

### Test

```bash
# Dev cycle (fmt + clippy + test)
cargo clippy -- -D warnings && cargo test --lib && cargo fmt -- --check

# Full test suite
cargo test

# With features
cargo test --features llm --lib
cargo test --features mcp
```

## Feature Flags

| Feature | Enables | Default |
|---------|---------|---------|
| `mcp` | rmcp + tokio JSON-RPC server | no |
| `websearch` | ureq (World Bank stats) | via `mcp` |
| `budget` | Token budget tracking | no |
| `bench` | criterion harness | no |
| `llm` | llama-gguf local LLM backend | no |

## CI/CD

### On every push/PR

```
fmt → clippy → cargo deny → cargo test → cargo audit
```

### On tag push (`v*`)

1. Full CI gate passes
2. Cross-compile 4 targets:
   - `x86_64-unknown-linux-musl`
   - `x86_64-unknown-linux-gnu`
   - `aarch64-unknown-linux-musl`
   - `aarch64-unknown-linux-gnu`
3. GitHub Release with binaries + SHA256 checksums

### Cut a release

```bash
git tag v0.1.0
git push origin v0.1.0
```

## License

MIT — see [LICENSE](LICENSE)
