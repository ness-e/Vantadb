# Test Suite

> Movido desde `.opencode/AGENTS.md` — referencia on-demand. Consultar para saber qué tests correr y cómo. Si editas, actualiza también el puntero en AGENTS.md.

```bash
:: Fast Gate (audit profile)
cargo nextest run --profile audit --workspace --build-jobs 2

:: Single test (adapt to use nextest or cargo test as needed)
cargo nextest run --profile audit -p vantadb --test <test_name>

:: Tests that require specific features:
cargo nextest run --profile audit --features "failpoints" --test chaos_integrity
cargo nextest run --profile audit --features "cli" --test cli_tests
cargo nextest run --profile audit --features "arrow" --test columnar

:: Experimental tests (parser, executor, governor) — NOTE: experimental-lisp and experimental-governance deleted Jul 2026

:: Fuzzing (requires nightly + Linux, in fuzz/ dir excluded from workspace)
cd fuzz && cargo +nightly fuzz run fuzz_parser -- -max_total_time=300
```

Test categories: `tests/core/`, `tests/storage/`, `tests/logic/`, `tests/api/`, `tests/certification/`, `tests/memory/`, plus root-level integration tests.
