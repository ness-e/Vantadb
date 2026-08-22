# Auditoría — Raíz pública (README ×2 + governance files)

> **Task:** GOV-F1 (auditoría segunda ola — batch público) · **Fecha:** 2026-08-22
> **Alcance:** `/README.md`, `/README_ES.md`, `/CONTRIBUTING.md`, `/SECURITY.md`, `/SUPPORT.md`, `/CLA_INDIVIDUAL.md`, `/CLA_CORPORATE.md`
> **Método:** verificación mecánica contra código (`src/`, `Cargo.toml`, `pyproject.toml`, workflows) + registries + Test-Path/fetch de cada link citado (método AUD-007). Contexto: release 0.5.0 verificado live (GOV-A5), wheels ARM64 ausentes (MKT-18h), adapters no publicados (MKT-18f), URL canónica `github.com/ness-e/Vantadb`.
> **Límite de alcance (pre-mortem del plan):** exactitud técnica y links; no tono ni marketing copy.

## Resumen

| Severidad | Count |
|-----------|-------|
| 🔴 | 2 |
| 🟠 | 5 |
| 🟡 | 3 |
| 🟢 | 2 |

**Fixes triviales aplicados inline: 10.** Pendientes de decisión owner: 2.

## Findings

| # | Archivo | Finding | Severidad | Evidencia | Acción |
|---|---------|---------|-----------|-----------|--------|
| 1 | SUPPORT.md | Invite de Discord `discord.gg/vantadb` **inválido** (404 NotFound en API de Discord); el válido es `g8nqB3NtXt` ("VantaDB Community") | 🔴 | `GET discord.com/api/v10/invites/vantadb` → NotFound; `…/g8nqB3NtXt` → guild válida | ✅ FIX: invite actualizado a `g8nqB3NtXt` (el mismo de ambos READMEs) |
| 2 | SUPPORT.md | 3 links de documentación a `vantadb.dev/docs/*` muertos: el dominio no resuelve DNS | 🔴 | `Invoke-WebRequest https://vantadb.dev` → "Host desconocido" | ✅ FIX: reemplazados por rutas del repo (`docs/QUICKSTART.md`, `docs/api/`, `docs/FAQ.md`) |
| 3 | SECURITY.md, CLA_CORPORATE.md, SUPPORT.md | Emails `security@vantadb.dev`, `cla@vantadb.dev`, `enterprise@vantadb.dev` sobre un dominio sin registro DNS → canal de reporte de vulnerabilidades por email probablemente no recibible | 🟠 | Mismo fetch que finding 2 (dominio sin resolver) | ⏳ Ticket owner: registrar dominio o cambiar contacto a GitHub Advisories como canal primario. El fallback GitHub Advisories sí funciona |
| 4 | README.md, README_ES.md | Badge "Security Audit" apuntaba a `ci-rust-10.yml`, workflow que NO ejecuta auditoría de seguridad; el análisis de seguridad real es CodeQL (`sec-codeql-30.yml`) | 🟠 | `rg audit .github/workflows/ci-rust-10.yml` → 0 hits; `sec-codeql-30.yml` existe con github/codeql-action | ✅ FIX: badge apuntado a `sec-codeql-30.yml` en ambos READMEs |
| 5 | SECURITY.md | Tabla "Supported Versions" stale: declaraba `0.4.x ✅ Active` cuando la versión live es **0.5.0** (GOV-A5) y `rust-version`/PyPI lo confirman | 🟠 | `pyproject.toml: version = "0.5.0"`; crates.io/npm live 2026-08-01 (GOV-A5) | ✅ FIX: tabla actualizada (0.5.x activo / 0.4.x security patches / <0.4 not maintained) |
| 6 | README_ES.md | Paridad rota con EN en instalación: nota y snippet usaban `import vantadb_py` cuando el import canónico es `import vantadb` (alias incluido en el wheel publicado) | 🟠 | `pyproject.toml [tool.maturin] include = ["vantadb/__init__.py"]`; `vantadb-python/vantadb/__init__.py` = alias thin-wrapper; README.md:66-68 ya corregido | ✅ FIX: nota y snippet alineados a EN (`import vantadb`) |
| 7 | README_ES.md | Números de benchmark local divergen del artefacto real Y del README_EN: decía 61.5 rec/s, p50 HNSW 3.3 ms, híbrido 12.1 ms; el artefacto real reporta **74.0 rec/s (p50 13.2 ms)**, **2.0 ms**, **3.1 ms**. Además llamaba "commiteado" al artefacto que está gitignored, y citaba outlier BM25 0.009 ms vs real 0.0035 ms | 🟠 | `benchmarks/vanta_benchmark_report.json` (74.005 rec/s, p50 13.17 ms, query_vector p50 2.02 ms, query_hybrid p50 3.11 ms, query_text p50 0.0035 ms); `.gitignore:29 vanta_benchmark_report.json` | ✅ FIX: tabla, outlier y footnote alineados al artefacto real + wording "regenerar localmente / gitignored" igual a EN |
| 8 | CONTRIBUTING.md | Sección CI Integration citaba `heavy-certification-50.yml` como job de fuzzing cargo-fuzz; ese workflow solo corre `fuzz_proptest`. El LibFuzzer (corpus + regression) vive en `fuzz-40.yml` | 🟡 | `rg fuzz heavy-certification-50.yml` → solo "fuzz_proptest"; `fuzz-40.yml` = "FUZZ: LibFuzzer — Corpus + Regression" | ✅ FIX: referencia corregida citando ambos con su rol exacto |
| 9 | README.md | Claim "pre-compiled wheels for Windows, macOS, and Linux" no menciona cobertura de arquitectura: los wheels ARM64 están ausentes estructuralmente (MKT-18h) — usuarios macOS Apple Silicon pueden no tener wheel nativo | 🟡 | MKT-18h verificado en GOV-A5 (registries live 2026-08-01); `rust-toolchain.toml` lista `aarch64-apple-darwin` pero eso no garantiza wheel publicado | ⏳ Ticket: decidir copy (declarar arch coverage o silencio) junto al fix de MKT-18h. No es fix trivial (decisión de marketing) |
| 10 | README.md, README_ES.md | Quick Links tenían comentarios `<!-- SECURITY.md (planned) -->` / `<!-- SUPPORT.md (planned) -->` aunque ambos archivos existen desde hace tiempo | 🟡 | Test-Path SECURITY.md/SUPPORT.md = OK | ✅ FIX: links descomentados y activos en ambos READMEs |
| 11 | SECURITY.md | Referenciaba `src/crypto/` (directorio) pero el módulo es el archivo único `src/crypto.rs` | 🟢 | `Test-Path src/crypto` = False; `src/crypto.rs` existe; feature `encryption = ["dep:aes-gcm", …]` en Cargo.toml:121 | ✅ FIX: path corregido |
| 12 | CLA_CORPORATE.md | Énfasis con asteriscos en línea 74 violaba MD049 (estilo underscore del repo) — error pre-existente detectado por el gate del cierre | 🟢 | markdownlint-cli2 → MD047/MD049 en :74 | ✅ FIX: `_…_` aplicado |

## Claims verificados SIN hallazgo (evidencia positiva)

| Claim | Fuente verificada | Resultado |
|---|---|---|
| Badges/workflows referenciados existen (ci-rust-10, gate-docs-21, heavy-certification-50, ci-examples-12, ci-gate) | Test-Path sobre `.github/workflows/*` | ✅ 5/5 |
| Assets públicos (banner-v3.gif, demo.gif, benchmark-sift1m.svg) | Test-Path `assets/*` | ✅ 3/3 |
| Ejemplos de integración existen (mem0, semantic_kernel, dspy, colab notebook) + workflow smoke `ci-examples-12.yml` | Test-Path | ✅ 4/4 |
| Install scripts (`scripts/install.sh`, `install.ps1`) | Test-Path | ✅ |
| Comandos CLI documentados (`put/list/export/rebuild-index/audit-index --json --deep/repair-text-index`) | `src/cli.rs` enum `Commands` (:35+) flags exactos | ✅ paridad total |
| Server defaults: `VANTADB_HOST`, port 8080, data dir `vantadb_data` | `config.rs:506-514`, `cli_handlers/server.rs:238` | ✅ |
| Seguridad: ConstantTimeEq, rate limit 5 intentos/60s, RBAC admin/writer/reader, TLS rustls feature `tls` | `cli_server.rs:39,200-202,418`; `Cargo.toml:137` | ✅ |
| Badge Rust 1.94.1+ | `Cargo.toml:648 rust-version = "1.94.1"` | ✅ |
| Badge Python 3.11+ | `pyproject.toml requires-python >=3.11` | ✅ |
| Fuzz targets `fuzz_parser` / `fuzz_node_deserialize` | `fuzz/fuzz_targets/*` Test-Path | ✅ |
| Perfiles nextest `audit`/`experimental` citados en CONTRIBUTING | `.config/nextest.toml:69,83` | ✅ |
| `just verify` / `just verify-quick` | `justfile:52,55` | ✅ |
| SIFT-1M claims (2.14x–2.80x) citan BENCHMARKS.md §5 con fecha+hardware+comando | Regla 11 compliant | ✅ |
| Benchmark local cita comando reproducible y aclara artefacto gitignored | README.md:333 | ✅ (EN; ES fixed hoy) |
| Adapters presentados como ejemplos de repo (no paquetes publicados) | README Integrations framing | ✅ consistente con MKT-18f |
| MCP listado bajo "Experimental / not MVP" | Product Boundary table | ✅ consistente con B6 |
| Cross-referencias CLA_INDIVIDUAL ↔ CLA_CORPORATE | lectura | ✅ |

## Paridad README vs README_ES (post-fix)

Estructura, badges, snippets, tablas y links ahora 1:1. Única divergencia intencional restante: idioma y anchor-links internos traducidos. El bloque de benchmarks ES quedó alineado al mismo artefacto fuente que EN.

## Verificación

- `Test-Path` sobre 48 paths citados en los 7 archivos → 48/48 OK tras fixes
- Discord API: invite válido confirmado post-fix
- markdownlint-cli2 sobre archivos tocados + este reporte: ver abajo (gate del cierre)

## Deuda / tickets derivados

1. **Dominio vantadb.dev sin DNS** (finding 3) — decisión owner: registrar o migrar contactos. Afecta SECURITY.md (email), SUPPORT.md (enterprise email), CLA_CORPORATE (cla@).
2. **Wheels ARM64 copy** (finding 9) — coordinar con MKT-18h.
