# VantaDB — Cómo Leer Failures de Nextest

> **Cómo usar:** Cuando un test falla en nextest, consulta esta guía para interpretar el output antes de diagnosticar.
> **Cómo editar:** Si encuentras un patrón de output nuevo, agrega un ejemplo con explicación.
> **Referencia desde:** `.opencode/AGENTS.md` — sección Nextest Output Reference.
> **Fuente oficial:** https://nexte.st/docs/reporting/

---

## Anatomía del Output

```
$ cargo nextest run --profile audit -p vantadb --test wal_tests

    Starting   3 tests across 1 binaries (4.2s build)
        PASS [   0.012s] vantadb::wal_tests test_write_basic
        PASS [   0.008s] vantadb::wal_tests test_recover_empty
        PASS [   0.103s] vantadb::wal_tests test_recover_corrupt
    ╰─▶ 3 passed, 0 failed, 0 skipped, 0 slow, 0 leaky, 0 retried

     Summary [   0.128s] 3 tests completed: 3 passed, 0 failed
```

### Columnas

| Parte | Significado |
|-------|-------------|
| `PASS` / `FAIL` | Estado del test |
| `[   0.012s]` | Tiempo wall-clock |
| `vantadb::wal_tests` | Crate `::` binary name |
| `test_write_basic` | Nombre del test function |

### Estados de Test

| Estado | Significado |
|--------|-------------|
| **PASS** | Test pasó |
| **FAIL** | Test falló (revisar output abajo) |
| **SKIP** | Test ignorado (vía `#[ignore]` o filtro) |
| **SLOW** | Test tomó >60s (configurable en `.config/nextest.toml`) |
| **LEAK** | Test pasó pero dejó handles abiertos (hijos con stdout/stderr heredado) |
| **RETRY** | Test falló en primer intento pero pasó en reintento (flaky) |

---

## Perfiles en VantaDB

| Perfil | Cuándo usar | Config |
|--------|-------------|--------|
| `audit` | Pre-commit, verify, CI | `fail-fast = false`, `failure-output = immediate-final`, `status-level = slow`, `slow-timeout = 60s × 3` |
| `ci-windows` | CI en Windows (hereda audit) | `test-threads = 2` (evita page file overflow) |
| `experimental` | Tests experimentales | Filtro: integration, parser, executor, governor, columnar, mcp, structured, graph |
| `chaos` | Tests de caos (failpoints) | `test-threads = 1`, filtro: `chaos_integrity_failpoints` |
| `default` | Iteración local rápida | Filtra: excluye ~50 bins pesados. Ver `.config/nextest.toml` |

### Cómo correr cada perfil

```bash
cargo nextest run --profile audit -p vantadb       # pre-commit
cargo nextest run --profile audit --workspace       # verify completo (lento)
cargo nextest run --profile ci-windows              # CI Windows
cargo nextest run --profile experimental            # experimental
cargo nextest run --profile chaos --features failpoints  # chaos
cargo nextest run                                   # default (rápido)
```

---

## Leyendo un FAIL

```
        FAIL [  15.342s] vantadb::storage_tests test_compact_bfs_empty

--- STDOUT:              vantadb::storage_tests test_compact_bfs_empty ---
running 1 test
test test_compact_bfs_empty ... ok

--- STDERR:              vantadb::storage_tests test_compact_bfs_empty ---
thread 'test_compact_bfs_empty' panicked at src/storage/archive.rs:42:9:
assertion `left == right` failed
  left: 0
  right: 100
stack backtrace:
   0: std::backtrace::Backtrace::create
   1: core::panicking::panic
   2: vantadb::storage::archive::compact
   ...
```

### Qué mirar primero

1. **`STDERR` section** → mensaje de pánico con archivo:línea y aserción fallida
2. **`STDOUT` section** — normalmente vacío. Si tiene output, son los `println!` del test
3. **Tiempo** — `[15.342s]`: ¿es SLOW? (el perfil audit marca SLOW a los 60s)
4. **Stack trace** — el primer frame después del panic es el origen

---

## SLOW Timeout

```
  SLOW [  61.203s] vantadb::hnsw_tests test_build_large_index
terminated after 1 periods

--- STDERR: vantadb::hnsw_tests test_build_large_index ---
Test did not complete within timeout period of 60s
```

**Interpretación:**
- El test excedió 60s (period configurado en `.config/nextest.toml`)
- `terminate-after = 3` significa que nextest lo mata tras 3 periodos (3 min)
- Si es esperado: ajustar slow-timeout vía per-test override
- Si no es esperado: el test tiene un bug (loop infinito, wait muerto)

---

## LEAK

```
  PASS [   0.023s] vantadb::mcp_tests test_mcp_connection
  LEAK [   0.045s] vantadb::mcp_tests test_mcp_subprocess

--- LEAK: vantadb::mcp_tests test_mcp_subprocess ---
Test did not close stdout within 500ms of exiting
```

**Interpretación:** El test lanzó un proceso hijo (MCP server) que heredó los pipes. Al salir el test, el hijo siguió corriendo con stdout abierto. Nextest esperó 500ms (`leak-timeout`) y lo marcó como LEAK.

**Qué hacer:** Asegurar que los procesos hijos se maten en cleanup o que no hereden handles.

---

## Flaky Test (RETRY → PASS)

```
  RETRY [   1.234s] vantadb::concurrency_tests test_parallel_writes
  PASS  [   0.987s] vantadb::concurrency_tests test_parallel_writes
```

**Interpretación:** El test falló en el primer intento y pasó en el retry. Señal de race condition o dependencia de timing. Los flaky tests deben investigarse, no ignorarse.

---

## Build Failure vs Test Failure

Nextest separa la compilación de la ejecución:

```
    Starting   3 tests across 1 binaries (4.2s build)
                                         ^^^^^^^^
```

Si la compilación falla, nextest muestra error de compilación antes de cualquier test. Busca la causa en el compilador, no en el test runner.

```bash
# Ver solo el error de compilación (sin correr tests)
cargo check -p vantadb
```

---

## Filtrado de Tests

```bash
# Por nombre exacto
cargo nextest run -p vantadb --test storage_tests -- test_compact_bfs_empty

# Por patrón
cargo nextest run -p vantadb --test storage_tests -- test_compact_

# Por binary type
cargo nextest run --test wal_tests          # solo integration tests (tests/)
cargo nextest run --lib -p vantadb          # solo unit tests (src/)
cargo nextest run --bin vantadb-cli         # solo doc tests del binary

# Expresión filterset (poderoso)
cargo nextest run -E 'test(compact)'
cargo nextest run -E 'binary(storage) & test(wal)'
```

---

## Recomendaciones

| Situación | Comando |
|-----------|---------|
| Ver solo fails | `cargo nextest run --profile audit -p vantadb 2>&1 \| Select-String -Pattern "FAIL\|error\|panicked"` |
| Test que cuelga | Presionar `t` en terminal interactiva para ver output en vivo |
| Test que falla en CI no local | Usar `--profile ci-windows` para reproducir condiciones de CI |
| Debug con logs | `RUST_LOG=debug cargo nextest run --no-capture -p vantadb --test <name>` |
| Reproducir sin nextest | `cargo test -p vantadb --test <name> -- <name> --nocapture` |

## Referencia rápida de options

```bash
cargo nextest run --help  # ver todos los flags
```
