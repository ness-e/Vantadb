# Error Observability — VantaDB

> Added by ERR-OBS-01 (2026-09-02). Companion to `docs/api/ERROR_HANDLING.md`
> (code contract) — this doc covers **capture and operation**: what an error
> carries, how to get it, how to watch for it.

## 1. The error chain

Every `VantaError` carries a full debugging chain. From the client surface
outward, in stable order:

| Layer | Surface | Contract |
|-------|---------|----------|
| `Display` | `e.to_string()`, HTTP bodies, `PyErr`/`JsValue` messages | Human-readable English message. Cross-language text — **never** parse it, never add telemetry to it. |
| `code()` | `VANTADB_*` one of ten canonical codes | Stable machine contract for branching/retry/alerting (`ERROR_HANDLING.md` §1). |
| `is_retriable()` / `recovery_hint()` | retry policy + operator guidance | See `ERROR_HANDLING.md` §2–§3. |
| `source()` | underlying cause chain (`#[source]`, `ChainedError::source`) | `anyhow` bins print the whole chain via `Debug` (`{:?}` = context + causes). |
| **backtrace** | `ChainedError::backtrace()` / `backtrace_str()` | Captured at construction (ERR-OBS-01). In `Debug` output only — deliberately **not** in `Display`, which would pollute cross-language error strings. |

The backtrace lives on `ChainedError` — the catch-all context carrier
(`Generic`, `WalError`, `BackendError`, `RestoreError`, `BackupError`,
`IqlError`, `CliError`, `SearchError`, `RuntimeError`). Simple variants like
`NodeNotFound` are fully explained by their fields + `code()`, so they carry
no backtrace by design.

## 2. Enabling backtraces

Backtrace capture is env-gated by the standard library (`std::backtrace`,
stable since Rust 1.65 — no nightly needed):

```sh
RUST_BACKTRACE=1    # classic gate
RUST_LIB_BACKTRACE=1 # library-only override (wins over RUST_BACKTRACE)
```

- When enabled, every `ChainedError::msg(..)` / `with_source(..)` holds a
  `Captured` backtrace — visible in `{:?}` and via `backtrace_str()`.
- When disabled, capture is a cheap cached env check returning `None`; there
  is no per-error cost in production default configs.
- Capture happens **in the error constructors only**. Errors are the cold
  path (~µs per capture); the design note and upgrade path are marked
  `// ponytail:` in `src/error.rs`.

## 3. Structured log levels (HTTP server)

`src/server/errors.rs` logs every `VantaError` crossing the HTTP boundary
with stable, bounded fields:

```
WARN  error.code=VANTADB_NOT_FOUND   error.retriable=false error.hint="…"  vanta request failed
ERROR error.code=VANTADB_IO_ERROR    error.retriable=true  error.hint=""   vanta request failed
```

| Status class | Level | Meaning | On-call action |
|--------------|-------|---------|----------------|
| 4xx | `WARN` | client mistake | watch trends, don't page |
| 5xx | `ERROR` | server-side invariant broke | investigate |

Panicked query tasks map through `panic_error_response` → `ERROR` + sanitized
generic 500 body (AUDREP-32: panic detail stays server-side).

Fields never carry message text, node IDs, or user content — only the ten
canonical codes — so pipelines and future metric labels stay low-cardinality.
Telemetry init: `src/server/telemetry.rs` (`init_telemetry`, JSON/fmt/OTEL).

## 4. Error-rate metric — TODO(FIND-53)

Planned (from the ERR-OBS-01 contract): one counter incremented at the two
HTTP error-envelope builders, `code` as the only label (cardinality = 10):

```rust
metrics::counter!("vantadb_errors_total", "code" => e.code());
```

**Not wired yet:** the external `metrics` crate is not a dependency of the
root `vantadb` crate (only the bespoke in-tree `src/metrics/` registry
exists), and ERR-OBS-01's constraint forbids adding a dep without its own
justification. Tracked as **FIND-53** in `docs/Backlog.md`; until it lands,
derive error rates from the structured `ERROR`/`WARN` log lines in §3.

## 5. Alerting example

Symptom-based, per the plan's rollout gate (§ Error-handling excellence):

```yaml
# page when server-side failures spike over the 7-day baseline
- alert: VantaServerErrorRateSpike
  expr: |
    sum(rate(vantadb_errors_total{code=~".+"}[5m])) > bool 2
    *
    (sum(rate(vantadb_errors_total{code=~".+"}[7d])) / 2016)
  for: 5m
  labels: { severity: page }
  annotations:
    summary: "5xx error rate > 2× baseline"
    runbook: docs/operations/OBSERVABILITY.md
```

Before FIND-53 lands, the equivalent log-pipeline query is the count of
level=ERROR lines with `error.code` present. Alert on the 5xx class (or the
total rate); never alert per individual code — the codes are dashboard
slices, not page conditions.

## 6. Panic boundaries (catch_unwind evidence)

Verified 2026-09-02 (ERR-OBS-01): every FFI/process boundary already traps
panics — no new `catch_unwind` wrappers were needed.

| Boundary | Mechanism | Evidence |
|----------|-----------|----------|
| **Python (PyO3 0.29)** | every `#[pyfunction]`/`#[pymethods]` call is wrapped in `std::panic::catch_unwind` and re-raised as `pyo3_runtime.PanicException` (derives `BaseException`) | `pyo3-0.29.0/src/impl_/trampoline.rs:301` (`PanicTrap` + `catch_unwind`), `pyo3-0.29.0/src/panic.rs` |
| **WASM (wasm-bindgen)** | a Rust panic becomes a wasm trap — surfacing in JS as a catchable `RuntimeError` — with the real message printed via `console_error_panic_hook` | `vantadb-wasm/src/lib.rs:1901` (`console_error_panic_hook::set_once()`); JS callers `try/catch` the trap |
| **HTTP server** | query tasks run in `tokio::spawn`; panics are caught by tokio's task boundary (`JoinError`) and mapped to the sanitized `panic_error_response` | `src/server/handlers.rs:129,192,678` |
| **CLI / server bins** | `anyhow::Result` main + `.context()` prints the full chain (context + sources) on stderr | `src/bin/vanta-cli.rs:26`, `vantadb-server/src/main.rs:31` |

`VantaError` → binding conversions (`map_vanta_error`, `to_js_err`) route
*expected* failures as typed errors/exceptions; the rows above cover the
*unexpected* (panic) path only.

## 7. Verifying your instrumentation

- Force a 4xx (bad IQL) and a 5xx (e.g. closed DB) against the server →
  confirm one `WARN` / one `ERROR` line with `error.code=` present
  (`cargo test -p vantadb --lib --features server server::errors` covers the
  envelope + level-mapping logic).
- `RUST_LIB_BACKTRACE=1 cargo test -p vantadb --lib error::tests` → the
  backtrace test exercises the `Some` branch.
