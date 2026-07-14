# Laverna — Agent Instructions

Deterministic Vedic reasoning engine. 4-layer pipeline:
**Primitive** → **Asauchi** → **Zanpakuto** → **Bankai**.
NAND gates at the bottom. Pure functions only. No ML in core.

## Environment

- aarch64 proot Debian on Android
- Check disk before building: `df -h / | tail -1`
- `CARGO_BUILD_JOBS` is NOT hardcoded — set per-invocation
- `/sdcard` is vfat FUSE: no symlinks, no exec bits, use `cp`
- Rust stable (see `rust-toolchain.toml`): `rustfmt`, `clippy`, `llvm-tools`

## Dev cycle

```bash
cargo clippy -- -D warnings && cargo test --lib && cargo fmt -- --check
```

## CI gate order (4 separate GitHub Actions jobs)

```
Job 1: Check    → fmt → clippy (default) → clippy (--features llm) → deny → test → swiss_oracle → mcp_parity → test (mcp) → test (llm)
Job 2: Audit    → cargo audit
Job 3: Cross    → musl x86_64 + aarch64 builds → verify static link (no NEEDED)
Job 4: Corpus   → build → run `ping` → `info` → `entities` from /tmp (must find 214)
```

Branch protection requires all 4 job names to pass.

## Build

```bash
cargo build --release                                                    # native
cargo build --release --target x86_64-unknown-linux-musl --no-default-features  # slim cross
```

### Feature flags

| feature | enables | default |
|---------|---------|---------|
| `mcp` | rmcp + tokio JSON-RPC server | no |
| `websearch` | ureq (World Bank stats) | via `mcp` |
| `budget` | token budget tracking | no |
| `bench` | criterion harness | no |
| `llm` | llama-gguf local LLM backend | no |

Seed corpus is **always embedded** by `build.rs` — no feature gate.
Binary is self-contained from any CWD.

### Release profile

`lto = "fat"`, `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"`.
There is also a `ci` profile that inherits from `release`.

## Architecture

| Layer | Modules | Role |
|-------|---------|------|
| 0 Primitive | `primitive/`, `descent/`, `gyro/` | NAND gates, 7-layer descent engine, wheel router |
| 1 Asauchi | `asauchi/`, `formula/`, `entity/`, `ephemeris/`, `chart/` | Registries, ephemeris, birth charts |
| 2 Zanpakuto | `zanpakuto/`, `shikai/` | NLP tokenization, query intent, domain classification |
| 3 Bankai | `bankai/`, `mcp/` | Verifier, diagnostics, MCP server |

Pipeline: `query` → `zanpakuto_nlp` → `descent_engine` → `shikai_process` → `bankai_solve`

Entry: `src/cli/mod.rs` (clap). Library: `src/lib.rs`.

## Naming (GNU/UNIX Pure Function Style)

All functions **pure** — no side effects, deterministic, all inputs as params.

- Functions: `snake_case`, **verb-first** (`compute_*`, `evaluate_*`, `validate_*`)
- **No abbreviations**: `accumulator` not `acc`, `left_operand` not `lhs`
- Module prefix when disambiguating: `nand_gate()`, `nand_not()`
- Bool predicates: `is_*`, `has_*`, `can_*`
- Types: `PascalCase`, suffix by role (`*Registry`, `*Engine`, `*Gate`, `*Result`)
- Constants: `SCREAMING_SNAKE_CASE`

## Conventions

- **Errors**: `anyhow` at call sites, `thiserror` for library types
- **Commits**: Conventional Commits — `feat(wheel):`, `fix(bankai):`
- **Formulas, not facts**: encode relationships, not static lookups
- **Cross-domain**: new formulas reference ≥2 grahas
- **Doc comments**: `/// Pure function:` prefix on every public fn
- **Section headers**: `// ─── Section Name ───` (Unicode box-drawing)
- **Testing**: determinism assertions (run 3×, compare bit-for-bit), float tolerance `1e-12`, no mocking
- **Deps**: only crates.io, `cargo deny` enforced, no git dependencies

## Tests

```bash
cargo test                                              # all unit + integration
cargo test --test swiss_oracle                          # Swiss Ephemeris regression (0.05° tolerance)
cargo test --features mcp --test mcp_parity             # CLI/MCP byte-identical parity
cargo test --features llm --lib                         # LLM feature tests
```

539 unit tests, 11 integration tests. Integration tests in `tests/`.
`energy_efficiency_integration.rs` exercises all 14 subsystems.

## Release

Push a tag to trigger the release workflow (CI gate → 4 cross-compiled binaries → GitHub Release):

```bash
git tag v0.1.0 && git push origin v0.1.0
```

Binaries: `laverna-{version}-{target}` + `SHA256SUMS.txt`.
Targets: x86_64/aarch64 × gnu/musl.
