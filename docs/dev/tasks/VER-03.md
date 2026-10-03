---
title: "VER-03: Redacción-on-write persistida + namespaces cifrados"
kind: task
description: "El write-back del proxy persiste la versión redactada (mismos kinds del Redactor) y el original pre-redacción sobrevive solo en un envelope AEAD por namespace (HKDF + AES-256-GCM reusando la primitiva de crypto.rs), con rotación v1 por versión de clave y degradación explícita sin key."
---

# VER-03: Redacción-on-write persistida + namespaces cifrados

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 39, Fase F4 — wave F4.2)
- **Fuente:** Backlog:925 (P52) + plan Task 39
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🔴
- **Tipo:** Rust (vanta-proxy + core crypto) (+ Docs)
- **Turns estimados:** 8-10
- **Creado:** 2026-09-29
- **Estado:** ✅ implementación + verificación + **review-fix batch vanta-audit (APPROVE) aplicado** — cierre (commit) = LEAD
- **Incógnitas (uphill):** 0 abiertas — path write-back verificado; primitiva AEAD verificada contra docs oficiales de `ring` (API 0.17.8; lockfile 0.17.14, F-08) y `aes-gcm` ya en uso
- **Pendientes (downhill):** 0 steps propios. Cierre LEAD: commit local + FINDs (F-05 `mem:create-skill` sin redacción · F-06 zeroize · F-07 rotación de master/recovery CLI). Externos: `check-docs`/`gen-index` repo-wide (VER-06/consolidación) y `public_api` snapshot (VER-02)
- **Branch:** develop · **Commit:** (LEAD)
- **Co-batch:** wave F4.2 (VER-06 ‖ VER-02 ‖ VER-03) — **NO tocar**: `src/cli_handlers/export_md.rs`, `src/cli_handlers/index.rs`, `vanta-memory/src/seed/**` (VER-06); `src/shred/**`, `src/gc.rs`, `src/storage/engine/delete.rs` (VER-02)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-proxy/src/server.rs` (`capture_turn` :620 y tool-loop :766 → `memory_tools::execute`), `vanta-proxy/src/memory_tools.rs` (:95). Ambos call sites ya son el único write path L0 (D47) → un solo choke point nuevo (`capture::turn_job`). |
| Callees | `vanta-proxy/src/redact.rs` (scan/mask existentes), `vanta-proxy/src/envelope.rs` (nuevo), `vantadb::crypto::{Cipher}` (primitiva AEAD existente; `derive_namespace` nuevo), `serde_json`, `tracing`. |
| Implicaciones | El payload de `proxy-turns` gana 2 campos opcionales (`redacted`, `original_envelope`) — aditivo, sin cambio de wire/serialización on-disk ni de `MemoryRecord`. El L1 record (`l1/{session}`, search-facing) pasa a llevar el texto redactado (era el original). `vanta-proxy` activa `vantadb/encryption` (feature existente; sin dependencias nuevas externas). `ProxyConfig` gana `envelope: EnvelopeConfig` → literal struct en tests del proxy necesita el campo (compilación fuerza la actualización). Sin migración de datos. Performance: scan HKDF AES-GCM síncrono por turno capturado (µs-ms, fire-and-forget; no hot path de search). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-proxy/src/capture.rs` (268L), `vanta-proxy/src/redact.rs` (regiones :15-300, :400-407), `vanta-proxy/src/writeback.rs` (:1-120), `vanta-proxy/src/config.rs` (:1-160), `vanta-proxy/src/error.rs` (109L), `vanta-proxy/src/memory_tools.rs` (:55-114), `vanta-proxy/src/server.rs` (:40-170, :240-284, :590-644, :740-789, :860-904), `vanta-proxy/src/lib.rs`, `vanta-proxy/Cargo.toml`, `vanta-proxy/tests/prx07_redact.rs` (318L), `src/crypto.rs` (668L), `src/config.rs` (:215-242, :845-874, :1350-1409), `src/lib.rs` (:40-69), `src/sdk/serialization/vector_types.rs` (:95-178), `src/sdk/search/mod.rs` (:195-314), `vanta-memory/src/core/record/l1_reader.rs` (:1-55), `vanta-memory/src/core/hooks/auto_capture.rs` (198L), `Cargo.toml` (:140-184, :760-809), `docs/dev/tasks/VER-05.md` (formato).
- **Archivos referenciados hacia dentro (imports/dependencias):** `capture.rs` → `writeback::L0Job`, `vanta_memory::{l1_namespace, epoch_ms_to_rfc3339}`, `vantadb::sdk::{Embedded, MemoryInput, ...}`; `redact.rs` → `regex`, `ProxyError`; `crypto.rs` → `aes-gcm`, `ring` (feature `encryption`), `rand`, `sha2`.
- **Archivos que referencian a los editados (referencias entrantes):** `capture::turn_job` ← `server.rs:620` + `memory_tools.rs:95` (2 call sites; tests de ambos módulos); `Redactor` ← `server.rs:430` (egress, intacto) + tests `prx07_redact.rs`; `Cipher` ← `src/storage/vfile.rs:17` (uso at-rest, intacto); `ProxyConfig` ← ~18 literales en tests del proxy (añadir el campo `envelope` es mecánico y lo fuerza el compilador).
- **Veredicto impacto:** 🟡 medio — aditivo en payload JSON + 1 módulo nuevo + 1 método nuevo en crypto + 2 firmas de write path. Nada se rompe si se revierte (el envelope es opt-in; con `[envelope] enabled=false` el payload queda byte-idéntico al actual).

## Contrato

(verbatim del plan Task 39 — ley)

"redacción-on-write persistida: lo que se persiste en store/índices es la versión redactada (mismos kinds del `Redactor`; test: PII sintética capturada → scan de store+índices+export v2 = 0 en claro) Y el original solo sobrevive en un envelope AEAD por namespace (descifrable únicamente con su key; rotación de claves declarada y testeada) Y sin key la degradación es explícita (modo configurado + warning; nunca caída silenciosa a claro) Y doc (config, formato del envelope, rotación, degradación)"

## Spec (SDD — feature-add: símbolos públicos nuevos)

> Gate P/D: el plan ya autorizó la familia (Gate Result ✅ DO; "Ruta: vanta-worker (+ vanta-audit: diseño AEAD/rotación)"); el diseño de abajo queda explícito para la revisión `vanta-audit` del P2-01. Decisiones resueltas por evidencia de código (file:line).

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Dónde aplica la redacción on-write | (A) `capture::turn_job` — único choke point de los 2 call sites (server.rs:620, memory_tools.rs:95) / (B) en cada caller / (C) dentro de `WriteBack::track` | **A** — plan dice "mover el `Redactor` al write-back (`capture.rs`)"; un solo punto no diverge | ✅ decidido-por-evidencia (`rg turn_job` = 2 callers, ambos L0 D47) |
| 2 | Semántica de modo en write | (A) Block/Mask enmascaran; Log observa (kinds) sin mutar — espejo del egress / (B) siempre Mask / (C) Block descarta el registro | **A** — un turno ya completado no se bloquea ni se descarta (pre-mortem F1 del plan: "nada se pierde"); Log = observar, igual que el wire | ✅ decidido-por-evidencia (`RedactMode` :26-34, `apply` :181-196) |
| 3 | Config | (A) reusar `[redact]` (enabled/mode/patterns) + nuevo `[envelope] {enabled, key_version}` opt-in / (B) knob nuevo por-camino (`redact.write_back`) / (C) sin envelope (solo redacción) | **A** — "la redacción que el proxy ya hace en vivo" se persiste cuando está encendida; envelope = feature propia con su sección (patrón `[redact]`/`[cache]`/`[cost]`) | ✅ decidido-por-evidencia (config.rs:44-58; invariante opt-in del proxy) |
| 4 | Dónde vive el envelope | (A) campo dedicado `original_envelope` en el payload JSON de `proxy-turns` ("capa de campos" del stop condition) / (B) tocar `MemoryRecord`/serialización on-disk (breaking) / (C) namespace aparte | **A** — sin tocar el core del record (stop condition del plan: "si el envelope exige tocar el formato del record → capa de campos dedicada") | ✅ decidido-por-evidencia (plan Task 39 Stop conditions; `MemoryInput.payload` es String libre) |
| 5 | Cuándo sellar el original | (A) solo cuando la redacción enmascaró algo (`masked=true`) / (B) siempre que `[envelope] enabled` | **A** — si no hubo findings, original == persistido (sellar duplicaría); el invariante es "el original NO queda en claro", no duplicar | ✅ decidido-por-evidencia (pre-mortem F1: "el original en envelope permite recuperación", solo relevante si difiere) |
| 6 | Key del envelope | (A) master = `VANTADB_ENCRYPTION_KEY` (hex 32B) + derivación HKDF-SHA256 por namespace / (B) key nueva por namespace almacenada/rotada a mano / (C) KMS/HSM | **A** — reusa la primitiva AEAD y el env canónico del proyecto; domain separation vía HKDF (salt fijo + info=ns‖0x00‖versión) evita cruce con el uso at-rest de vfile | ✅ decidido-por-evidencia (crypto.rs:15-16/:104-108/:137; KMS = rabbit hole prohibido por el plan) |
| 7 | Rotación v1 | (A) `key_version` en config + campo `k` en el envelope; nuevos writes usan la versión configurada; envelopes viejos descifrables (misma master) / (B) re-encrypt masivo sincrónico / (C) sin rotación | **A** — stop condition del plan ("rotación v1: nueva key para nuevos writes + re-encrypt lazy"); rotar la MASTER exige re-encrypt → deuda declarada + FIND | ✅ decidido-por-evidencia (plan Task 39 Stop conditions/Pre-mortem F2) |
| 8 | Degradación sin key | (A) explícita: modo `Disarmed` + `tracing::warn!` al arranque + nunca escribe el original en claro (redacted-only) / (B) fallar el write-back / (C) guardar en claro con warning | **A** — contrato: "modo configurado + warning; nunca caída silenciosa a claro"; el capture nunca rompe el wire (D47) | ✅ decidido-por-evidencia (contrato; invariante D47 capture.rs:1-4) |
| 9 | Trazabilidad de kinds | (A) campo `redacted: [kinds]` sin valores en el payload de `proxy-turns` / (B) nada | **A** — el plan dice: `RedactConfig` clasifica kinds "sin valores — insumo directo para persistir «redactado» manteniendo trazabilidad" | ✅ decidido-por-evidencia (plan Task 39 Verificación real, última línea) |
| 10 | Alcance del registro L1 (search) | (A) `l1/{session}` lleva SOLO el texto redactado, sin envelope (recovery vive en `proxy-turns`) / (B) envelope duplicado en L1 | **A** — el search/inject es el consumidor (métrica "0 PII en store/índices"); una sola copia del ciphertext simplifica el forget de VER-02 | ✅ decidido-por-evidencia (VER-04 nota de coordinación: "redacción en el payload inyectado") |

## Diseño AEAD explícito (input para `vanta-audit` del P2-01)

**Formato del envelope (v1, campo `original_envelope` del payload de `proxy-turns`):**

```json
{ "v": 1, "k": 1, "ns": "proxy-turns", "ct": "<hex>" }
```

- `v` (u8): versión del framing del envelope. Cambios de formato futuro → bump + parseo por versión.
- `k` (u8): versión de clave usada en la derivación. Rotación sin romper lectura.
- `ns` (string): namespace cuyo key selló el payload (informativo + auditabilidad; el binding real es la derivación).
- `ct` (hex): salida de `Cipher::encrypt` = `[nonce 12B ‖ ciphertext ‖ tag 16B]` (AES-256-GCM, nonce aleatorio por mensaje vía CSPRNG).

**Derivación de key por namespace (nueva, `vantadb::crypto::Cipher::derive_namespace`):**

```
key(ns, k) = HKDF-SHA256(IKM = master, salt = "vanta-namespace-envelope-v1", info = "ns" ‖ ns_bytes ‖ [k])
master     = VANTADB_ENCRYPTION_KEY (hex 32B) — la misma primitiva/canal de crypto.rs
```

- Domain separation: salt fijo distinto del uso at-rest de `vfile.rs` + `info` con marcador `"ns"`.
- El namespace y la versión PARTICIPAN de la key: un blob re-etiquetado (`ns` o `k` alterados) no autentica → `DecryptionFailed`.
- Solo masters de 32 bytes raw: un passphrase (path PBKDF2 de `Cipher`) no puede derivar → degradación explícita `Disarmed` (no hay derivación silenciosa).

**Rotación v1 (declarada):** bump de `key_version` → nuevos writes con key derivada nueva; envelopes previos legibles mientras la master esté configurada. **Límite declarado:** rotar la master (cambiar `VANTADB_ENCRYPTION_KEY`) invalida envelopes existentes → re-encrypt lazy no implementado (FIND). No hay `Cipher`-per-envelope almacenado: las keys se re-derivan.

**Matriz de degradación (nunca caída silenciosa a claro):**

| `[envelope]` | key master | `[redact]` | Comportamiento (modo) |
|---|---|---|---|
| off (default) | — | off | Transparente: nada se redacta ni se sella (comportamiento actual, byte-idéntico) |
| off | — | on | Redactado-on-write; el original NO sobrevive (la redacción es la única copia — declarado) |
| on | válida 32B | on | Envelope `Active`: original solo en ciphertext; redactado en store/índices |
| on | ausente/inválida/passphrase | on | `Disarmed` + `tracing::warn!` al arranque; redacted-only (el original NO se escribe en claro) |
| on | válida | off | Sin hallazgos de redacción → sin envelope (documentado: el envelope preserva el original pre-redacción) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) el capture nunca bloquea/rompe el wire (D47) — errores de redacción/envelope degradan, no propagan; (2) PII sintética en claro = 0 en store/índices/export tras captura con `[redact]` on; (3) el original pre-redacción NUNCA se escribe en claro en ninguna superficie (nunca "fallback a claro"); (4) sin envelope activo, el payload de `proxy-turns` queda byte-idéntico al actual (opt-in puro); (5) `MemoryRecord`/serialización on-disk/wire intactos (stop condition); (6) `Redactor` de egress intacto (server.rs:430); (7) `src/crypto.rs` solo gana API aditiva (`derive_namespace`), el uso at-rest de `vfile.rs` intacto.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-proxy --build-jobs 2` (verde, incluye `ver03_*`) · `cargo nextest run --profile audit -p vantadb --build-jobs 2` (crypto sin regresiones) · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `cargo fmt --check` · `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` · `scripts/validate-docs-coverage.ps1`.
- **Deuda pendiente (declarada, no introducida):** (a) re-encrypt lazy tras rotación de la master key (FIND propuesto — requiere barrido masivo); (b) CLI/tool de recuperación (`envelope open`) — la recuperación es programática (`Envelope::open`) en este slice (FIND propuesto); (c) verificación criptográfica formal del envelope por `vanta-audit` (P2-01, el plan lo pide) — diseño en §"Diseño AEAD explícito". El orchestrator registra los FIND-* (Backlog.md fuera de mi scope en esta wave).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** sin deuda neta nueva — aditivo (1 módulo nuevo, 1 método core, campos JSON opcionales) y 0 dependencias externas nuevas (`aes-gcm`/`ring` ya eran deps del workspace; `vanta-proxy` activa la feature existente `vantadb/encryption`). El blanket `#![allow(clippy::expect_used, clippy::unwrap_used)]` de `src/crypto.rs` ya existe y cubre el nuevo método (misma clase de invariantes estáticos). **Pago:** `turn_job` gana documentación de contrato más estricta y el proxy gana tests de degradación que no existían. **Deuda declarada (diferida):** re-encrypt lazy tras rotación de master → FIND; recuperación por CLI → FIND (recuperación programática en este slice).

## Verificación

| Ítem | Comando → resultado esperado |
|------|------------------------------|
| Contrato c1 (redacción-on-write) | `ver03_write_redact::pii_synthetic_absent_from_store_indices_export` → PII sintética (email + AWS key) capturada con `[redact]` mask → `list_turns` + `read_session_records` + `search(text_query=PII)` + `export_line_from_record` → 0 en claro |
| Contrato c2 (envelope roundtrip) | `ver03_write_redact::envelope_recovers_original_and_binds_namespace` → `Envelope::open` recupera el original; blob re-etiquetado (ns/k/ct) no autentica |
| Contrato c2b (rotación) | unit tests `envelope.rs`: `rotation_bump_keeps_old_envelopes_readable` + `key_version_participates_in_derivation` |
| Contrato c3 (degradación explícita) | `ver03_write_redact::disabled_envelope_and_missing_key_never_persist_clear_originals` → modo `Disarmed`/`RedactedOnly`, sin `original_envelope`, sin claro |
| Contrato c4 (doc) | `docs/api/PROXY.md` §Redaction/§Envelope (config, formato, rotación, degradación) + validate-docs-* verdes |
| Gates | `cargo fmt --check` · clippy workspace `-D warnings` · nextest `-p vanta-proxy` + `-p vantadb` full · `validate-docs-coverage.ps1` |

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del plan ✅ cláusula por cláusula (redacción persistida + envelope per-namespace + degradación + doc) + fmt/clippy/nextest scoped verdes |
| **Commit** | LEAD (el worker NO commitea); diff limpio, conventional commit `feat(proxy): VER-03 — ...` |
| **Release** | N/A local (release-plz post-push, lane owner) |

## Herramientas necesarias
- Terminal cargo (nextest scoped `-p vanta-proxy`/`-p vantadb`, clippy, fmt) con `CARGO_BUILD_JOBS=2`
- codegraph_codegraph_explore (blast radius — hecho) · `node scripts/docs/*.mjs` · `scripts/validate-docs-coverage.ps1`
- FIND-173/177 vigentes (prohibiciones de co-batch) — verificado: no se toca ningún path de los listados

**Skills cargadas (SDP v3):** `security-and-hardening` (trust boundary storage/PII + checklist), `source-driven-development` (ring HKDF verificado contra docs.rs/ring/0.17.8), `test-driven-development` + `rust-write-tests` (RED→GREEN, sin env mutation), `doubt-driven-development` (stakes de seguridad — review adversarial delegado al P2-01), `incremental-implementation` (slices), `context-engineering`, `documentation-skill` (docs/api/PROXY.md). Descartada `frontend-ui-engineering` (sin UI). Base fija `campaign-executor`/`progreso` vía MCP.

## Investigation Notes

### Código — path verificado (2026-09-29)

- **Redacción egress existente:** `Redactor::apply` (`redact.rs:181-196`) se aplica al REQUEST pre-forward (`server.rs:430`); `Block`→422 sin forward; `Mask`→`[REDACTED_*]`; `Log`→sin mutar + warn. `scan` :153-176 (AWS key/secret, tokens sk-…/ghp_…, emails, custom regex), `mask_body` :216-229. `RedactConfig` :41-63 (enabled default false, mode default Mask). Findings sin valores por construcción (`RedactionBlocked` kinds-only).
- **Write-back SIN redacción:** `capture::turn_job` (`capture.rs:53-141`) persiste el texto del último mensaje user SIN escanear (`rg redact` en capture.rs = 0 antes de este slice). Dual-write: `proxy-turns` (audit trail, payload JSON `{session, protocol, space, model, text}`) + `l1/{session}` (registro `MemoryRecord` Episodic para recall). `server.rs:256-257` captura el body ORIGINAL (el redactado solo iba al wire).
- **Primitiva AEAD:** `Cipher` (`crypto.rs:137-290`): AES-256-GCM, key 32B raw (`[nonce‖ct‖tag]`) o passphrase (framing PBKDF2). `from_env` :178-184 lee `VANTADB_ENCRYPTION_KEY` vía `Config::default().encryption_key` (config.rs:1380-1386). `EncryptionStream` :335-417 solo lo usa `storage/vfile.rs:17` (at-rest de VantaFile) — no hay envelope por namespace (`rg namespace_key` = 0). Módulo gated: `#[cfg(feature = "encryption")]` (lib.rs:56-57) → `vanta-proxy` debe activar `vantadb/encryption` (feature existente; `ring` ya transitiva vía reqwest/rustls).
- **Precedente de tests:** `prx07_redact.rs` (patrón RED con sintéticos AWS_KEY/emails); `state_with_redact` (literal ProxyConfig completo — añadir `envelope` es mecánico); `capture.rs` tests inline con engine InMemory.

### Internet (fuentes oficiales — verificadas 2026-09-29)

- `ring` HKDF (API verificada contra docs.rs/ring/**0.17.8**; el lockfile del repo trae **0.17.14** — misma API, F-08): `Salt::new(HKDF_SHA256, salt)` + `Salt::extract(secret) -> Prk` + `Prk::expand(&info, len) -> Result<Okm, Unspecified>`; `KeyType` trait local (sin impl para `usize`, se implementa un struct de 32B). — https://docs.rs/ring/0.17.8/ring/hkdf/index.html
- AES-256-GCM por RustCrypto (`aes-gcm` 0.11, ya en el workspace): encrypt infalible con key válida + nonce fresco (invariante documentada en `crypto.rs:193-206`).
- **Web research adicional: no requerida** (no hay APIs externas nuevas; RFC 5869 es el spec citado por ring).

### Confusion management

- Plan decía "mover el `Redactor` al write-back (`capture.rs`) + store" — verificada la ambigüedad de "store": se interpreta como los DOS destinos del turno (`proxy-turns` + `l1/{session}`), ambos alimentados por `turn_job` (un solo punto). Sin ambigüedad pendiente.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — diseño AEAD explícito arriba; APIs verificadas |
| Pendientes de ejecución (downhill) | 5 steps (RED → crypto GREEN → envelope GREEN → wiring → docs/verify) |
| % completado | 10% (DISCOVERY); implementación pendiente |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** (MANDATORIA — storage/PII/crypto) — `security-and-hardening` checklist: (a) threat model: PII en claro persistida (hoy) + degradación silenciosa (hoy no existe el control) → mitigaciones: redacción-on-write, envelope AEAD, degradación explícita; (b) tokens/keys: el envelope NO puede contener la key; `Cipher` nunca loggea material; (c) datos: `ct` en hex solo es ciphertext; el plaintext original no se escribe en claro en ningún path; (d) DoS: seal opera sobre el texto del turno ya acotado (scan cap 2MiB del Redactor); (e) errores: `open` falla tipado sin filtrar material (`DecryptionFailed`); (f) crypto: reuso de primitiva auditada + HKDF domain-separated; `vanta-audit` revisará el diseño en el P2-01. Cierra con checklist + `cargo audit` NO requerido (0 deps nuevas).
- [ ] **PERFORMANCE** — N/A justificado: no toca hot paths de search/ingestión (`engine.rs`, `vector/`, HNSW); scan+encrypt síncrono por turno capturado en fire-and-forget, coste µs-ms sobre un path que ya hacía 2 writes; Regla 9 no aplica sin claim de performance.

## Steps

### Step 1: Task file + DISCOVERY (este archivo)
- **Archivos:** `docs/dev/tasks/VER-03.md`
- **Acción:** contrato verbatim + blast radius + Regla 0 + Spec (10 decisiones) + diseño AEAD explícito (para vanta-audit) + matriz de degradación
- **Verify:** archivo existe con contrato verbatim + §Spec llena + §"Diseño AEAD explícito"
- **Estado:** ✅ DONE

### Step 2: RED — tests que fallan (integración + unit)
- **Archivos:** `vanta-proxy/tests/ver03_write_redact.rs` (nuevo), unit tests inline en `vanta-proxy/src/envelope.rs` (nuevo) y `src/crypto.rs` (derive) — los tests RED primero, contra API inexistente
- **Acción:** tests del contrato c1/c2/c2b/c3 (PII sintética → 0 claro en store+índices+export; envelope roundtrip + binding ns/k/ct; rotación; degradación sin key/passphrase/off)
- **Verify:** `cargo nextest run -p vanta-proxy --test ver03_write_redact` → **E0432 unresolved import `vanta_proxy::envelope`** (RED correcto; el primer intento murió antes por un compile error transitorio de `src/attestation.rs` de VER-02 en el worktree compartido — ajeno a este slice)
- **Estado:** ✅ DONE

### Step 3: GREEN — `Cipher::derive_namespace` + `Redactor::for_write`
- **Archivos:** `src/crypto.rs` (método aditivo + 3 tests), `vanta-proxy/src/redact.rs` (`for_write` + `enabled()` + `WriteRedaction` + 3 tests)
- **Acción:** HKDF-SHA256 por namespace/versión (solo key raw 32B; passphrase → `InvalidKey` explícito); `for_write` reusa `scan`/`mask_body` (Block/Mask enmascaran, Log observa)
- **Verify:** `cargo nextest run --profile audit -p vantadb --lib --features encryption derive_namespace --build-jobs 2` → **3/3 ✅** (el módulo `crypto` está gated tras `encryption` — correr con esa feature)
- **Estado:** ✅ DONE

### Step 4: GREEN — `envelope.rs` + wiring `capture.rs`/config/server
- **Archivos:** `vanta-proxy/src/envelope.rs` (nuevo: `EnvelopeConfig` — patrón de la casa, junto a `RedactConfig` en `redact.rs` —, `Envelope`, `EnvelopeMode`, `EnvelopeBlob`, `seal`/`open`, hex), `vanta-proxy/src/lib.rs`, `vanta-proxy/src/config.rs` (`ProxyConfig.envelope`), `vanta-proxy/src/capture.rs` (`WriteGuard` + `turn_job`), `vanta-proxy/src/server.rs` (AppState.envelope + warn de degradación + call sites + snapshot), `vanta-proxy/src/memory_tools.rs`, `vanta-proxy/src/mem_command.rs` (test seed), `vanta-proxy/Cargo.toml` (`vantadb/encryption`), 17 literales `ProxyConfig` en tests
- **Acción:** sellar original solo si `masked`; payload gana `redacted`/`original_envelope`; L1 con texto redactado; `Disarmed`+warn sin key, nunca claro
- **Verify:** `cargo nextest run --profile audit -p vanta-proxy --build-jobs 2` → **300/300 ✅** (1 skipped) — incluye `ver03` 6/6 + prx07 + capture + tool_loop + mem_command sin regresiones
- **Estado:** ✅ DONE

### Step 5: Docs + verify full + cierre de contrato
- **Archivos:** `docs/api/PROXY.md` (§Redaction-on-write & encrypted namespaces: config, formato, rotación, degradación; tabla de features 8→9; `/snapshot` documenta `envelope`), task file sync
- **Acción:** doc del contrato c4; gates; deuda/FINDs; recitation; NO commit (LEAD) · NO self-review (LEAD)
- **Verify:** `cargo fmt` scoped ✅ (solo archivos propios; `attestation.rs`/`shred/mod.rs` de VER-02 tienen diffs ajenos y no se tocaron) · `clippy -p vanta-proxy --all-targets --all-features -D warnings` ✅ · `clippy -p vantadb --features encryption --lib -D warnings` ✅ · `validate-docs-coverage.ps1` ✅ 0 gaps · `check-links` ✅ (dentro de presupuesto) · `check-docs` ✖ gated SOLO por `docs/dev/tasks/VER-06.md` sin frontmatter (archivo de VER-06, no tocado) · `gen-index --check` stale por los task files del co-batch + cambios del generador de la sesión de consolidación (regenerarlo mezclaría 2000+ líneas ajenas → revertido, lane de docs-consolidation) · `nextest -p vantadb` full ✅ **2533/2533** (excl. `public_api`, fail externo de VER-02: snapshot desactualizado por `pub mod attestation`)
- **Estado:** ✅ DONE (docs + gates scoped)

## Verificación mecánica (resultados)

| Gate | Comando | Resultado |
|------|---------|-----------|
| fmt (propios) | `cargo fmt -p vanta-proxy -- --check` + `rustfmt --edition 2021 --check src/crypto.rs` | ✅ exit 0 |
| clippy proxy | `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` | ✅ exit 0 |
| clippy crypto | `cargo clippy -p vantadb --features encryption --lib -- -D warnings` | ✅ exit 0 |
| tests proxy | `cargo nextest run --profile audit -p vanta-proxy --build-jobs 2` | ✅ **303/303** (1 skip ambient) — post-batch |
| tests VER-03 | `cargo nextest run --profile audit -p vanta-proxy --test ver03_write_redact` | ✅ **7/7** — post-batch (incluye regresión F-01) |
| tests crypto | `cargo nextest run --profile audit -p vantadb --lib --features encryption crypto::tests --build-jobs 2` | ✅ **22/22** — post-batch (superset del scoped `derive_namespace` 3/3; incluye decode multibyte + nonce freshness) |
| docs coverage | `pwsh scripts/validate-docs-coverage.ps1` | ✅ 0 gaps |
| links | `node scripts/docs/check-links.mjs` | ✅ dentro de presupuesto |
| docs schema | `node scripts/docs/check-docs.mjs` | ⚠️ fail externo: `VER-06.md` sin frontmatter (no tocado); mis archivos: 0 gating |
| index | `node scripts/docs/gen-index.mjs --check` | ⚠️ stale cross-sesión (co-batch + generador en vuelo); revertido a HEAD — la sesión de consolidación ya regenera (`docs/index.md`/`llms.txt` modificados por ella) |
| OCR (advisory) | `pwsh dev-tools/ocr-review.ps1 -Format json` + `ocr delegate rule <paths>` | ✅ sin API key → emite Rule Groups; check manual del grupo Security-Sensitive vs diff (hex/UTF-8 tipados, sin log de material, crypto de crates auditados): **0 Critical/High** |
| core full | `cargo nextest run --profile audit -p vantadb --build-jobs 2 --no-fail-fast -E 'not binary(public_api)'` | ✅ **2533/2533** (2 skipped) — excluye `public_api` (fail EXTERNO: snapshot de API pública desactualizado por el `pub mod attestation` de VER-02; 9122 líneas de diff ajenas, ya en lane de VER-02 — `public-api.txt` apareció modificado por su sesión). Corrido PRE-batch; el batch solo tocó `decode_hex` (privado, único caller `from_env:200`) → cubierto por `crypto::tests` 22/22 |

## Review-fix batch (vanta-audit — APPROVE, 0 High; 1M+3L+3I)

| Ítem | Hallazgo | Fix aplicado | Evidencia (comando → resultado) |
|------|----------|--------------|---------------------------------|
| **F-01** (Medium) | `envelope.rs::decode_hex` con slicing `&s[i..i+2]` panica con multibyte UTF-8 en offset par (`"aa€x"`, PoC real exit 101) | `chunks_exact(2)` sobre bytes + `std::str::from_utf8` por chunk → `None` (nunca panic). Hardening del mismo patrón en `src/crypto.rs::decode_hex` (env-fed; degrada a `InvalidKey` en vez de paniquear el arranque) | ver03 **7/7** ✅ (`open_rejects_malformed_envelopes_instead_of_panicking`, caso `"aa€x"` → `Err(Format)`) · crypto **22/22** ✅ (`test_decode_hex_multibyte_degrades_instead_of_panicking`) |
| **F-02** (Low) | `redact.rs::for_write` fallback `unwrap_or_else(\|_\| text.to_string())` devolvía el **texto en claro** con `masked:true` | `String::from_utf8_lossy(...).into_owned()` — fail-closed: nunca re-emite el original (fallback inalcanzable por construcción: matches char-boundary + placeholders ASCII) | proxy **303/303** ✅ + test `for_write_masks_multibyte_text_without_losing_non_pii` |
| **F-03** (Low) | Techo del scan no documentado para persistencia | Doc en `PROXY.md`: cap 2 MiB fail-open → turno persisted **unscanned** y **sin envelope**; + warn runtime en `for_write` al exceder | proxy **303/303** ✅ + test `for_write_oversize_fails_open_unmasked` |
| **F-04** (Low) | Fila de degradación ambigua para `mode="log"` | Nota explícita en `PROXY.md`: `log` observa sin mutar → texto tal cual y **sin envelope** (el envelope solo preserva originales que Mask/Block enmascararon) | docs check ✅ |
| **F-08** (nit) | Cita `ring 0.17.8` vs lockfile **0.17.14** | Nota corregida en §Investigation Notes (misma API) | — |
| **Rec. 3** | Tests baratos faltantes | `test_encrypt_uses_fresh_nonce_per_message` (2× mismo plaintext → nonces distintos) + `Err(Format)` para version≠1 / hex inválido / multibyte en `open` | ver03 **7/7** + crypto **22/22** ✅ |
| **F-05** | `mem:create-skill` sin redacción | **No aplicado** (pre-existente, fuera del contrato VER-03) — queda como FIND propuesto para el LEAD | — |
| **F-06** | Zeroize de material | **No aplicado** (pre-existente) — FIND propuesto para el LEAD | — |
| **F-07** | Rotación de master + recovery CLI | **FIND del LEAD** (decisión de producto/lane, no código de este slice) | — |

## Context Save Point

- Handoff FINAL: implementación + verificación COMPLETAS (5/5 steps ✅) + **review vanta-audit APPROVE con batch aplicado** (F-01 Medium + F-02/03/04 Low + Rec.3; F-05/06 no-aplicados documentados; F-07 → FIND LEAD). Cierre = LEAD: commit local; push solo con orden del owner. Evidencia por cláusula en §Verificación mecánica + §Review-fix batch.
- Core full: `cargo nextest --profile audit -p vantadb --build-jobs 2 --no-fail-fast -E 'not binary(public_api)'` → **2533/2533 ✅**. `public_api` excluido = fail EXTERNO de VER-02 (su `pub mod attestation` desactualiza el snapshot; su sesión ya lo está arreglando).
- Notas de entorno (no del slice): (1) disco C: se llenó 2× durante la wave (LNK1140 / os error 112) — mitigado borrando `target/debug/incremental` y PDBs >20min de `target/debug/deps` (35.8GB acumulados; quedan ~34GB libres); (2) `campaign_get_next_task`/`campaign_verify_cmd` fallan con "Ambiguous active plan" por un bug del server (`campaign-server.mjs:581` → `budgetStatus` re-resuelve `findPlanFile` sin propagar `planFile`); workaround: `campaign_get_task_detail`/`campaign_update_task_state` con `planFile`; (3) `check-docs` fail externo por VER-06.md; `gen-index` stale cross-sesión (la consolidación ya regenera).
- Deviación vs Spec (menor, documentada): `EnvelopeConfig` vive en `envelope.rs` (patrón de la casa de `RedactConfig`/`ContextConfig`) y `ProxyConfig` lo referencia — no en `config.rs` como decía el borrador de ARCHIVOS.
- Evidencia de índices: este feature set no compila BM25 (`advanced-tokenizer` off) → el test escanea TODOS los namespaces del store en vez de un query textual (evidencia igual o más fuerte para "0 PII en claro"); marcado en el test.
