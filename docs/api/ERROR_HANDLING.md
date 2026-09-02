---
title: VantaDB Error Handling Reference
type: api
status: active
tags: [vantadb, api, errors]
last_reviewed: 2026-09-02
aliases: [VantaError, ERROR_CODES, McpError]
---

# VantaDB Error Handling Reference

This is the **canonical contract** for how VantaDB surfaces errors across every
binding (Rust core, Python, TypeScript/WASM, MCP, HTTP API). If you are writing
client code, integrating with an LLM agent, or adding a new error variant,
**read this first** — error codes are the contract that lets clients `match`
without parsing human-readable messages.

## Design Principles

VantaDB follows the same standardization lesson that
[Vanta's engineering team documented in 2024](https://www.vanta.com/resources/how-we-standardized-error-handling):

> *"We settled on a set of canonical error codes that everyone could use:
> fundamental concepts such as InvalidInputError, NotAuthorizedError,
> ResourceNotFoundError, etc. We focused on keeping the list small so that
> teams would be forced to make tough decisions and decide if the extra work
> for custom errors was really necessary."*
> — Ruyan Chen, Vanta Engineering (Mar 2025)

Applied to VantaDB:

1. **Small canonical code list** — 10 codes (`VANTADB_VALIDATION_ERROR`,
   `VANTADB_NOT_FOUND`, `VANTADB_TIMEOUT`, `VANTADB_BUSY`,
   `VANTADB_RESOURCE_LIMIT`, `VANTADB_CORRUPT`, `VANTADB_INVALID_ARGUMENT`,
   `VANTADB_IO_ERROR`, `VANTADB_WASM_ERROR`, `VANTADB_CLOSED`) instead of
   one-per-variant proliferation.
2. **Codes are stable; messages are not** — clients must `match` on `code`,
   never on `message` text. `Display` strings can change without a major
   version bump.
3. **Retry semantics are explicit** — every error exposes `is_retriable()`
   (Rust) / `.retriable` (Python) / `code` class (TS). Never guess.
4. **Recovery hints shipped with the error** — `recovery_hint()` /
   `.details.hint` so users see actionable guidance, not a generic toast.
 5. **Cause chain preserved** — `#[source]` (Rust) / `cause` (TS 4.4+) /
    `__cause__` (Python 3 `raise … from`) keeps the debug trail intact.
 6. **Backtrace captured, never displayed** (ERR-OBS-01) — `ChainedError`
    variants (`Generic`, `WalError`, `BackendError`, …) capture a
    `std::backtrace::Backtrace` at construction when `RUST_LIB_BACKTRACE=1` /
    `RUST_BACKTRACE=1`; exposed via `Debug` and `backtrace_str()`, never in
    `Display` (cross-language messages stay clean). See
    `docs/operations/OBSERVABILITY.md`.

> **Resolved (ERR-CORE-01, 2026-09-02):** `VantaError::code()` now exists and
> returns `&'static str` with the `VANTADB_` prefix. The table below is the
> **implemented contract** — Rust `code()` emits exactly these strings.

---

## 1. VantaError code table

### 1.1 Canonical codes (10 — contract surface)

Every VantaDB error, regardless of source binding, normalizes to one of these
codes. Clients should branch on `code`; the variant-specific fields are
informational, not contractual for matching. The Rust core emits the prefixed
form (`VANTADB_*`); TS/WASM/Node now emit the same prefixed values on the wire
(ERR-TS-01); Python attaches the same prefixed values as the `.code` attribute
on the MOD-20 exception classes (`NotFoundError.__name__` stays unprefixed -
the *code*, not the class name, is the cross-binding contract) (ERR-PY-01).

| Code | Meaning | Source Rust variant(s) | Retriable |
|------|---------|-------------------------|-----------|
| `VANTADB_VALIDATION_ERROR` | Input failed validation (dimension, schema, duplicate, collision, cycle, conflict, IQL parse) | `DimensionMismatch`, `DuplicateNode`, `NodeIdCollision`, `CycleDetected`, `ExecutionConflict`, `ValidationError`, `InvalidInput`, `IqlParseError`, `UnsupportedOperation`, `NoVectorForKey` | ❌ |
| `VANTADB_NOT_FOUND` | Requested entity does not exist | `NodeNotFound`, `NotFound` | ❌ |
| `VANTADB_TIMEOUT` | Operation exceeded its time budget | `Timeout` | ✅ |
| `VANTADB_BUSY` | Resource locked or not initialized | `DatabaseBusy` (✅), `NotInitialized` (❌) | ✅/❌ per variant |
| `VANTADB_RESOURCE_LIMIT` | Memory, disk, or backpressure limit exceeded; on-disk header field overflow | `ResourceLimit` (✅), `VectorLenOverflow` (❌ hard format limit), `EdgeCountOverflow` (❌ hard format limit) | see §2 |
| `VANTADB_CORRUPT` | Persisted data is corrupt or incompatible format | `WALVersionMismatch`, `IncompatibleFormat`, `SerializationError`, `SchemaError`, `RestoreError`, `BackupError` | ❌ |
| `VANTADB_INVALID_ARGUMENT` | Caller passed a malformed argument (runtime IQL failure) | `IqlError` | ❌ |
| `VANTADB_IO_ERROR` | Filesystem or backend I/O failure | `IoError` (❌), `WalError` (✅), `BackendError` (✅), `CliError`, `SearchError`, `RuntimeError` | see §2 |
| `VANTADB_WASM_ERROR` | Generic catch-all fallback (`Generic`) | `Generic` | ❌ |
| `VANTADB_CLOSED` | Operation attempted on a closed database handle | (lifecycle, not a `VantaError` variant — never returned by `code()`) | ❌ |

> **Final mapping note (ERR-CORE-01):** the previously provisional overlap
> between `VALIDATION_ERROR` and `INVALID_ARGUMENT` on `IqlParseError` is
> resolved: `IqlParseError` → `VANTADB_VALIDATION_ERROR` (matching the TS
> classification), `IqlError` → `VANTADB_INVALID_ARGUMENT`. `code()` is a
> compile-exhaustive match: every new variant must declare its code.

### 1.2 `VANTADB_*` prefixed codes (implemented)

`VantaError::code()` (`src/error.rs`) returns a `&'static str` with the
`VANTADB_` prefix. The TS SDK keeps its readable unprefixed *keys*
(`ERROR_CODES.BUSY`) while the *wire values* are the canonical prefixed
strings (ERR-TS-01); `VANTADB_CLOSED` is emitted by handle-lifecycle checks
(TS `close()` guard), not by `code()`:

```rust
match err.code() {
    "VANTADB_VALIDATION_ERROR" | "VANTADB_INVALID_ARGUMENT" => { /* ... */ }
    "VANTADB_NOT_FOUND" => { /* ... */ }
    c if c.starts_with("VANTADB_") => { /* known family, unhandled */ }
    _ => unreachable!("code() always returns a VANTADB_* string"),
}
```

---

## 2. `is_retriable()` matrix (Rust core)

Defined in `src/error.rs:269`. Use this when implementing retry policies at the
binding boundary.

```rust
impl VantaError {
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            VantaError::DatabaseBusy(_)
                | VantaError::Timeout { .. }
                | VantaError::ResourceLimit(_)
                | VantaError::BackendError(_)
                | VantaError::WalError(_)
        )
    }
}
```

| Rust variant | `is_retriable()` | Recommended backoff |
|--------------|:---------------:|---------------------|
| `DatabaseBusy(_)` | ✅ | exponential, jitter, max 30s |
| `Timeout { .. }` | ✅ | exponential |
| `ResourceLimit(_)` | ✅ | linear, monitor memory |
| `BackendError(_)` | ✅ | exponential, check disk |
| `WalError(_)` | ✅ | exponential, check disk |
| `IoError(_)` | ❌ | not retryable by the core classifier (OS-level failures are typically terminal; bindings may retry selectively) |
| `VectorLenOverflow { .. }` / `EdgeCountOverflow { .. }` | ❌ | hard on-disk format limits — shrink the node, do not retry |
| All other 25 variants | ❌ | do not retry |

---

## 3. `recovery_hint()` guide (Rust core)

Defined in `src/error.rs:281`. Returns a `&'static str` with actionable
guidance for the operator. Bindings should surface this in error details.

```rust
impl VantaError {
    pub fn recovery_hint(&self) -> Option<&'static str> {
        match self {
            VantaError::DatabaseBusy(_) => Some("Wait for the lock to be released and retry"),
            VantaError::Timeout { .. } => Some("Increase the timeout or reduce system load"),
            VantaError::ResourceLimit(_) => Some("Reduce memory pressure or increase configured limits"),
            VantaError::IncompatibleFormat { .. } => Some("Delete the WAL or run dump/restore to migrate formats"),
            VantaError::SchemaError(_) => Some("Reinitialize the database or restore from backup"),
            VantaError::WALVersionMismatch { .. } => Some("The WAL was written by a different version of VantaDB"),
            VantaError::RestoreError(_) => Some("Check that the backup file exists and is readable"),
            VantaError::BackupError(_) => Some("Ensure the backup directory is writable and has free space"),
            VantaError::NodeNotFound(_) => Some("The node may have been deleted or never existed"),
            VantaError::NotFound { .. } => Some("Verify that the namespace or identifier is spelled correctly"),
            _ => None,
        }
    }
}
```

Clients with structured error details should include `hint` in the response
payload so end-users see actionable guidance instead of a generic toast.

---

## 4. TypeScript / WASM error mapping

The TypeScript SDK and WASM binding normalize Rust `VantaError` to a
`VantaError` class with a stable `code` from the 10-code contract. The wire
values are the canonical `VANTADB_*` codes (ERR-TS-01); the TS object keys
keep their unprefixed names.

```ts
export const ERROR_CODES = {
  CLOSED: "VANTADB_CLOSED",
  WASM_ERROR: "VANTADB_WASM_ERROR",
  VALIDATION_ERROR: "VANTADB_VALIDATION_ERROR",
  NOT_FOUND: "VANTADB_NOT_FOUND",
  INVALID_ARGUMENT: "VANTADB_INVALID_ARGUMENT",
  CORRUPT: "VANTADB_CORRUPT",
  RESOURCE_LIMIT: "VANTADB_RESOURCE_LIMIT",
  TIMEOUT: "VANTADB_TIMEOUT",
  BUSY: "VANTADB_BUSY",
  IO_ERROR: "VANTADB_IO_ERROR",
} as const;
```

`vantadb-node` (napi) carries the code as a `"{VANTADB_CODE}: {message}"`
prefix on the thrown `Error` — `wrapNativeError` in `vantadb-ts` parses it
back into `err.code` and strips it from the message, falling back to
`classifyWasmError` for unprefixed (host-level) failures.

### 4.1 `VantaError` shape

```ts
export class VantaError extends Error {
  readonly code: ErrorCode;        // one of the 10 above
  readonly details?: unknown;      // structured payload (Rust variant fields)
  readonly timestamp: Date;

  toJSON(): {
    name: string;
    code: string;
    message: string;
    details?: unknown;
    timestamp: string;            // ISO-8601
  };
}
```

### 4.2 `wrapWasmError` and `classifyWasmError`

The WASM binding preserves the structured `code` via
`to_js_err` (Rust) → `Reflect::set(err, "code", ...)` (TS). When that is
missing (older pkg builds), `classifyWasmError` falls back to message-prefix
regex mirroring the `Display` strings in `src/error.rs`.

```ts
import { wrapWasmError } from "vantadb";

try {
  db.put(...);
} catch (e) {
  const err = wrapWasmError(e, "db.put");
  if (err.code === "VANTADB_VALIDATION_ERROR") {
    console.warn("validation failed:", err.details);
  } else if (err.code === "VANTADB_BUSY") {
    await sleep(100); // is_retriable equivalent
    retry();
  }
}
```

### 4.3 Cause chain (TS 4.4+)

Since ERR-TS-01, `wrapWasmError` / `wrapNativeError` set
`VantaError.cause` to the original thrown value (ES2022 `ErrorOptions`),
preserving the error chain natively. The legacy `details.original` /
`details.{name,stack}` fields are kept unchanged for backward compat.

---

## 5. Python exception hierarchy (10 subclasses)

The Python binding exposes a `VantaError(RuntimeError)` base and 10 typed
subclasses (`MOD-20`). All inherit from `RuntimeError` for backward compat.

```
VantaError (RuntimeError)
├── NotFoundError         # NodeNotFound, NotFound
├── ValidationError       # DimensionMismatch, DuplicateNode, ValidationError, InvalidInput, …
├── CorruptError          # IncompatibleFormat, WALVersionMismatch, SchemaError, SerializationError, …
├── StorageError          # IoError, WalError, BackendError
├── ConflictError         # ExecutionConflict, NodeIdCollision, CycleDetected
├── UnsupportedError      # UnsupportedOperation
├── ResourceLimitError    # ResourceLimit
├── BusyError             # DatabaseBusy, NotInitialized
├── NoVectorError         # NoVectorForKey
└── TimeoutError          # Timeout
```

### 5.1 Attributes (0.5.0+)

Every `VantaError` subclass exposes:

| Attribute | Type | Meaning |
|-----------|------|---------|
| `.code` | `str` | Canonical code from §1.1, exact prefixed value (e.g. `"VANTADB_VALIDATION_ERROR"`) — ERR-PY-01 |
| `.retriable` | `bool` | Equivalent to `is_retriable()` |
| `.details` | `dict \| None` | Structured fields from the Rust variant — *pending*: not attached yet (ERR-PY-01 shipped `code`/`retriable`/`hint`; variant-field extraction is a follow-up task) |
| `.hint` | `str \| None` | Recovery hint (mirrors `recovery_hint()`) |

### 5.2 `error_to_dict()` (`.to_dict()` deferred)

PyO3 `create_exception!` types cannot carry methods, so the plain-dict shape
is exposed as a module-level helper `vantadb.error_to_dict(exc)` (ERR-PY-01;
`exc.to_dict()` tracked for a follow-up if a method-carrying redesign lands):

```python
import vantadb
try:
    db.put(...)
except vantadb.VantaError as exc:
    log.error("vanta_error", extra=vantadb.error_to_dict(exc))
    # {"name": "NotFoundError", "code": "VANTADB_NOT_FOUND",
    #  "message": "...", "retriable": false, "hint": "..."}
```

---

## 6. MCP JSON-RPC error codes

The VantaDB MCP server (`vantadb-mcp`) speaks JSON-RPC 2.0 over stdio. Every
error response uses the JSON-RPC standard codes (5 factories) plus a Vanta
custom range (`-32001`..`-32099`) for domain-specific errors.

### 6.1 Standard JSON-RPC factories

Defined in `vantadb-mcp/src/error.rs`:

| Code | Constant | Meaning |
|------|----------|---------|
| `-32700` | `parse_error` | Invalid JSON was received by the server |
| `-32600` | `invalid_request` | The JSON sent is not a valid Request object |
| `-32601` | `method_not_found` | The method does not exist or is unavailable |
| `-32602` | `invalid_params` | Invalid method parameter(s) |
| `-32603` | `internal_error` | Internal JSON-RPC error |

### 6.2 Vanta custom `-320xx` codes

Mapped from `VantaError` (Task `ERR-MCP-01`):

| Code | Constant | VantaError variant(s) | Meaning |
|------|----------|------------------------|---------|
| `-32001` | `vanta_busy` | `DatabaseBusy`, `NotInitialized` | Database is busy or not initialized |
| `-32002` | `vanta_corrupt` | `WALVersionMismatch`, `IncompatibleFormat`, `SchemaError`, `SerializationError` | Persisted data is corrupt |
| `-32003` | `vanta_conflict` | `ExecutionConflict`, `NodeIdCollision`, `CycleDetected` | Concurrent modification conflict |
| `-32004` | `vanta_not_found` | `NodeNotFound`, `NotFound` | Requested entity does not exist |
| `-32005` | `vanta_unauthorized` | (auth layer) | Caller not authorized for this operation |
| `-32006` | `vanta_rate_limited` | (rate limit layer) | Too many requests |
| `-32007` | `vanta_resource_limit` | `ResourceLimit` | Resource limit exceeded |
| `-32008` | `vanta_timeout` | `Timeout` | Operation timed out |
| `-32009` | `vanta_validation` | `DimensionMismatch`, `DuplicateNode`, `ValidationError`, `InvalidInput`, `IqlParseError` | Input validation failed |

### 6.3 Response envelope

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "validation failed on 'namespace': must not be empty",
    "data": {
      "code": "VALIDATION_ERROR",
      "retriable": false,
      "hint": "Provide a non-empty namespace identifier",
      "details": { "field": "namespace" }
    }
  }
}
```

The `data.code` field carries the canonical code from §1.1, so LLM agents can
branch on a stable identifier even when the JSON-RPC code is just the
transport layer.

### 6.4 Retry guidance for LLM clients

| JSON-RPC code | Vanta canonical code | Should LLM retry? |
|---------------|----------------------|:-----------------:|
| `-32602` | `VALIDATION_ERROR` | ❌ fix the input |
| `-32001` | `BUSY` | ✅ wait + retry |
| `-32004` | `NOT_FOUND` | ❌ resource missing |
| `-32007` | `RESOURCE_LIMIT` | ⚠️ retry with backoff |
| `-32008` | `TIMEOUT` | ✅ retry (or increase timeout) |
| `-32603` | (unknown internal) | ⚠️ retry once, then escalate |

---

## 7. HTTP API error envelope (cross-reference)

The HTTP API (`docs/api/HTTP_API.md`) returns errors as:

```json
{ "success": false, "error": "<message>", "hint": "<optional>" }
```

with HTTP status codes `400 / 404 / 409 / 422 / 429 / 500`. Mapping from
`VantaError` variant → HTTP status is defined in `src/server/errors.rs`
(`vanta_error_status`). Since ERR-CORE-01 both error envelopes
(`vanta_error_response`, `query_error_response`) include a canonical
`"code"` field alongside the message — e.g.
`{ "success": false, "error": "...", "code": "VANTADB_NOT_FOUND" }`. See
`docs/api/HTTP_API.md` § Error responses.

---

## 8. See also

- [`docs/api/EMBEDDED_SDK.md`](EMBEDDED_SDK.md) — Rust `VantaError` reference
- [`docs/api/PYTHON_SDK.md`](PYTHON_SDK.md) — Python `VantaError` subclasses
- [`docs/api/TS_SDK.md`](TS_SDK.md) — TypeScript `VantaError` + `ERROR_CODES`
- [`docs/api/MCP.md`](MCP.md) — MCP JSON-RPC codes + Vanta `-320xx` table
- [`docs/api/HTTP_API.md`](HTTP_API.md) — HTTP error envelope
- `src/error.rs` — canonical Rust definition
- `vantadb-ts/src/errors.ts` — TypeScript canonical
- `vantadb-mcp/src/error.rs` — MCP canonical
- Plan `docs/plans/2026-09-02-error-observability-excellence.md` — task lineage

## Changelog

- **2026-09-02 (ERR-TS-01)** — TS/WASM/Node aligned to the canonical
  `VANTADB_*` wire values: `vantadb-wasm`'s `to_js_err` now calls
  `VantaError::code()` directly (local 30→8 table removed);
  `vantadb-node`'s `map_err` emits `"{CODE}: {Display}"`;
  `vantadb-ts`'s `ERROR_CODES` keeps unprefixed keys but prefixed values
  (BREAKING for `err.code === "VALIDATION_ERROR"`-style comparisons);
  `wrapNativeError` recovers the code (no invented `NATIVE_ERROR`);
  `guards.validateVector` throws `VantaError(VANTADB_VALIDATION_ERROR)`
  instead of `TypeError`/`RangeError` (BREAKING for TypeError catchers);
  `cause` chain set by both wrappers (§4.3).
- **2026-09-02 (ERR-CORE-01)** — `VantaError::code()` implemented in
  `src/error.rs`: 10 canonical `VANTADB_*` codes, exhaustive match, snapshot
  test `error::tests::code_snapshot_all_variants`. Final mapping resolved
  (`IqlParseError` → `VANTADB_VALIDATION_ERROR`). Typed overflow variants
  `VectorLenOverflow` / `EdgeCountOverflow` replace the `ResourceLimit(format!)`
  catch-alls in `write_node_to_vstore`. HTTP envelopes gain the `code` field.
  §2 corrected: `IoError` is **not** retriable per the code (was a doc error).
- **2026-09-02** — Created as part of `ERR-DOCS-01`. Provisional 10-code
  table pending `pub fn code()` from `ERR-CORE-01`.