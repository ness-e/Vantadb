---
title: "ICP-02: Track local-LLM/privacidad — one-pager + demo E2E + auditoría PII"
kind: task
description: "Track privacidad: one-pager de garantías exactas, auditoría PII de producto (script versionado: store+índices+export = 0 en claro), demo CI E2E (captura PII → auditoría → delete --attest + certificate verify exit 0 → audit op=injection) y entrada en COMPARISON.md. Disarmed verificado."
---

# ICP-02: Track local-LLM/privacidad — one-pager + demo E2E + auditoría PII

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 43, Fase F5 — wave F5.1)
- **Fuente:** plan Task 43 (`L1125-1149`, bloque verbatim leído) + contexto de wave del orquestador
- **Esfuerzo:** 🟡 1sem (estimado plan) — ejecución real: 1 sesión (los 3 pilares ya existen; el track pasa de "implementar" a "demostrar")
- **Prioridad:** 🟠
- **Tipo:** Rust (vanta-proxy: bin de auditoría + test E2E) + scripts/CI + Docs
- **Turns estimados:** 6-8
- **Creado:** 2026-09-29
- **Estado:** ✅ implementación + verificación completas; **batch de review 2026-09-30 aplicado** (CRITICAL byte-scan + REQUIRED renumber + OPTIONAL/NIT) y re-verificado — cierre final (review P2-01 fresh + commit) = LEAD · **Branch:** develop · **Commit:** (LEAD)
- **Deps:** VER-02 ✅ (`ccc51d09`) · VER-03 ✅ (`95407c78`) · VER-04 ✅ (`575ce8dd`) — las 3 garantías ya están en código (F4)
- **Incógnitas (uphill):** 0 abiertas — (1) `vanta-proxy` compila `vantadb` sin `fjall` (`cargo tree -e features`: solo `encryption` + `llm-driver` de vanta-memory) → el store on-disk del demo requiere `--features vantadb/fjall` (precedente documentado: `api05_snapshot_auth.rs:163-166`, live smoke con fjall); (2) el test E2E debe ser `#[ignore]` para no romper la suite default sin el feature; (3) `delete --attest` / `certificate verify` viven en el bin `vanta-cli` (crate `vantadb`), no invocables desde el test de vanta-proxy (CARGO_BIN_EXE es por-crate) → el encadenado cross-process lo orquesta el script.
- **Pendientes (downhill):** 0 steps propios. Cierre LEAD: review P2-01 fresh (diff `docs/api/**` ⇒ tier adversarial) + commit local `feat: ICP-02 — track privacidad: one-pager + demo E2E + auditoría PII` (NO push).
- **Branch:** develop · **Commit:** (LEAD)
- **Co-batch:** wave F5.1 (ICP-01 ‖ VER-08 en vuelo) — **NO tocar**: `vanta-proxy/src/governance.rs`, `vanta-memory/src/core/hooks/auto_recall.rs`, `vantadb-mcp/**` (solo LEER para citar), regiones de ICP-01/VER-08.
- **PROHIBIDO (wave/plan):** `docs/dev/Backlog.md` · `.github/workflows/perf-bench.yml` · `CONSTRAINTS.md` · `desktop/**` · `opencode.jsonc` · plan file (LEAD)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno nuevo sobre código existente: el bin `vanta-pii-audit` y el test `icp02_privacy_demo` son artefactos NUEVOS; el script/workflow/one-pager también. `docs/user/COMPARISON.md` y `docs/api/PROXY.md` reciben secciones aditivas. |
| Callees | `vanta_proxy::redact::{RedactConfig, Redactor}` (built-ins + `--pattern` opcional; `scan()` value-free), `vanta_proxy::capture::{turn_job, list_turns, WriteGuard}` (path de captura real), `vanta_proxy::envelope::{Envelope, EnvelopeConfig, EnvelopeMode}` (`Disarmed` + `seal/open`), `vanta_proxy::server::{AppState, router}` (flujo HTTP real con mock upstream, patrón `ver04_governance.rs`), `vanta_memory::core::{persona::persona_generator, scene::scene_index, record::l1_reader}` (semillas + lectura L1), `vantadb::{sdk::{Embedded, export_line_from_record, MemoryListOptions}, entity::EntityStore, storage::StorageEngine}` (store fjall on-disk), `vanta-cli` (subprocess: `delete --attest`, `certificate verify`). |
| Implicaciones | Cero cambios a código/producción: 0 símbolos públicos nuevos en el core, 0 cambios de wire, 0 cambios de config, 0 dependencias nuevas (`Cargo.toml` intacto: bin auto-descubierto en `src/bin/`; el test usa deps existentes). El test E2E es `#[ignore]` (requiere feature `vantadb/fjall` + store on-disk) → la suite default lo lista como ignorado, nunca lo salta en silencio. El bin de auditoría es un sumidero de solo-lectura (nunca imprime valores matcheados). Los paths de docs (`COMPARISON.md`, `PROXY.md`) son aditivos y quirúrgicos (no se renumeran secciones ajenas). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o regiones núcleo):** `vanta-proxy/src/{capture.rs (329L), redact.rs (:1-459), envelope.rs (:1-150), server.rs (:100-219, :610-684), session.rs (:22-139), auth.rs (:1-114), config.rs región features}, vanta-proxy/tests/{ver03_write_redact.rs (358L), ver04_governance.rs (330L), tool_loop.rs (:330-509)}, vanta-proxy/Cargo.toml (43L)`, `src/{attestation.rs (766L), cli.rs (:1-43, :466-478), crypto.rs (región from_env), config.rs (:1095-1154), backend.rs (:80-209), storage/engine/init.rs (:1-295), sdk/builder.rs (:225-249), sdk/api/memory.rs (:781-879), sdk/api/admin.rs (:115-144)}`, `docs/{api/PROXY.md (278L), api/CERTIFIED_DELETE.md (128L), api/MCP.md (:540-659), user/COMPARISON.md (153L)}`, `docs/dev/{workflow/RULES.md (218L), strategy/GO_TO_MARKET.md (:130-189), plans/2026-09-26-master-roadmap.md (:1125-1149), tasks/VER-04.md (formato)}`, `.github/{workflows/ci-examples.yml (146L), actions/rust-setup/action.yml (100L)}`, `scripts/validate-docs-coverage.ps1 (227L)`.
- **Referencias hacia dentro (imports):** nada importa a los artefactos nuevos (no existían). Los artefactos nuevos importan: `vanta_proxy::{redact, capture, envelope, server, config, session}`, `vantadb::{sdk, entity, storage}`, `vanta_memory::core::{persona, scene, record::l1_reader}` — todos `pub` ya consumidos por tests existentes (`ver03`/`ver04`/`tool_loop`).
- **Referencias entrantes:** `docs/user/COMPARISON.md` ← enlazado desde `docs/user/index.md` generado + README/docs; `docs/api/PROXY.md` ← enlazado desde `docs/api/index.md` + FIND-68 (checklist anti-drift: `rg -c "\.route\(" server.rs = 11` — NO se toca el router). El nuevo one-pager `docs/user/PRIVACY.md` quedará referenciado por `COMPARISON.md` + índice generado (`gen-index --write`).
- **Veredicto impacto:** 🟢 bajo — 100% aditivo (2 archivos nuevos de código, 2 de CI/script, 3 docs; 2 docs modificados en secciones nuevas). Sin cambios de API/wire/config; sin migración; sin hot paths. Revertible por borrado de archivos.

## Contrato

(verbatim del plan Task 43 — ley)

"one-pager del track (privacy/local-LLM: garantías exactas — qué se redacta, qué cubre el certificado, qué NO (unlearning) — con comandos) Y demo CI E2E verde que encadena: captura de PII sintética por el proxy → auditoría PII de store+índices+export = 0 en claro (script versionado, no test ad-hoc) → forget certificado (`delete --attest` + `certificate verify` exit 0) → audit de inyección consultable (`op:"injection"`) Y entrada en `COMPARISON.md` Y degradación sin key declarada (Disarmed) verificada en el demo o cubierta por test"

## Spec (SDD — artefactos nuevos; decisiones resueltas por evidencia)

> Gate D: el contrato del plan prescribe los 4 artefactos + Ruta vanta-docs (+ vanta-worker para demo/auditoría PII); stop conditions pre-autorizan el recorte (demo con engine/tests de integración si el release pesa; auditoría reusa patrones del Redactor o FIND). Sin pregunta al usuario: alcance fijado por el plan + wave F5.1.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Dónde vive la auditoría PII | (A) bin nuevo `vanta-pii-audit` en `vanta-proxy` (reusa los detectores del `Redactor`; corre sobre un store real; CARGO_BIN_EXE la hace invocable desde el test) / (B) solo assertions en el test (espejo, no artefacto) / (C) comando nuevo en `vanta-cli` (toca CLI del core + docs + paridad) | **A** — el contrato exige "script versionado, no test ad-hoc"; C es scope creep sobre el CLI del core | ✅ decidido-por-evidencia (plan :1135; `redact.rs:165-188` scan público; `api05_snapshot_auth.rs:157-172` precedente bin/test en el crate) |
| 2 | Qué escanea la auditoría | (A) bytes crudos de cada archivo del store (cubre store + índices + WAL + shred de una pasada, formato-agnóstico) + archivos extra (export v2 JSONL) / (B) solo vía SDK (no ve archivos) / (C) solo export | **A** — "store+índices+export = 0 en claro" verificable sobre el artefacto en disco; el test además corre el scan SDK por namespace (cinturón y tirantes) | ✅ decidido-por-evidencia (pre-mortem F1 plan :1140 "script sobre el store real") |
| 3 | Fail-open vs fail-closed de la auditoría | (A) archivo > cap (64 MiB) = `unscanned` + exit ≠0 (no auditado ≠ limpio) / (B) skip silencioso (precedente `Redactor::scan` fail-open 2 MiB de egress) / (C) streaming por chunks | **A** — una auditoría de privacidad no puede reportar "limpio" lo que no miró; el cap es configurable por archivo y el límite se declara | ✅ decidido-por-evidencia (pre-mortem F1; `redact.rs:165-168` fail-open egress NO es aceptable para auditoría) |
| 4 | Cómo corre el demo (encadenado) | (A) test E2E en `vanta-proxy` (HTTP real + mock upstream) que produce el store/export/audit + script que orquesta los pasos cross-crate con los bins reales (`vanta-cli delete --attest` / `certificate verify` / `vanta-pii-audit`) / (B) todo en un test (imposible: `CARGO_BIN_EXE_vanta-cli` no aplica cross-crate) / (C) todo en script con servidor real (necesita seeds+upstream reales) | **A** — "demo CI E2E" = script+workflow; el test cubre lo in-process, el script encadena los comandos de usuario reales con exit 0 | ✅ decidido-por-evidencia (plan :1131/:1135; `cli_tests.rs:1773-1778` CARGO_BIN_EXE por-crate) |
| 5 | Persistencia on-disk del demo | (A) `--features vantadb/fjall` + test `#[ignore]` (documentado; precedente live smoke) / (B) store InMemory (imposible: el CLI de otro proceso no lo ve) / (C) agregar feature `fjall` a `vanta-proxy` (cambio de build-graph ajeno a la tarea) | **A** — el test declara el requisito en el nombre de ignore + script; la suite default queda intacta | ✅ decidido-por-evidencia (`cargo tree -p vanta-proxy -e features -i vantadb` → solo encryption; `init.rs:274-279` registra Fjall feature-gated; `api05_snapshot_auth.rs:163-166`) |
| 6 | PII sintética del demo | (A) set de `ver03_write_redact.rs:20-21` (email + AWS key) + aws_secret + token `sk-` / (B) solo email / (C) PII real (PROHIBIDO) | **A** — cubre los 4 kinds built-in; sintética por construcción | ✅ decidido-por-evidencia (`redact.rs:65-99` kinds; `ver03_write_redact.rs:19-21`) |
| 7 | Consulta del injection audit | (A) `op:"injection"` en JSONL + filtro del contrato (Select-String/rg en script, select en el test) / (B) comando CLI nuevo | **A** — el contrato nombra el campo (`op:"injection"`); la consulta documentada (jq/rg) ya existe en `PROXY.md:182-193` | ✅ decidido-por-evidencia (plan :1135; `PROXY.md:171-193`; `ver04_governance.rs:262-296`) |
| 8 | Disarmed | (A) leg explícita en el demo (`Envelope::with_master(cfg, None)` → Disarmed + reason; captura → sin envelope y sin claro) + asserts de `ver03` intactos / (B) solo citar el test existente | **A** — el contrato pide "verificada en el demo **o** cubierta por test"; el demo la encadena (y el test existente queda como regresión) | ✅ decidido-por-evidencia (plan :1135; `ver03_write_redact.rs:279-313`) |
| 9 | One-pager: destino y forma | (A) `docs/user/PRIVACY.md` (kind `howto`; garantías + tabla cubierto/no cubierto + comandos; enlaces a PROXY/CERTIFIED_DELETE/MCP) / (B) sección en QUICKSTART / (C) `docs/dev/strategy/` | **A** — "one-pager destino `docs/user/` (a crear en DISCOVERY)" | ✅ decidido-por-evidencia (plan :1131; `documentation-skill` §2.1 kind→path) |
| 10 | Entrada COMPARISON | (A) sección nueva al final (§7) con tabla garantía→verificación + límites declarados, sin tocar §1-§6 (co-batch: ICP-01/03 también agregan) / (B) reescribir la nota "Capa pendiente" | **A** — quirúrgico para co-batch; la nota :17 se mantiene (la capa memory-as-a-service completa sigue pendiente de ICP-01/03) | ✅ decidido-por-evidencia (co-batch wave F5.1; plan :1135) |
| 11 | Workflow CI | (A) `.github/workflows/icp02-privacy-demo.yml` nuevo (patrón `ci-examples.yml`: paths scoped, concurrency, permissions read, checkout pin SHA, rust-setup con nextest, 1 job timeout, sin `continue-on-error`) / (B) job en `ci-rust.yml` | **A** — "demo CI: `.github/workflows/` (nuevo; patrón ci-examples.yml)" + RULES.md 1-7 | ✅ decidido-por-evidencia (plan :1131; `RULES.md` reglas 1-7; `ci-examples.yml`) |

## Diseño (explícito)

**1. `vanta-proxy/src/bin/vanta-pii-audit.rs` (nuevo — la auditoría PII de producto):**
- Uso: `vanta-pii-audit [--json] [--pattern <regex>]... <path>...` — cada path: archivo o directorio (recursivo, orden determinista, symlinks omitidos).
- Escanea **bytes crudos** de cada archivo con `Redactor` (`enabled: true`; built-ins email/aws_key/aws_secret/token + `--pattern` opcionales). Salida **value-free**: `{path, kind, offset}` — jamás el texto matcheado.
- Cap por archivo: 64 MiB (fail-closed) → entra a `unscanned` y el exit es ≠0 ("no auditado ≠ limpio").
- `--json`: `{"ok":bool,"audited_files":n,"audited_bytes":n,"findings":[…],"unscanned":[…]}`. Exit: 0 limpio · 1 findings/unscanned · 2 error de uso.
- Unit tests inline (detectores, dir recursivo, cap, value-free).

**2. `vanta-proxy/tests/icp02_privacy_demo.rs` (nuevo — el demo E2E in-process):**
- `#[ignore = "on-disk demo: run with --features vantadb/fjall --ignored (scripts/demo-privacy-e2e.ps1)"]`; dir `ICP02_DEMO_DIR` (default `<workspace>/target/icp02-demo`, limpiado al iniciar).
- Setup: store **Fjall on-disk** en `<demo>/db`; `ProxyConfig` con `[redact] enabled/Mask`, `[envelope] enabled` (+ `VANTADB_ENCRYPTION_KEY` de test = 64 hex), `[injection] audit_log_path = <demo>/injection-audit.jsonl`, writeback sin persist; `EntityStore` user seed (patrón `ver04_governance.rs:45-63`); persona+scene seed; `AppState::from_engine` + router en puerto loopback + mock upstream.
- **Captura con PII sintética por el proxy:** POST `/agent/space/v1/chat/completions` con `x-vanta-user-key` + `x-vanta-session: sess-icp02` y mensaje user con `EMAIL`, `AWS_KEY`, `AWS_SECRET`, `TOKEN`, `MARKER` (no-PII) → `writeback.flush()` → asserts: `proxy-turns` payload con `[REDACTED_*]` + `redacted: [kinds]` + `original_envelope`; L1 `l1/sess-icp02` con texto enmascarado; envelope abre con la key y devuelve el original; **0 en claro**.
- **Audit de inyección consultable:** las mismas requests inyectan persona/escena → `<demo>/injection-audit.jsonl` con filas `op=injection`, `outcome=ok`, `surface=proxy`; el test filtra como la consulta documentada y asserta metadata-only (sin contenido).
- **Disarmed:** `Envelope::with_master(entregar cfg activo, None)` → `mode() == Disarmed` + `disarm_reason().is_some()`; `turn_job` con ese guard → sin `original_envelope`, sin claro (leg del demo).
- **Auditoría PII:** scan SDK de TODOS los namespaces (0 claro) + `flush()`; export v2 JSONL de todos los records (`export_line_from_record`); corre el **bin** vía `CARGO_BIN_EXE_vanta-pii-audit` sobre `<demo>/db` + export → exit 0, JSON `ok=true, findings=[]`; **control negativo**: archivo scratch con PII → exit ≠0 con `kind=email` (auditoría no vacua).
- **Handoff:** `<demo>/handoff.json` `{db_dir, records:[{namespace,key}×2 (proxy-turns + l1)], export, injection_audit}` para el script.

**3. `scripts/demo-privacy-e2e.ps1` (nuevo — orquesta el E2E CI/local):**
1. `cargo nextest run -p vanta-proxy --features vantadb/fjall --test icp02_privacy_demo --ignored --no-capture`
2. por cada record del handoff: `cargo run -q -p vantadb --bin vanta-cli -- delete --db <db> --namespace <ns> --key <k> --attest --out <cert>` + `certificate verify --db <db> --file <cert> --json` → **exit 0 c/u**
3. auditoría post-forget: `cargo run -q -p vanta-proxy --features vantadb/fjall --bin vanta-pii-audit -- --json <db> <export>` → exit 0
4. consulta del audit: `Select-String '"op":"injection"'` sobre `<demo>/injection-audit.jsonl` → ≥1 fila (+ `outcome":"ok"`)
- `$ErrorActionPreference='Stop'` + chequeo explícito de `$LASTEXITCODE` en cada paso; cross-platform (pwsh 7 en CI ubuntu; 5.1+ en Windows local).

**4. `.github/workflows/icp02-privacy-demo.yml` (nuevo):** patrón `ci-examples.yml` — `on: push [main, develop] + pull_request [main]` con `paths` scoped (`vanta-proxy/**`, `src/attestation.rs`, `src/crypto.rs`, `src/sdk/**`, `src/cli_handlers/**`, `scripts/demo-privacy-e2e.ps1`, one-pager, el propio workflow) + `workflow_dispatch`; `concurrency` cancel-in-progress; `permissions: contents: read`; job único `timeout-minutes: 45`; checkout pin SHA `# v7.0.1`; `rust-setup` con `install-nextest: true`; step `pwsh -NoProfile -File scripts/demo-privacy-e2e.ps1`; sin `continue-on-error`.

**5. Docs:**
- `docs/user/PRIVACY.md` (nuevo one-pager): garantías exactas (qué se redacta/kinds; envelope AEAD por namespace + rotación + Disarmed; certificado VER-02 qué cubre y `out_of_scope` — **no unlearning**, no secure erase, backups; governance de inyección budget/ACL/audit metadata-only), tabla **cubierto / no cubierto**, comandos (config TOML, demo, auditoría, delete+verify, consulta jq), defaults honestos (todo opt-in).
- `docs/user/COMPARISON.md`: §7 nueva (tabla garantía→evidencia + límites + links) sin tocar §1-§6.
- `docs/api/PROXY.md`: subsección `### PII audit (`vanta-pii-audit`)` en §Redaction-on-write (comando + límites declarados: built-ins + `--pattern`, cap 64 MiB fail-closed, value-free).

## Invariantes de dominio (handoff — MUST)

- (1) **Cero cambios a comportamiento existente:** defaults opt-in intactos; suites `ver03_write_redact`/`ver04_governance`/`prx07_redact` verdes sin editar sus archivos; router (`server.rs`) sin tocar.
- (2) **La auditoría nunca imprime valores** — solo `path/kind/offset` (value-free por construcción).
- (3) **No auditado ≠ limpio:** cap excedido = `unscanned` + exit ≠0; jamás skip silencioso.
- (4) **PII sintética** en demo/test (nunca datos reales); claves de test efímeras (64 hex en env de proceso).
- (5) **No tocar** las regiones de co-batch F5.1 (`governance.rs`, `auto_recall.rs`, `vantadb-mcp/**`) ni Prohibidos (Backlog, perf-bench.yml, CONSTRAINTS.md, desktop/**, opencode.jsonc, plan).
- (6) **El demo respeta la causa raíz:** `delete --attest` + `certificate verify` corren como comandos de usuario reales (subprocess), con exit 0 verificable — no una simulación del flujo.
- (7) **Sin commit/push del worker** (LEAD); sin self-review (LEAD, P2-01).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** 0 — aditivo puro (2 archivos código nuevos, 0 deps nuevas, 0 `unsafe`, 0 hot paths). **Pago:** la garantía "0 PII en claro" pasa de tests por-pilar a auditoría de producto ejecutable sobre un store real + demo E2E CI (antes inexistentes). **Deuda declarada:** los `redact.patterns` custom del TOML del proxy se pasan a la auditoría vía `--pattern` (no se leen del config automáticamente); envelopes históricos rotados no se descifran en el demo (declarado en PROXY.md :124-128).

## Verificación

| Ítem | Comando → resultado obtenido |
|------|------------------------------|
| c1 one-pager | ✅ `docs/user/PRIVACY.md` publicado (garantías + cubierto/no cubierto + comandos) · `node scripts/docs/check-links.mjs` **exit 0** (0 rotos nuevos) · `check-docs.mjs` **all clear** · `gen-index.mjs --write` + `--check` **exit 0** (PRIVACY.md en `docs/user/index.md:89` + `llms.txt:192`) |
| c2 auditoría PII (script versionado) | ✅ bin `vanta-pii-audit` **7/7** unit (post-review: scan byte-oriented) · demo **1/1**: auditoría **exit 0** (`audited_files=44, non_utf8_files=38, findings=[]`) + **control negativo binario** (email + `0xFF` → exit 1, `kind=email`) + `[REDACTED_EMAIL]` persistido (prueba que el scan lee contenido real, no vacuo) — evidencia pre/post del Critical en el batch de abajo |
| c3 forget certificado | ✅ `pwsh scripts/demo-privacy-e2e.ps1`: `delete --attest` + `certificate verify` **exit 0 ×2** (`proxy-turns` + `l1/sess-icp02`; `ok:true`, `status:"purged"`, `residues_now:0`, integrity ok) — 2 corridas completas verdes |
| c4 audit de inyección consultable | ✅ `3 total, 3 ok` filas `"op":"injection"` (Select-String en el script + asserts del test: `outcome=ok`, `surface=proxy`, metadata-only, sin `PERSONA-MARKER`) |
| c5 entrada COMPARISON | ✅ §7 "Privacy & local LLMs" (tabla garantía→evidencia + límites declarados + links) — `check-links` verde |
| c6 Disarmed | ✅ leg del demo: `mode()==Disarmed` + `disarm_reason().is_some()` + sin `original_envelope` + 0 claro · `ver03_write_redact` **7/7** |
| Gates | rustfmt `--check` ✅ · clippy `-p vanta-proxy --all-targets --features vantadb/fjall -- -D warnings` **exit 0** ✅ (post-fix) · `ver03_write_redact`+`ver04_governance` **11/11** ✅ (post-fix) · script E2E **PASSED ×3** (última post-fix) ✅ · markdownlint (4 docs) **0 issues** ✅ · `validate-docs-coverage` **0 gaps** ✅ · actionlint workflow **exit 0** ✅ · PS parse del script OK ✅ |

**Nota de gates:** la suite completa default de `-p vanta-proxy` no se pudo completar por **disco lleno del runner** (C: 0 bytes; `LNK1180` al linkear) — es entorno, no código. Se corrió scoped (instrucción de wave: `ver03`/`ver04`) + `--all-targets` de clippy (que compila todos los targets del paquete, incluida la suite, sin linkear los .exe) + demo/script E2E. El demo `#[ignore]` aparece listado como ignorado en la suite default (nunca skip silencioso: el nombre de ignore documenta el cómo correrlo).

### Batch de review (2026-09-30 — ❌ REQUEST CHANGES → fixes)

| Finding | Fix | Evidencia (comando → resultado) |
|---------|-----|---------------------------------|
| **CRITICAL [1]** — el auditor contaba como "auditado" archivos no-UTF-8 sin escanearlos (fail-open: `redact.rs` `scan` devuelve vacío si no es UTF-8 + el bin los contaba sin `unscanned`). 38/44 archivos del store real (256.1 MB) quedaban sin escanear; repro email + `0xFF` → `ok:true` exit 0 | (1) `Redactor::scan_bytes` **byte-oriented**: detectores refactorizados a `&[u8]` (compartidos por `scan`/`scan_bytes`, sin drift) + customs `regex::bytes`; (2) el bin usa `scan_bytes` y reporta cobertura explícita `non_utf8_files`; fail-closed intacto (cap/IO/symlink → `unscanned` ≠ clean); (3) tests: `redact.rs` +3, bin +2, demo negativo **binario** (`leak.bin` con `0xFF`) + assert `non_utf8_files ≥ 1` en el store real; docs PRIVACY/PROXY precisan "binary-safe" | **ANTES:** `vanta-pii-audit --json target/icp02-repro` → `{"ok":true,"audited_files":1,"findings":[],"unscanned":[]}` **exit 0** (email en claro no detectado) · **DESPUÉS:** `{"ok":false,"findings":[{"kind":"email","offset":7,"path":"…leak.bin"}],"non_utf8_files":1}` **exit 1** · **store real (cobertura plena):** `{"ok":true,"audited_files":44,"audited_bytes":268559974,"non_utf8_files":38,"findings":[]}` (los 38 binarios —incl. `0.jnl`— ahora escaneados) · redact lib **12/12** (wire `scan` no-UTF-8 sigue fail-open: `non_utf8_fails_open` intacto) · bin unit **7/7** · demo **1/1** · ver03+ver04 **11/11** |
| **REQUIRED [2]** — `COMPARISON.md` con dos `## 7.` (colisión co-batch ICP-01/ICP-02) | La sección de ICP-02 → `## 8. Privacy & local LLMs — the guarantee layer (ICP-02)`; sin cross-refs con anchor a la §7 vieja (`rg 'COMPARISON.md#'` = 0 hits) | `Select-String '^## '` → §1–§6 + §7 (ICP-01) + §8 (ICP-02) **sin duplicados** · `check-links` exit 0 |
| **OPTIONAL** — `PRIVACY.md:17-20` "Everything here is opt-in" inexacto | Precisado: `[redact]`/`[envelope]` off por defecto; ACL/audit de inyección opt-in; **excepción default-on documentada: `[injection] max_tokens = 2000`** (`0` desactiva) | `docs/user/PRIVACY.md:17-21` (callout reescrito) |
| **OPTIONAL** — `COMPARISON.md` "four verifiable guarantees" vs 5 filas | Reescrito: "four guarantees — plus the audit that proves they hold on a real store" | `docs/user/COMPARISON.md:179-181` |
| **NIT** — `vanta.wal*.corrupt` en el store demo | Documentado: artefactos de **cuarentena/salvage del WAL** del engine (4 archivos ≤1.7 KB, pre-existentes, ajenos a ICP-02; el audit los escanea byte-wise y no contienen PII en claro) | `Get-ChildItem target/icp02-demo/db -Recurse -Filter *corrupt*` → 4 archivos; audit del store `findings:[]` |
| Re-verify | bin unit **7/7** · redact **12/12** · demo **1/1** (42.5s, audit byte-wise ×2) · ver03+ver04 **11/11** · `scripts/demo-privacy-e2e.ps1` **PASSED** (delete+verify exit 0 ×2, post-forget audit `files=44 bytes=268559974` cleanup, `op=injection` 3/3) · rustfmt ✅ · clippy `--all-targets --features vantadb/fjall -D warnings` **exit 0** ✅ · check-links/check-docs/gen-index **0** ✅ · markdownlint **0 issues** ✅ | ✅ |

**Nota de entorno (batch):** un fallo transitorio de caché incremental (`vanta_proxy-*/…o` no encontrado, os error 2) en el primer intento de ver03/04 se resolvió limpiando `target/debug/incremental/vanta_proxy-*` y re-corriendo (11/11) — sin cambios de código.

## Cierre (handoff LEAD)

- **Commit (LEAD):** `feat: ICP-02 — track privacidad: one-pager + demo E2E + auditoría PII` — archivos: `vanta-proxy/src/redact.rs` (`scan_bytes` binary-safe) · `vanta-proxy/src/bin/vanta-pii-audit.rs` · `vanta-proxy/tests/icp02_privacy_demo.rs` · `scripts/demo-privacy-e2e.ps1` · `.github/workflows/icp02-privacy-demo.yml` · `docs/user/PRIVACY.md` · `docs/user/COMPARISON.md` · `docs/api/PROXY.md` · índices generados (`docs/index.md`, `docs/user/index.md`, `llms.txt`) · este task file. **NO incluir** el plan file (LEAD) ni artefactos de otras waves (`vanta-proxy/src/memory_tools.rs`, `evals/**`, etc. = ICP-01/VER-08).
- **Review P2-01 (LEAD):** fresh (diff `docs/api/**` ⇒ tier adversarial por tabla P2-01; el resto fast).
- **Observaciones:**
  - `validate_scope` (advisory) no reconoce los 4 artefactos nuevos (computa blast radius desde "Archivos clave"; ver §Re-baselines) — procedido por contrato verbatim; el LEAD decide si toca el parser de scope.
  - `campaign_verify_cmd`/`campaign_budget_status` con 2 planes activos fallan por ambigüedad (FIND-193) → verificación registrada por shell + esta recitation.
  - Disco del runner: C: llegó a 0 bytes (target/ = 82.5 GB) durante la suite completa — se liberó `target/session-icp02/debug/incremental` + `cargo clean -p vanta-proxy` y se corrió scoped. El LEAD puede querer limpiar `target/` antes de la próxima wave.
  - Índices regenerados absorben drift de co-batch (filas ICP-01/VER-08 + refresh MKT-18f) — inherente a `gen-index` (fuente única por frontmatter).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato ✅ cláusula por cláusula (c1-c6 en §Verificación) + rustfmt/clippy/nextest scoped + gates docs |
| **Commit** | LEAD (el worker NO commitea); conventional commit `feat: ICP-02 — …` |
| **Release** | N/A local (release-plz post-push, lane owner) |

## Herramientas necesarias
- Terminal cargo (nextest scoped `-p vanta-proxy`, clippy, fmt) · codegraph (blast radius — hecho)
- `node scripts/docs/{check-links,check-docs,gen-index}.mjs` · `scripts/validate-docs-coverage.ps1` · pwsh (script + actionlint)
- FIND-173/177 vigentes (prohibiciones de co-batch)

**Skills cargadas (SDP v3):** `security-and-hardening` (auditoría PII value-free, trust boundary, LLM08), `documentation-skill` (docs/user + frontmatter + gates), `source-driven-development` (APIs verificadas en código real, 0 deps nuevas), `doubt-driven-development` (garantías de privacidad — review adversarial delegado al LEAD), `test-driven-development` + `systematic-debugging` + `documentation-and-adrs` + `api-and-interface-design` + `ci-cd-and-automation` + `git-workflow-and-versioning` (pins del SDP). Base fija `campaign-executor`/`progreso` vía MCP.

## Steps (atomics — PLAN → ACT → VERIFY)

- [x] **S1 (auditoría PII — bin):** `vanta-proxy/src/bin/vanta-pii-audit.rs` (scan bytes crudos con Redactor; `--json`; `--pattern`; cap 64 MiB fail-closed; value-free; exit 0/1/2) + unit tests inline. Verify: `cargo nextest run -p vanta-proxy --bin vanta-pii-audit` → ✅ **5/5** (fix de separador Windows en el assert de orden).
- [x] **S2 (demo E2E in-process):** `vanta-proxy/tests/icp02_privacy_demo.rs` (HTTP real + mock upstream; captura PII → asserts; injection audit; Disarmed; export; auditoría bin + control negativo; handoff.json). Verify: `cargo nextest run -p vanta-proxy --features vantadb/fjall --test icp02_privacy_demo --run-ignored ignored-only` → ✅ **1/1** (2 corridas; fix real: shutdown graceful + close antes del scan de archivos — lock files Windows).
- [x] **S3 (script):** `scripts/demo-privacy-e2e.ps1` (nextest → CLI delete/verify ×2 → auditoría post-forget → consulta audit). Verify: `pwsh -NoProfile -File scripts/demo-privacy-e2e.ps1` → ✅ **PASSED ×2** (exit 0; `certificate verify ok:true` ×2; audit `files=44 bytes=268559977`; injection `3 total, 3 ok`).
- [x] **S4 (workflow):** `.github/workflows/icp02-privacy-demo.yml`. Verify: ✅ `actionlint` **exit 0** + revisión contra RULES.md 1-7 (paths scoped, concurrency, permissions read, checkout SHA pin, timeout 45, sin continue-on-error) + PS parse OK.
- [x] **S5 (docs + gates):** `docs/user/PRIVACY.md` + `COMPARISON.md` §7 + `PROXY.md` §PII audit; `check-links` ✅ / `check-docs` ✅ / `gen-index --write` + `--check` ✅ / `validate-docs-coverage` 0 gaps ✅ / markdownlint 0 issues ✅; verify scoped: rustfmt ✅ · clippy `--all-targets --features vantadb/fjall -D warnings` ✅ · ver03+ver04 **11/11** ✅.

## Investigation Notes

### Código — superficies verificadas (2026-09-29)
- **VER-03:** `Redactor::scan(&[u8]) -> Vec<Finding>` público y value-free (`redact.rs:165-188`, `Finding{kind,start,end}` :102-107, `RedactKind::as_str` :78-86; built-ins email/AKIA/aws_secret/token `:304-447`). `turn_job` = path único de captura (proxy-turns + l1) con `WriteGuard{redactor,envelope}` (`capture.rs:74-179`). `Envelope::with_master/mode/disarm_reason/seal/open` (`envelope.rs:117-150+`); Disarmed = key ausente → nunca claro.
- **VER-02:** `Embedded::delete_certified` (`sdk/api/memory.rs:837-853`) + `verify_purge_certificate` (`:871-879`) públicos; `vanta-cli delete --attest` + `certificate verify --file` (`cli.rs:467-478`; bin del crate `vantadb` → cross-crate para el test de proxy).
- **VER-04:** audit JSONL `[{op:"injection", namespace, key, outcome, reason:"surface=proxy;…"}]` (`PROXY.md:171-193`; `ver04_governance.rs:262-296`); config `[injection] audit_log_path`.
- **Flujo HTTP de captura:** `server.rs:640-667` (`capture_turn` ← headers de sesión `session.rs:25-32` + último mensaje user) tras la respuesta; `AppState.writeback` es `Arc<WriteBack>` (flush en tests: `capture.rs:289`).
- **On-disk:** `BackendKind::Fjall` default feature-gated (`backend.rs:105-113`; `backends/registry.rs:30-42`); `-p vanta-proxy` NO activa `fjall` → el demo necesita `--features vantadb/fjall` (precedente `api05_snapshot_auth.rs:163-166`).
- **Gate docs:** `validate-docs-coverage` chequea SDK/config/error/CLI/MCP/skills-mirror — el bin nuevo no entra en su radar; los docs de `/docs/` los cubren `check-links`/`check-docs`/`gen-index`.
- **CI:** `ci-examples.yml` (patrón de triggers/pins/concurrency), `rust-setup` action (`install-nextest: true`), pwsh en ubuntu ya usado (`gate-docs.yml:98`).

### Re-baselines y hallazgos de implementación (2026-09-29)

- **Flag nextest:** la selección de tests ignorados es `--run-ignored ignored-only` (no `--ignored`, que es de `cargo test`; nextest 0.9.133 lo rechaza). Script + task file actualizados.
- **Windows: el store abierto byte-bloquea sus lock files** (`lock`, `.vanta.lock`) → la auditoría los reportaba `unscanned` (fail-closed correcto, exit 1). Resolución: el demo hace `flush()` → shutdown del server (graceful) → `db.close()` + drop de handles → **recién entonces** corre la auditoría de archivos. Es además la secuencia recomendada de producto (documentada en PRIVACY.md/PROXY.md).
- **`validate_scope` (advisory) no reconoce los artefactos nuevos:** computa el blast radius desde "Archivos clave" del plan, que describe el demo/auditoría por directorio (`docs/user/`, `.github/workflows/` nuevo) sin listar los paths nuevos (`vanta-proxy/src/bin/vanta-pii-audit.rs`, `vanta-proxy/tests/icp02_privacy_demo.rs`, `scripts/demo-privacy-e2e.ps1`, `.github/workflows/icp02-privacy-demo.yml`). Se procede por contrato verbatim + instrucción de wave; registrado aquí para el orquestador/LEAD.
- **Índices regenerados con drift de co-batch:** `gen-index --write` estaba pendiente (`docs/index.md` stale) — el diff absorbe también las filas ICP-01/VER-08 + refresh de MKT-18f (docs de otros agentes de la wave). Inherente al generador (fuente única por frontmatter); no editable a mano (banner GENERATED).
- **Interpretación fina del contrato (c3):** `delete --attest` + `certificate verify` corren como **comandos reales del CLI** en el script (subprocess, exit 0 ×2: `proxy-turns` + `l1`), porque `CARGO_BIN_EXE_vanta-cli` no existe en el crate `vanta-proxy` (bin de `vantadb`). El test in-process cubre captura→auditoría→Disarmed→handoff; el script encadena el resto.
- **Corrida final (post-format/clippy-fix) — PASSED:** demo 1/1 (10.05s) + `certificate verify ok:true` ×2 (`proxy-turns`/`l1/sess-icp02`, `residues_now:0`, integrity ok) + auditoría post-forget `files=44 bytes=268559977` (0 findings) + `op=injection` `3 total, 3 ok` → `== ICP-02 demo PASSED ==`. Evidencia completa en el output del shell (2 corridas idénticas).

=== RECITATION ICP-02 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: ICP-02 — track local-LLM/privacidad: one-pager + demo E2E + auditoría PII
Estado: in-progress (implementación + verificación + **batch de review 2026-09-30 aplicado**; commit + re-review P2-01 = LEAD)
Última acción: batch ❌→fixes: (CRITICAL) scanner byte-oriented `Redactor::scan_bytes` + bin con `non_utf8_files` + fail-closed + 5 tests nuevos (redact 12/12 · bin 7/7) + control negativo binario; repro ANTES ok:true/exit 0 → DESPUÉS `kind:email`/exit 1; store real `non_utf8_files:38` escaneados; (REQUIRED) COMPARISON §7→§8 sin duplicados; (OPTIONAL) callout opt-in preciso + wording "four guarantees + audit"; (NIT) `vanta.wal*.corrupt` documentado. Re-verify: demo 1/1 · ver03+04 11/11 · script PASSED · clippy/fmt 0 · docs gates 0
Resultado: OK (pendiente solo commit + review fresh del LEAD)
Próxima acción: LEAD — re-review P2-01 fresh del delta + commit `feat: ICP-02 — track privacidad: one-pager + demo E2E + auditoría PII` (NO push)
Contrato: cláusulas c1-c6 ✅ + batch de review cerrado (evidencia §Verificación + §Batch de review)
Próxima tarea si completa: ICP-03
=== END RECITATION ===
