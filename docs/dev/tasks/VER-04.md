---
title: "VER-04: Governance de inyección (presupuesto + ACLs + audit log)"
kind: task
description: "Enforcement real del presupuesto de inyección (proxy e2e + MCP), ACL opt-in por prefijo de namespace en las superficies de inyección (proxy block + recall), y audit log WORM-ready por inyección (ns/key/score/budget/truncado/ACL) reusando AuditEvent/AuditLogger (cadena VER-01 citada, no duplicada)."
---

# VER-04: Governance de inyección (presupuesto + ACLs + audit log)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 40, Fase F4 — wave F4.3)
- **Fuente:** Backlog:933 (P52) + plan Task 40 (`L1047-1071`, bloque verbatim leído)
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟠
- **Tipo:** Rust (vanta-memory + vanta-proxy + vantadb-mcp + core audit) (+ Docs)
- **Turns estimados:** 8-10
- **Creado:** 2026-09-29
- **Estado:** ✅ implementación + verificación + **batch de review (F1/F2/F3) aplicado** — cierre (commit + re-review P2-01) = LEAD
- **Incógnitas (uphill):** 0 abiertas — 3 superficies verificadas; presupuesto proxy unit-test existe (`inject.rs:589-651`), falta e2e + MCP; RBAC core es `pub(crate)` (no consumible por proxy/MCP) → ACL propia opt-in; `desktop/**` construye `RecallConfig` literal (`desktop/src-tauri/src/commands/memory.rs:266`) → PROHIBIDO agregar campos a `RecallConfig`/`AutoRecallParams`; se usa función nueva `perform_auto_recall_governed`.
- **Pendientes (downhill):** 0 steps propios. Cierre LEAD: review P2-01 fresh + commit local `feat: VER-04 — governance de inyección` (NO incluir el plan file — ya modificado por el orquestador).
- **Branch:** develop · **Commit:** (LEAD)
- **Co-batch:** wave F4.3 (única en vuelo) — **NO tocar**: `src/wal*.rs` (VER-01), `src/attestation.rs`/`src/shred/**`/`src/gc.rs`/`src/storage/engine/delete.rs` (VER-02), `vanta-proxy/src/{envelope,redact}.rs` + `src/crypto.rs` (VER-03), `src/cli_handlers/export_md.rs`/`src/cli_handlers/index.rs`/`vanta-memory/src/seed/**` (VER-06)
- **PROHIBIDO (wave/plan):** `docs/dev/Backlog.md` · `.github/workflows/perf-bench.yml` · `CONSTRAINTS.md` · `desktop/**` · `opencode.jsonc` · plan file (LEAD)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-proxy/src/server.rs:434-435` (`build_memory_block` — único caller de producción), `vanta-proxy/src/mem_command.rs:406` → `memory_tools::execute` (`mem: search`), `vantadb-mcp/src/handlers/tools.rs` (`memory_recall` :1806, `inject_context` :2085), `vantadb-mcp/src/context.rs` (`context_assemble` :71), `vanta-memory/src/core/hooks/{mod.rs,pipeline_worker.rs,context_assembly}` (`perform_auto_recall` 16 callers — la variante gobernada es ADITIVA; los callers existentes no cambian) |
| Callees | `vantadb::audit::{AuditEvent, AuditLogger}` (`pub mod audit`, `with_rotation`/`record` públicos), `vanta_memory::core::hooks` (recall + policy nueva), `vanta_memory::core::{persona,scene}` (namespaces canónicos `persona/<s>`, `scene/<s>`), `src/audit.rs` (constructor `injection` nuevo) |
| Implicaciones | API pública aditiva: `AuditEvent::injection` (core), `InjectionPolicy` + `perform_auto_recall_governed` + `RecallGovernance` + campos `RecalledMemory.{source_namespace,source_key}` (vanta-memory). `InjectionBlock` (vanta-proxy, `publish=false`) cambia la firma de `build_memory_block` — 1 caller + 1 test. `ProxyConfig.injection` gana 2 campos con `#[serde(default)]` (compat de TOML; los literales de tests usan `Default::default()`). `McpConfig` gana campos con env vars (2) — tests usan `Default`. Audit = opt-in por path (default off → 0 cambio de comportamiento). ACL = opt-in por allowlist (default vacío → allow-all, comportamiento actual). Sin cambios de wire/serialización on-disk. Sin migración. Performance: O(1) por sección/hit en paths ya lineales. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o regiones núcleo):** `vanta-proxy/src/inject.rs` (:1-300, :559-653), `vanta-proxy/src/config.rs` (:14-110, :298-340), `vanta-proxy/src/server.rs` (:36-200, :394-468), `vanta-proxy/src/session.rs` (494L), `vanta-proxy/src/auth.rs` (:1-210), `vanta-proxy/src/memory_tools.rs` (:55-145, :480-503), `vanta-proxy/src/cost.rs` (:78-98), `vanta-proxy/tests/pipeline.rs` (:1-100), `vanta-proxy/Cargo.toml`, `vantadb-mcp/src/config.rs` (167L), `vantadb-mcp/src/context.rs` (159L), `vantadb-mcp/src/handlers/tools.rs` (:330-409, :590-660, :1295-1330, :1640-1680, :1800-1943, :2040-2153, :3360-3430), `vantadb-mcp/src/validation.rs` (:519-640), `vanta-memory/src/core/hooks/auto_recall.rs` (:36-330, :346-545), `vanta-memory/src/core/record/l1_reader.rs` (322L), `vanta-memory/src/context_engine/engine.rs` (:175-250), `src/audit.rs` (:18-180), `src/sdk/builder.rs` (:120-150, :291-311), `src/rbac.rs` (170L), `src/server/middleware.rs` (:38-258), `src/lib.rs` (:65/:131), `docs/dev/tasks/VER-03.md` (formato).
- **Referencias hacia dentro (imports):** `inject.rs` → `vanta_memory::core::{persona::persona_generator,scene::scene_index}`, `crate::cost::estimate_text_tokens`; `memory_tools.rs` → `vanta_memory::core::hooks::{perform_auto_recall, AutoRecallParams, RecallConfig}`; `server.rs` → `crate::{inject, session, auth, writeback, ...}`; `context.rs` → `vanta_memory::core::hooks::{...}` + `vanta_memory::context_engine::assemble_with_recall`; `tools.rs` (`memory_recall`) → `vanta_memory::core::hooks::{...}`.
- **Referencias entrantes:** `build_memory_block` ← `server.rs:435` + test `inject.rs:589-651` (2 sitios); `perform_auto_recall` ← 16 callers (proxy/mem_command, mcp/context, mcp/tools, desktop, pipeline, tests) — **firma intacta** (variante gobernada aditiva); `RecallConfig` ← 10 literales (vanta-memory tests, mcp/tools.rs:1844, desktop/memory.rs:266) — **sin campos nuevos** por desktop; `McpConfig` ← `from_storage` + tests; `AppState` ← tests del proxy (literales via `from_engine`); `AuditEvent` ← middleware server/handlers/conversation + desktop mirror (`desktop/src-tauri/src/connections/types.rs:253` — op values nuevos no lo rompen; solo se agrega un constructor).
- **Veredicto impacto:** 🟡 medio — aditivo en su mayoría (constructores/structs/funciones nuevas + config opt-in); 1 firma interna cambiada (`build_memory_block`, crate `publish=false`) + 1 campo nuevo en `RecallResult`/`RecalledMemory` (construidos en un solo sitio, `auto_recall.rs`; consumidores por campo). Revertible por slice (cada pieza nueva degrada a no-op con config default).

## Contrato

(verbatim del plan Task 40 — ley)

"presupuesto de inyección enforced y consistente por request (bloque `<vanta-memory>` ≤ budget; budget 0/bajo → inyección vacía o recortada verificada en test e2e del proxy; envelope MCP coherente con `byte_count`/`truncated`) Y ACLs por tool/namespace aplicadas a las superficies de inyección (precedente rbac.rs/middleware SRV-05; deny fuera de scope) Y audit log de inyección consultable: {sesión/prompt, memoria inyectada (ns/key), score, budget/truncado, decisión ACL} como eventos WORM-ready (preparado para el chain de VER-01, citado) Y doc (config + consulta del audit)"

## Spec (SDD — feature-add: símbolos públicos nuevos)

> Gate D: el contrato del plan prescribe los 3 bloques y Ruta vanta-worker; stop conditions pre-autorizan el mínimo (ACL por namespace de sesión, sin motor ABAC; sin UI de consulta). Decisiones resueltas por evidencia de código (file:line). Sin pregunta al usuario: alcance fijado por el plan + prompt de wave.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Superficies de inyección cubiertas | (A) proxy block + proxy `mem:search` + MCP `memory_recall` + MCP `context_assemble` + MCP `inject_context` / (B) solo proxy / (C) + desktop | **A** — las 3 del plan (proxy/MCP/auto_recall); desktop sin tocar (prohibido + consume el hook compartido) | ✅ decidido-por-evidencia (plan :1060; `rg perform_auto_recall` = 16 callers; desktop prohibido) |
| 2 | Dónde se enforcea el ACL | (A) hook compartido `perform_auto_recall_governed` + `build_memory_block` (choke points de lectura, un solo predicado) / (B) en cada tool por separado / (C) motor ABAC | **A** — precedente del plan "un choke point" (l1_reader :51-53); ABAC = stop condition → fuera | ✅ decidido-por-evidencia (stop condition plan :1063) |
| 3 | Forma del ACL | (A) allowlist de prefijos de namespace (vacía = allow-all, opt-in, vacío ⇒ compatible) / (B) denylist / (C) roles por tool | **A** — "deny fuera de scope" = lo no listado se deniega; `rbac.rs` usa `Permission::NamespaceRead(String)` (match exacto) — prefijos son el mínimo viable sin motor de políticas | ✅ decidido-por-evidencia (`src/rbac.rs:7-20/:70-85`; pre-mortem F2 "ACL opt-in default actual") |
| 4 | Dónde vive el ACL compartido | (A) `vanta_memory::core::hooks::InjectionPolicy` (vanta-proxy y vantadb-mcp ya dependen de vanta-memory) / (B) duplicar por crate / (C) core `vantadb` (no lo usan los otros sin API nueva) | **A** — fuente única (pre-mortem F3 "única fuente"); 0 dependencias nuevas | ✅ decidido-por-evidencia (`vanta-proxy/Cargo.toml:33`, `vantadb-mcp/Cargo.toml`) |
| 5 | Cómo llega la policy al hook sin romper callers | (A) función nueva `perform_auto_recall_governed(.., policy)`; `perform_auto_recall` delega con allow-all / (B) campo nuevo en `RecallConfig`/`AutoRecallParams` | **A** — `desktop/**` (prohibido) construye `RecallConfig` literal (`memory.rs:266`); B rompe desktop | ✅ decidido-por-evidencia (grep 10 literales) |
| 6 | Budget MCP de contenido | (A) `config.byte_budget` (40 KB default, env `VANTADB_MCP_BYTE_BUDGET`, clamp 1KB-1MB) = fuente única; exceder ⇒ error de validación (fail-closed) / (B) `max_payload_length` (1 MB actual, solo cap de payload) / (C) truncar silenciosamente | **A** — pre-mortem F3 ("única fuente `InjectionConfig`/`byte_budget`") + "presupuesto que corta de verdad (fail-closed/declarado)"; al máximo del clamp = mismo techo que hoy | ✅ decidido-por-evidencia (`config.rs:104-114/:138-140`; prompt wave INVESTIGACIÓN PROBLEMA) |
| 7 | Budget MCP de recall | (A) char caps de `RecallConfig` derivados de `byte_budget` (total y por memoria) + envelope `byte_count`/`truncated` vía `budget_value` existente / (B) sin caps (hoy) | **A** — "envelope MCP coherente con `byte_count`/`truncated`"; `budget_value` es el chokepoint MCP-39 | ✅ decidido-por-evidencia (`validation.rs:519-640`; contrato c1) |
| 8 | Audit: destino | (A) `AuditLogger` (JSONL append-only + rotación, `pub mod audit`) abierto por cada superficie desde su config (proxy `[injection] audit_log_path`, MCP `VANTADB_MCP_AUDIT_LOG`) / (B) publicar `Embedded::audit` (API core nueva + snapshot) / (C) log dedicado nuevo | **A** — reusa el modelo/logger existentes sin tocar API del core (0 snapshot); opt-in; `init_audit` usa la misma política "no bloquear" | ✅ decidido-por-evidencia (`src/audit.rs:132-163`; `builder.rs:123-138`; VER-01 cita: chain cubre WAL) |
| 9 | Audit: granularidad y PII | (A) un evento por memoria inyectada (ns/key/score donde exista) + un evento por denegación ACL; metadata only (nunca payload) / (B) un evento resumen por request / (C) payload completo | **A** — contrato "{sesión/prompt, memoria inyectada (ns/key), score, budget/truncado, decisión ACL}"; pre-mortem F1: sin valores (precedente kinds sin valores) | ✅ decidido-por-evidencia (plan :1062; `redact.rs:65`) |
| 10 | Audit: compatibilidad chain VER-01 | (A) JSONL append-only + rotación + `op=injection` determinista; chain/WAL NO se toca (citado) / (B) encadenar el audit dentro de VER-01 | **A** — VER-01 ya cerrado (`WAL_FORMAT_VERSION=3` cubre WAL); el plan dice "consumidor del chain, citado, sin duplicar" | ✅ decidido-por-evidencia (plan :1054/:1071; `VER-01.md:39-40`) |
| 11 | Budget proxy e2e | (A) test e2e con mock upstream (`tests/ver04_governance.rs`, patrón `pipeline.rs`) que corre 0/bajo/amplio + ACL + audit / (B) solo unit test existente | **A** — DoD Backlog: "presupuesto respetado en test e2e"; el unit test no cruza `server.rs` | ✅ decidido-por-evidencia (Backlog:933; `tests/pipeline.rs:30-100`) |

## Diseño (explícito)

**1. `InjectionPolicy` (nueva, `vanta-memory/src/core/hooks/auto_recall.rs`):**

```rust
pub struct InjectionPolicy { allow_prefixes: Vec<String> }  // vacía = allow-all
impl InjectionPolicy {
    pub fn allow_all() -> Self;
    pub fn from_prefixes(prefixes: impl IntoIterator<Item = impl Into<String>>) -> Self;
    pub fn is_empty(&self) -> bool;
    pub fn allows(&self, namespace: &str) -> bool;  // prefijo con frontera `/` (o exacto)
}
```

**2. `perform_auto_recall_governed(db, params, embed, policy) -> Result<Option<RecallResult>, RecallError>`:**
- `perform_auto_recall` = delegación con `InjectionPolicy::allow_all()` (firma intacta — 16 callers).
- Enforcement por fuente ANTES de leer: L1 sesión (`l1/<s>`), L1 cross-session (cada `l1/*`), persona (session + scoped), escenas (`scene/<s>`).
- `RecallResult.governance: RecallGovernance { sources: Vec<String>, denied: Vec<String> }` (bounded a 16; los denegados NO son silenciosos).
- `RecalledMemory.{source_namespace, source_key}` (`#[serde(default)]`) para audit ns/key por hit.

**3. Proxy:**
- `InjectionConfig` gana `namespace_allow_prefixes: Vec<String>` (default `[]`) + `audit_log_path: String` (default vacío = off).
- `build_memory_block(db, session, max_tokens, policy) -> InjectionBlock { block, sources: Vec<InjectedSource{namespace,key,kind}>, denied: Vec<String>, budget_tokens, used_tokens, truncated }`.
- `AppState.audit: Option<Arc<AuditLogger>>` (abierto en `from_engine`; fallo ⇒ warn + None).
- `server.rs` emite audit (por sección + por denegado) tras construir el bloque; `mem_command`/`memory_tools::search` reciben `Option<&Arc<AuditLogger>>` + policy y usan recall gobernado (budget char = `injection.max_tokens*4` cuando `max_tokens>0`).

**4. MCP:**
- `McpConfig` gana `injection_namespaces: Vec<String>` (env `VANTADB_MCP_INJECT_NAMESPACES`, coma-separado) + `audit: Option<Arc<AuditLogger>>` (env `VANTADB_MCP_AUDIT_LOG`; abierto en `from_storage`).
- `memory_recall`: recall gobernado; char caps = `byte_budget`; envelope `{prepend_context, recalled, effective_mode, byte_count, truncated}` (vía `budget_value`); audit por memoria + denegados.
- `context_assemble`: recall gobernado; envelope gana `byte_count`/`truncated` (derivado de `report.msgs_conserved < msgs_before` o budget); audit.
- `inject_context`: `content.len() > byte_budget` ⇒ error de validación fail-closed (declarado); respuesta gana `byte_count`; audit.

**5. Audit (schema, `op = "injection"`):** `namespace` = ns de la memoria fuente (o sesión para el resumen de bloque), `key` = key fuente (`N/A` agregado), `outcome` ∈ `{ok, denied}`, `reason` = `surface=<proxy|mcp>;tool=<...>;session=<...>;kind=<persona|scene|scene_index|l1>;score=<n|n/a>;budget=<used>/<max>;truncated=<b>;acl=<allow|deny>`. WORM-ready: JSONL append-only + rotación; chain VER-01 citado (no se duplica). Sin valores de contenido (PII = 0).

## Invariantes de dominio (handoff — MUST)

- (1) Defaults = comportamiento actual byte-idéntico: ACL vacía = allow-all; audit path vacío = no-op; `perform_auto_recall` intacta; `RecallConfig`/`AutoRecallParams` sin campos nuevos (desktop compila sin tocar).
- (2) `build_memory_block` con `max_tokens` = 0 sigue devolviendo bloque vacío; `estimate_text_tokens(block) <= max_tokens` SIEMPRE (hard cap).
- (3) Audit NUNCA registra valores de contenido (solo ns/key/score/budget/acl) — pre-mortem F1.
- (4) Fail-loud en config inválida de budget (MCP `inject_context` > budget ⇒ error tipado); NUNCA truncado silencioso.
- (5) No tocar las regiones de otras waves (lista en Metadata) ni `desktop/**`/Backlog/perf-bench/CONSTRAINTS/opencode.jsonc/plan.
- (6) Chain/WAL (VER-01) no se modifica — solo se cita en docs.
- (7) El recall gobernado no cambia resultados cuando la policy es allow-all (regresión byte-idéntica en tests existentes).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** 0 — aditivo (constructores/structs/funciones nuevas; config opt-in). Sin `unsafe`, sin dependencias nuevas (usa `vantadb::audit` y `serde_json` ya presentes), sin hot paths nuevos. **Pago:** los tests e2e nuevos cubren el presupuesto del proxy que solo tenía unit test; el envelope MCP gana coherencia presupuestal. **Deuda declarada (diferida):** firma/encadenado criptográfico del audit de inyección (requiere ancla externa; VER-01 dejó el follow-up para `vanta-audit`) → FIND solo si el cierre lo pide; ABAC por rol/tool (MGR-04) fuera por stop condition.

## Verificación

| Ítem | Comando → resultado esperado |
|------|------------------------------|
| c1 proxy budget e2e | `cargo nextest run -p vanta-proxy --test ver04_governance` → **4/4 ✅** — `budget_zero_disables_block_and_low_budget_truncates` (0 ⇒ sin bloque; 60 ⇒ `…[truncated]` con `estimate ≤ 60`; 10k ⇒ persona), `acl_denies_sources_outside_the_allowlist`, `injection_audit_logs_sources_budget_and_acl_without_payload`, `acl_denials_are_audited_with_deny_outcome` |
| c1 MCP budget/envelope | `cargo nextest run -p vantadb-mcp --test ver04_governance` → **6/6 ✅** — `inject_context_rejects_content_over_the_byte_budget` (fail-closed + knob), `inject_context_within_budget_reports_byte_count_and_audits_metadata_only`, `memory_recall_envelope_carries_byte_count_and_source_identity`, `memory_recall_truncates_when_the_response_exceeds_the_byte_budget`, `memory_recall_acl_denies_out_of_scope_namespaces_and_audits_the_denial`, `context_assemble_envelope_carries_byte_count_and_truncated` |
| c2 ACL | unit `auto_recall::tests::injection_policy_*` (3) + `vanta-proxy/tests/ver04_governance` deny + `vantadb-mcp/tests/ver04_governance` deny — **✅** |
| c3 audit | e2e proxy JSONL (`op=injection`, ns/key/budget/acl, sin payload) + MCP JSONL (`tool=inject_context`/`memory_recall`, denial) — **✅** |
| c4 doc | `PROXY.md` § Injection governance + `MCP.md` § Injection governance + `CONFIGURATION.md` cross-ref + `scripts/validate-docs-coverage.ps1` **0 gaps** |
| Gates | fmt ✅ · clippy scoped ✅ · nextest 576/310/258 ✅ · snapshot `public-api.txt` regenerado (+1) |

### Batch de review vanta-audit (F1/F2/F3 — 2026-09-29)

| Finding | Fix | Evidencia (comando → resultado) |
|---------|-----|---------------------------------|
| **F1 Medium** (todo-denegado descartaba `governance.denied`) | early-return de `perform_auto_recall_governed` exige `governance.denied.is_empty()` — el path denegado devuelve `Some` con contextos `None` (wire idéntico por `unwrap_or_else(NO_MEMORIES)`/envelope null); allow-all intacto | RED: `deny_all_pass...` paniquea "must still return Some" + MCP `deny-all must still audit its denials: []` (0 filas) → GREEN: `cargo nextest run -p vanta-memory --test ver04_governance` **5/5** · `-p vantadb-mcp --test ver04_governance` **7/7** |
| **F2 Low** (metadata post-medición excedía el budget) | `recall_envelope` mide contra `byte_budget - 64` y reporta el tamaño final entregado; MCP.md re-alinea el claim de `context_assemble` (token_budget MEM-37 ≠ byte_budget) | `memory_recall_envelope_carries_byte_count_and_source_identity` + `memory_recall_truncates...` ✅ |
| **F3 Low** (cota 16 no documentada) | `RecallGovernance::OVERFLOW_MARKER = "…overflow"` en la última ranura al saturar + doc en PROXY.md/MCP.md | unit `recall_governance_bounds_lists_and_marks_overflow` ✅ |
| **F4/F5 Info** | sin acción del worker (commit/lista = LEAD; semgrep = LEAD) | — |
| Re-verify | `-p vanta-memory --test ver04_governance` **5/5** · `-p vanta-proxy --test ver04_governance` **4/4** · `-p vantadb-mcp --test ver04_governance` **7/7** · `-p vanta-memory` full **576/576** · `-p vanta-proxy` full **310/310** · `-p vantadb-mcp` full **258/258** · fmt ✅ · clippy scoped ✅ · markdownlint `docs/api/{MCP,PROXY}.md` **0 issues** · docs gates post-batch: `check-docs` **all clear** + `gen-index --check` exit 0 (regenerados externamente, incluyen VER-04) | ✅ |

## Cierre (handoff LEAD)

- **Review vanta-audit:** ❌→ fixes F1/F2/F3 aplicados (S6); re-verify verde. F4/F5 (Info) = LEAD.
- **Commit (LEAD):** `feat: VER-04 — governance de inyección (presupuesto + ACLs + audit log)` — archivos: `src/audit.rs` · `vanta-memory/src/core/hooks/{auto_recall.rs,mod.rs}` · `vanta-proxy/{Cargo.toml,src/{config,inject,server,memory_tools,governance.rs,lib}.rs,tests/ver04_governance.rs}` · `vantadb-mcp/{src/{config,context,handlers/tools,governance.rs,lib}.rs,tests/ver04_governance.rs}` · `vanta-memory/tests/ver04_governance.rs` · `tests/api/public-api.txt` · docs (`PROXY.md`, `MCP.md`, `CONFIGURATION.md`) · `Cargo.lock` (dev-dep `tempfile` de vanta-proxy) · este task file. **NO incluir** `docs/dev/plans/2026-09-26-master-roadmap.md` (LEAD) — ya viene modificado por el orquestador.
- **Deuda/observaciones:** (1) docs gates **all clear** al cierre del batch: `check-docs` gating ✅ + `gen-index --check` exit 0 (índices regenerados externamente por el orquestador/lane docs durante el batch, ya incluyen VER-04; `VER-06.md` ganó frontmatter externamente); (2) `campaign_verify_cmd` bloqueado por FIND-193 (planFile ambiguo) — verificación registrada por shell + recitation; (3) workspace-wide nextest/clippy sigue bloqueado por la deuda pre-existente `vanta-memory/tests/smoke.rs:15` × feature-unification (`llm-driver`) — documentada en VER-01 §Deuda 5; (4) firma/encadenado criptográfico del audit = deuda v1.0 citada (VER-01 / `vanta-audit`).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato ✅ cláusula por cláusula (budget e2e proxy + envelope MCP + ACL allow/deny + audit consultable + doc) + fmt/clippy/nextest verdes |
| **Commit** | LEAD (el worker NO commitea); conventional commit `feat: VER-04 — governance de inyección` |
| **Release** | N/A local (release-plz post-push, lane owner) |

## Herramientas necesarias
- Terminal cargo (nextest scoped con `CARGO_BUILD_JOBS=2`, clippy, fmt)
- codegraph_codegraph_explore (blast radius — hecho) · `node scripts/docs/*.mjs` · `scripts/validate-docs-coverage.ps1`
- FIND-173/177 vigentes (prohibiciones de co-batch)

**Skills cargadas (SDP v3):** `security-and-hardening` (trust boundary + checklist LLM06/LLM10), `test-driven-development` + `rust-write-tests` (RED→GREEN, sin env mutation, TempDir), `source-driven-development` (APIs verificadas en código real, sin deps nuevas), `doubt-driven-development` (stakes de governance — review adversarial delegado al LEAD), `incremental-implementation` (slices S1–S5), `context-engineering`. Base fija `campaign-executor`/`progreso` vía MCP. Descartada `frontend-ui-engineering` (sin UI).

## Steps (atomics — PLAN → ACT → VERIFY)

- [x] **S1 (vanta-memory + core):** `InjectionPolicy` + `perform_auto_recall_governed` + `RecallGovernance` + `RecalledMemory.{source_namespace,source_key}` + `AuditEvent::injection`. RED→GREEN con unit tests (policy prefijos; allow-all byte-idéntico; deny excluye L1/persona/scena y reporta en governance). Verify: `cargo nextest run -p vanta-memory -E 'test(governance) | test(injection) | test(policy)'` + `-p vantadb -E 'test(audit)'`. → ✅ **574/574 vanta-memory** + audit unit PASS; `tests/ver04_governance.rs` 4/4.
- [x] **S2 (vanta-proxy):** config (`namespace_allow_prefixes`, `audit_log_path`), `InjectionBlock` + policy en `build_memory_block`, `AppState.audit` + emisión en `server.rs`, recall gobernado + audit en `memory_tools`, test e2e `tests/ver04_governance.rs` (0/bajo/amplio + ACL + JSONL). Verify: `cargo nextest run -p vanta-proxy`. → ✅ **310/310** (e2e ver04 4/4, `governance.rs` unit 3/3, inject test actualizado).
- [x] **S3 (vantadb-mcp):** config (2 env knobs + helper `injection_policy`), `memory_recall` (budget+envelope+audit), `context_assemble` (envelope+audit), `inject_context` (budget fail-closed + byte_count + audit), tests `tests/ver04_governance.rs`. Verify: `cargo nextest run -p vantadb-mcp --ignore-default-filter`. → ✅ **6/6** nuevos + **257/257** full package.
- [x] **S4 (docs):** `PROXY.md` § Injection governance (config + consulta jq) + tabla defaults + `MCP.md` § Injection governance (env knobs + envelopes) + filas de tools + cross-ref en `CONFIGURATION.md` (op `injection`). Verify: check-links (dentro de budget) · check-docs (gating externo `VER-06.md`, mis archivos limpios) · validate-docs-coverage **0 gaps** · gen-index stale (externo, ver §Deuda). → ✅
- [x] **S5 (cierre):** fmt ✅ · clippy ✅ (vanta-memory/vanta-proxy/vantadb-mcp `--all-targets --all-features -D warnings` + vantadb `--lib`) · nextest: vanta-memory **574/574**, vanta-proxy **310/310**, vantadb-mcp **257/257**, vantadb **2543/2543** · `public-api.txt` regenerado (**+1 línea**, solo `AuditEvent::injection`). `campaign_verify_cmd` bloqueado por FIND-193 (bug planFile, workaround solo update/detail — VER-03 lo documentó). → ✅
- [x] **S6 (batch review vanta-audit — F1 Medium + F2/F3 Low):** (F1) deny-all deja de colapsar en `Ok(None)` cuando `governance.denied` no está vacío → los denials se auditan aunque no se inyecte nada (`auto_recall.rs` early-return); +2 tests RED→GREEN (vanta-memory deny-all `Some`+denied; MCP deny-all → `outcome=denied` en JSONL). (F2) `recall_envelope` mide con 64 B de reserva para `byte_count`/`truncated` (post-medición) y `byte_count` reporta el tamaño ENTREGADO; claim re-alineado en MCP.md (`context_assemble` presupuesta por `token_budget` MEM-37, no por `byte_budget`). (F3) cota `MAX_ENTRIES=16` documentada en PROXY.md/MCP.md + marcador `…overflow` en la última ranura + unit test. Re-verify completo ✅.

## Investigation Notes

### Código — superficies verificadas (2026-09-29)
- **Proxy block:** `build_memory_block` (`inject.rs:106-153`) arma persona+escena+índice con `fit_sections` (`:159-195`, hard cap con `…[truncated]`); `max_tokens==0` ⇒ `String::new()` (`:110`); único caller `server.rs:434-435`; unit test `:589-651` ya cubre prioridad/presupuesto (falta e2e + ACL + audit).
- **MCP:** `inject_context` (`tools.rs:2085-2153`) inserta mensaje a thread vía IQL; único cap = `max_payload_length` (:2124). `memory_recall` (`:1806-1907`) usa `perform_auto_recall` con `RecallConfig` sin caps de chars; envelope `{prepend_context, recalled, effective_mode}` sin `byte_count`/`truncated`. `context_assemble` (`context.rs:71-129`) usa `RecallConfig::default()` + `assemble_with_recall` (presupuesto compartido MEM-37); envelope `{messages, report..., mmd_injected, recall_injected}`.
- **Envelope MCP-39:** `budget_value`/`apply_output_budget` (`validation.rs:519-640`): trunca el primer array top-level; sin array ⇒ intacto + truncated=false; `byte_budget` default 40 KB (env, clamp 1KB-1MB).
- **ACL existente:** `rbac.rs` (`pub(crate)`; `Permission::NamespaceRead/Write(String)` match exacto) consumido solo por `server/middleware.rs:180-223` (HTTP). No consumible por proxy/MCP → policy propia compartida en vanta-memory.
- **Audit existente:** `src/audit.rs` — `AuditEvent` (timestamp/op/namespace/key/outcome/reason/request_id) + `AuditEvent::{new,memory,auth}`; `AuditLogger::with_rotation/record` públicos; `pub mod audit` (`lib.rs:65`); opt-in `Config.audit_log_path` (10MB/5 archivos). Proxy/MCP NO registran audit hoy (`rg audit` en vanta-proxy = writeback labels).
- **Recall:** `perform_auto_recall` (`auto_recall.rs:198-279`) lee `l1/<session>` + scoped `l1/*` (filtro agent/team), persona (scoped + session) y escenas; `apply_recall_budget` (:498) char caps opcionales; choke point cuarentena `l1_reader.rs:51-53` (`include_quarantined:false`) — los gates SCH-05 quedan intactos.

### Re-baselines y decisiones de implementación (2026-09-29)

- **`RecallConfig` prohibido de tocar:** `desktop/src-tauri/src/commands/memory.rs:266` construye el struct literal → se añadió `perform_auto_recall_governed(.., policy)` aditiva (allow-all por default) en vez de un campo. Igual con `AutoRecallParams` (16 call sites).
- **Constructor `AuditEvent::injection`** (core) en vez de eventos ad-hoc: reusa el JSONL/rotación existentes; `op = "injection"`, `outcome ∈ {ok, denied}`, `reason` = `k=v;` metadata (sin valores). Snapshot public-api regenerado (+1 línea).
- **ACL = allowlist de prefijos por namespace** (boundary-aware) en `vanta-memory::core::hooks::InjectionPolicy` (fuente única proxy/MCP); aplicada en `build_memory_block` (proxy) y `perform_auto_recall_governed` (todas las superficies de recall). Default vacío = allow-all.
- **Budget MCP:** `byte_budget` como fuente única — `inject_context` fail-closed (error de validación, menciona el knob), `memory_recall`/`context_assemble` con char caps + `budget_value` + `byte_count`/`truncated` en `structuredContent` (mismo shape que los envelopes de search).
- **Audit sinks opt-in por superficie:** proxy `[injection] audit_log_path` (TOML) y MCP `VANTADB_MCP_AUDIT_LOG`; apertura fallida ⇒ warn + deshabilitado (nunca bloquea). `VANTADB_MCP_INJECT_NAMESPACES` para el ACL del MCP.
- **Docs — re-baseline del plan:** el bloque listaba `docs/user/operations/CONFIGURATION.md`, pero esa página documenta el `VantaConfig` del core; la config TOML del proxy vive en `docs/api/PROXY.md` (precedente VER-03). Se documentó ahí + `MCP.md`, y `CONFIGURATION.md` ganó el cross-ref del op `injection` (una sola fuente por hecho).

=== RECITATION VER-04 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: VER-04 — Governance de inyección (presupuesto + ACLs + audit log)
Estado: in-progress (implementación COMPLETA + batch review F1/F2/F3 aplicado; commit = LEAD)
Última acción: batch review vanta-audit ❌→fixes: F1 (deny-all deja de colapsar en Ok(None) → denials auditados; RED repro MCP "0 filas" → GREEN outcome=denied), F2 (reserva 64 B en recall_envelope + claim context_assemble re-alineado), F3 (cota 16 + marcador …overflow documentados/testeados); re-verify 5/5 · 4/4 · 7/7 · full 576/310/258 · fmt/clippy/markdownlint ✅
Resultado: OK (pendiente solo commit + re-review del LEAD)
Próxima acción: LEAD — re-review P2-01 (delta F1/F2/F3) + commit `feat: VER-04 — governance de inyección (presupuesto + ACLs + audit log)`
Contrato: ver bloque Task 40 del plan — cláusulas c1–c4 ✅ (evidencia §Verificación + §Batch review)
Próxima tarea si completa: gate F4 → F5
=== END RECITATION ===
