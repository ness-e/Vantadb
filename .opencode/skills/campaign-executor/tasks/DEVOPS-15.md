# DEVOPS-15 — Optimizar default features de Cargo.toml

## Tipo
CI/CD / DevOps → vanta-lead (yo mismo)

## Discovery
- `Cargo.toml:89`: `default = ["cli", "arrow", "fjall", "sysinfo", "memmap2", "fs2", "prometheus", "rayon", "advanced-tokenizer"]`
- 9 features en default, muchas innecesarias para el core library
- `cli` feature arrastra `indicatif`, `console`, `clap`, `clap_complete` — innecesario para usuarios de library
- `prometheus` feature arrastra `prometheus` crate — solo necesario si usan métricas
- `sysinfo`, `memmap2`, `fs2` son para server/deploy, no core
- Server features (`server`, `tls`, `async-ingestion`, `remote-inference`) ya están separadas como opcionales
- `rayon` para paralelismo opcional — útil pero no debería ser default

## Objetivo
Reducir default features del core a solo lo esencial: `["arrow", "fjall", "advanced-tokenizer"]`

## Pasos atómicos
1. Leer dependencias de cada feature no-esencial (cli, sysinfo, memmap2, fs2, prometheus, rayon)
2. Modificar default features en Cargo.toml
3. Verificar que `cargo check -p vantadb` compile sin las features removidas
4. Verificar que `cargo check --no-default-features` compile
5. Verificar que los tests sigan pasando con `cargo nextest run --profile audit`
6. Verificar que los workflows CI que usan features específicas no se rompan

## Verification
- `cargo check -p vantadb` ✅
- `cargo check --no-default-features` ✅
- `cargo nextest run --profile audit` ✅
- CI workflows que referencian features → revisar

## Estado
PENDING
