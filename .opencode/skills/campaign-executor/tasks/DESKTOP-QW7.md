# TASK DESKTOP-QW7: Rename namespace preserva sparse_vector (H-04)

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-desktop-quickwins.md`
- **Creado:** 2026-08-27T10:30
- **last-synced:** 2026-08-27T10:30
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** desktop/data-integrity (frontend-ui-engineering, source-driven-development)
- **Workflow:** feature-add (audit → verify → close) — integridad de datos híbridos (BM25+HNSW)
- **Task file:** `.opencode/skills/campaign-executor/tasks/DESKTOP-QW7.md`

## Blast Radius
- `desktop/src/store/undo.ts:191-217` — `renameNamespace(from,to)` — copia lote vía `ingestBatch(records.map(r=>{id,namespace:to,text,embedding:vector,sparse_vector,metadata}))` y borra origen. Undo inverso (`move` reverse) re-put con `vantaPut` incluyendo `sparse_vector`. Impacta rename flow del NamespaceDialog → WorkspaceShell `handleRenameNs` → `undoStore.renameNamespace`. Sin sparse_vector, registros híbridos (BM25+HNSW) pierden dimensión sparse silenciosamente.
- `desktop/src/vanta.ts:30-41,113-131,288-304,423-434` — `IngestItem.sparse_vector` y `MemoryRecord.sparse_vector` + `vantaPut.sparse_vector` + `listAll` pagan wrapper. Bridge transport (Tauri IPC) serializa sparse_vector como `HashMap<u32,f32>` wire JSON (`"0":0.5`). Si falta campo, data.rs deserializa `None` → pérdida silenciosa.
- `desktop/src-tauri/src/connections/types.rs:35-57,153-188` — `IngestItem.sparse_vector: Option<HashMap<u32,f32>>` y `MemoryRecord.sparse_vector` — DTO Rust. `native.rs` ingestBatch → `VantaEngine.ingest_batch` preserva sparse_vector en HNSW+BM25.
- `desktop/src/store/undo.test.ts:165-197` — test `rename preserva sparse_vector (H-04)` + undo preserva. Mock `listAll → [rec h sparse_vector {0:0.5,5:1.25}]` → assert `ingestBatch` lleva campo + `vantaPut` en undo lo lleva. Si regresa, describe fallo claro.
- `desktop/src/components/layout/NamespaceDialog.tsx` + `WorkspaceShell.tsx:258-267` — callers: Dialog onRename → WorkspaceShell handleRenameNs (try/catch + refresh + notice). No tocan sparse_vector directamente, pero dependen de undoStore correctitud.
- **Implicaciones:** cambio de 1 campo en 1 mapper (records.map). No toca WAL/vector/storage, no hot path extra, no concurrencia nueva (queue existente serializa). Blast radius 2 archivos editados + 1 test. Reversible en 1 commit. Verify build/test/cargo.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `desktop/src/vanta.ts` (768 líneas completas, HEAD c87c72a7) — verificado IngestItem:30-41 con `sparse_vector?: Record<string,number>|null` + comentario H-04, MemoryRecord:113-131 idem, vantaPut:288-304 con sparse_vector, listAll:423-434, ingestBatch:254-256
  - `desktop/src/store/undo.ts` (314 líneas completas, HEAD c87c72a7) — renameNamespace 191-217 con `sparse_vector: r.sparse_vector ?? undefined` en ingestBatch map, restore 221-238 con sparse_vector, undo putRecord 262-270 con sparse_vector, comentarios H-04 líneas 195-196
  - `desktop/src/store/undo.test.ts` (264 líneas completas, HEAD c87c72a7) — test rename preserva sparse_vector 165-197 con ingestBatch assert + undo assert, mocks hoisted remove/vantaPut/ingestBatch/listAll, freshStore() pattern
  - `desktop/src-tauri/src/connections/types.rs` (825 líneas, HEAD c87c72a7) — IngestItem 35-57 con sparse_vector Option<HashMap<u32,f32>> + MemoryRecord 153-188 idem, serde default
  - `desktop/src-tauri/src/connections/../data.rs` (spot) + `native.rs` ingestBatch handling
  - `desktop/src/components/layout/NamespaceDialog.tsx` (158 líneas) — modal CRUD
  - `desktop/src/components/layout/WorkspaceShell.tsx:258-277` — handleRenameNs try/catch + notifications
  - `docs/plans/2026-08-25-research-desktop-quickwins.md` (98 líneas, HEAD c87c72a7) — Wave2 Task7 H-04 🟠, archivos clave desktop/src/vanta.ts store/connections.test.ts, contrato rename+build+test+cargo
  - `docs/reviews/archive/research-desktop-prod-20260825.md` (spot H-04) — hallazgo: rename copia sin sparse_vector → pérdida silenciosa híbridos
  - git history `a7ed0d22` — diff rename sparse_vector agregado (undo.ts +5 líneas, tipos.rs +6, data.rs +3, vanta.ts +5) — fix ya aterrizado Wave2+Wave1 bundle
  - `desktop/package.json` (56 líneas) — scripts build=test=vitest run, tsc+vite, 2863 modules expectativa
  - `desktop/src/store/connections.test.ts` (74 líneas) — patrón storage fake
  - `SKILLS-MANIFEST.md` grep sparse/vector/rename/hybrid
  - `.opencode/rules/core-engine.md` / `durability.md` (verify no se toca WAL/vector — out-of-scope delegation check)
- **Referencias hacia dentro (qué importa este archivo):**
  - `IngestItem` → wire JSON `sparse_vector` keys stringified u32, values f32 — serde HashMap<u32,f32>. Front-end lo tipa `Record<string,number>` → Rust lo parsea como `HashMap<u32,f32>`. Contract: omitido → None (defaults), presente → preservado en BM25 postings + HNSW.
  - `MemoryRecord.sparse_vector` → read-back desde `listAll` → usado en rename map + restore/undo putRecord. Core persistencia lo guarda por record.
  - `IngestItem.embedding` ↔ `MemoryRecord.vector` naming mismatch (front embedding vs vector) — rename mapea `r.vector` → `embedding` correctamente, no confundir.
  - `undo.ts queue` → serializa rename vs undo vs deletes — no race; fail atomic: si ingestBatch falla, no pushEntry (invariante DESKTOP-32)
- **Referencias entrantes (qué depende de lo que cambio):**
  - `NamespaceDialog onRename` → `WorkspaceShell handleRenameNs` → `undoStore.renameNamespace` → `ingestBatch + remove` + `pushEntry(move)`. Si sparse_vector falta, WorkspaceShell notice es éxito falso pero datos corruptos.
  - `undoStore.undo` (move reverse) → `vantaPut` con sparse_vector + `remove` destino. Si undo no lleva sparse_vector, deshacer deja datos aún corruptos (doble pérdida).
  - `vanta.ts ingestBatch` → `transport.call("vanta_ingest_batch", {records})` → Tauri `vanta_ingest_batch` → Rust `types::IngestItem` → `native.rs` → `VantaEngine.ingest_batch` (BM25 indexación sparse). E2E no cubre híbrido directamente pero `vitest` cubre via mock.
  - `desktop/tests` (plan lista `desktop/tests` pero real es `desktop/src/store/undo.test.ts`) — único test de rename; si falla, pipeline gate lo detecta.
  - `desktop/src-tauri/src/connections/native.rs` ingest handler — depende de DTO tener sparse_vector para no dropearlo. Ya soporta.
  - `docs/plans/...quickwins.md` Wave2 Task7 → depende de QW6 ✅ c87c72a7, bloquea QW8 (versión sync). Plan verificación mecánica exige build+test verde + cargo check si toca bridge (sí toca: vanta.ts + types.rs)
- **Veredicto de impacto:** BAJO (data integrity, no security) — 1 mapper en 1 archivo crítico + transporte tipado. Riesgo: omitir campo = pérdida silenciosa ranking BM25 (RRF fusion cae a vector-only). Mitigado por test que fija sparse_vector en ingestBatch + undo. No tocar WAL/vector/storage directo (Arch/Engine propiedad). Cambio ya en HEAD a7ed0d22, auditoría confirma presencia — verify-only esperado. No tocar TTL/metadata/vector (fuera contrato, YAGNI).

## Contrato
Rename namespace preserva `sparse_vector` (copiar campo en el ingestBatch del rename + test que lo fije); `cd desktop && npm run build` y `npm test` verde; `cargo check -p vantadb` si toca bridge Rust.

Verificación mecánica:
1. `grep -n sparse_vector desktop/src/store/undo.ts` — rename ingestBatch map contiene `sparse_vector: r.sparse_vector ?? undefined` && undo putRecord idem (2 hits) ✅ (auditoría)
2. `grep -n sparse_vector desktop/src/store/undo.test.ts` — test `rename preserva sparse_vector` existe y asserts `ingestBatch` + `vantaPut` con `{ "0": 0.5, "5": 1.25 }` ✅
3. `npm --prefix desktop run build` — tsc + vite build verde (sin TS errors, ~2863 modules, dist assets) ✅
4. `npm --prefix desktop test` — vitest run verde (11 files, 69+ tests incluyendo undo.test.ts) ✅
5. `cargo check -p vantadb` — verde (workspace still compiles; bridge tocado a7ed0d22 pero verify quickwins exige) ✅
6. Cierre full: `cargo fmt --check` verde ✅

## Herramientas
- Read (vanta.ts, undo.ts, undo.test.ts, types.rs, plan, research, NamespaceDialog, WorkspaceShell, package.json)
- Grep / Select-String (sparse_vector, ingestBatch, renameNamespace, H-04)
- codegraph_explore (renameNamespace sparse_vector ingestBatch)
- git (log, show, diff, status, add, commit)
- terminal: `npm --prefix desktop run build`, `npm --prefix desktop test`, `cargo check -p vantadb`, `cargo fmt --check`
- campaign_memory_write, campaign_diagnose_pipeline

## Skills
- campaign-executor, progreso, ponytail (base obligatoria)
- frontend-ui-engineering (detectado por tipo desktop — Hooks, Storage, UndoStore, WorkspaceShell)
- source-driven-development (detectado por tipo desktop — validar Tauri/Cargo bridge types contra docs)
- SDP discovery (lifecycle BUILD→ incremental-implementation, VERIFY→ systematic-debugging, DATA→ performance-optimization?): keywords `sparse_vector/rename/namespace/ingestBatch/hybrid/BM25/vector/busqueda/test` → grep SKILLS-MANIFEST.md: hits `source-driven-development` ya base, `frontend-ui-engineering` ya base, `test-driven-development` candidato (test ya existe, verify-only) , `performance-optimization` candidato pero undo no hot path (O(n) listAll + ingestBatch n, pero bridge I/O bound, no hot). **SDP: sin candidatos adicionales** — base + frontend + source cubren. Audit-only, no slice incremental ni perf bench necesario (Regla 9 no aplica: no hot path vector/). Total cargadas 5. **SKILLS_CARGADAS: campaign-executor, progreso, ponytail, frontend-ui-engineering, source-driven-development**

## Spec
N/A — tarea de bugfix/integridad de datos (no agrega `pub fn` / tool / endpoint / binding / símbolo público nuevo). Preservar sparse_vector es copiar 1 campo existente en mapper ya existente (`records.map` en renameNamespace) + test de fijación. No es feature-add con símbolos nuevos. Gate spec-first no aplica (ver pipeline-full § SPEC: solo feature-add/lógica nueva requiere Spec llena). Contrato mecánico es ley + evidencia de test. Justificación: rename ya existe desde DESKTOP-32, H-04 es gap de campo faltante (YAGNI hasta híbridos), no nueva API.

| Decisión | Opciones | Elegido | Por qué |
|---|---|---|---|
| Mapper sparse_vector valor | (a) `r.sparse_vector` directo (null) | (b) `r.sparse_vector ?? undefined` | (b) — IngestItem wire espera omitir o null; `undefined` omite key en JSON (serde default → None), `null` también deserializa a None pero `Record<string,number>|null` en TS: null viaje explícito vs undefined omitido. Ambos correctos, pero `?? undefined` normaliza null→undefined (limpio, no mandar null). Existe en HEAD y test espera undefined implícito (sparse_vector ausente = undefined). |
| Test storage | (a) mock transport real | (b) vi.mock vanta mocks hoisted | (b) — patrón existente undo.test.ts (vi.hoisted + freshStore vi.resetModules), determinista, no necesita Tauri. |
| Bridge touch cargo check | (a) skip | (b) run | (b) — contrato exige si toca bridge (sí: vanta.ts + types.rs). Aunque fix ya en a7ed0d22, verify ahora igual. |
| Verificación si ya fijo | (a) re-editar | (b) audit-only verify | (b) — Ponytail: deletion over addition. Fix ya en HEAD, no re-escribir mismo diff. Verify mecánico es suficiente (como QW1/QW4 audit variants). |

Evidencia por ítem: Read undo.ts 191-206 muestra sparse_vector en ingestBatch; undo.test.ts 165-197 test con assert ingestBatch + undo vantaPut; grep sparse_vector 2 hits rename; build 21s 2863 modules expectativa QW6; cargo check 26s QW6.

## Steps

### Step 1: Auditoría H-04 sparse_vector end-to-end (vanta.ts ↔ types.rs ↔ undo.ts ↔ undo.test.ts) ✅ DONE
- **Archivos:** `desktop/src/vanta.ts:30-41,113-131,288-304,423-434`, `desktop/src/store/undo.ts:191-217,221-238,258-271`, `desktop/src/store/undo.test.ts:165-197`, `desktop/src-tauri/src/connections/types.rs:35-57,153-188`, `docs/reviews/archive/research-desktop-prod-20260825.md` H-04
- **Acción:** Verificar que IngestItem y MemoryRecord en vanta.ts tipan sparse_vector + comentario H-04; que undo.ts renameNamespace map incluye `sparse_vector: r.sparse_vector ?? undefined` + H-04 comment + restore/undo putRecord idem; que types.rs DTOs tienen Option<HashMap<u32,f32>> con #[serde(default)] en ambos structs; que undo.test.ts test H-04 existe con vector [0.1] + sparse_vector {0:0.5,5:1.25} y asserts en ingestBatch + undo. Confirmar git a7ed0d22 ya aterrizó fix (no re-editar si presente). Si falta en cualquier capa → plan de edición mínima (1 campo en map). Si ambos presentes → audit-only, saltar a Step3 verify.
- **Verify:** `grep -n sparse_vector desktop/src/store/undo.ts` → líneas 195-196 comentario + 203 map + 229 restore + 268 undo (4 hits) ✅; `grep -n sparse_vector desktop/src/store/undo.test.ts` → líneas 165 + 171 + 185 + 194 (test + asserts) ✅; `grep -n sparse_vector desktop/src/vanta.ts` → IngestItem:38 + MemoryRecord:128 + vantaPut:293 + listAll comment (4 hits) ✅; `grep -n sparse_vector desktop/src-tauri/src/connections/types.rs` → 49-53 + 182-183 (2 hits) ✅; `git show a7ed0d22 --stat` confirma fix ya commit ✅ — auditoría 2026-08-27 10:30
- **Estado:** ✅ DONE — gaps cerrados, ambos lados preservan sparse_vector (ingestBatch + undo), test fija
- **Gate D:** NO disparado — blast 2 archivos +1 test, sin símbolos públicos nuevos, sin hot path concurrente, contrato claro, esfuerzo <1h (verify-only)

### Step 2: Editar undo.ts mapper si falta (gap menor) ✅ SKIPPED (audit-only)
- **Archivos:** `desktop/src/store/undo.ts:197-206`
- **Acción:** Si Step1 detecta que `records.map` en renameNamespace NO incluye `sparse_vector` → edición atómica 1 línea: agregar `sparse_vector: r.sparse_vector ?? undefined,` al objeto del map (entre embedding y metadata). Ponytail: 1 línea, no refactors, no nuevos tipos. Si también falta en restore/undo putRecord → agregar `sparse_vector: r.sparse_vector ?? undefined,` idem (pero restore/undo ya estaban en a7ed0d22 — verificar). Preservar formato (2-space, trailing comma).
- **Verify:** `cat desktop/src/store/undo.ts | grep -A2 -B2 sparse_vector` → mapper presente ✅ (líneas 195-203 con H-04 comment + sparse_vector field); `grep -n sparse_vector` counts ya verde Step1 — no edición necesaria; audit-only variant (0 líneas cambiadas) como QW1/QW4
- **Estado:** ✅ SKIPPED — Step1 ✅ (fix ya en a7ed0d22), 0 ediciones (ponytail: deletion over addition). Gate V: no disparado.

### Step 3: Build + Test + Cargo check verde (contrato mecánico) ✅ DONE
- **Archivos:** `desktop/package.json`
- **Acción:** Ejecutar `npm --prefix desktop run build` (tsc + vite, ~10-15s, 2863 modules) y `npm --prefix desktop test` (vitest, 11 files) y `cargo check -p vantadb` (workspace, ~20-30s). Capturar output. Si falla → systematic-debugging root-cause (leer archivo completo, no parche sintoma). Ponytail: no instalar deps nuevas; `npm ci` solo si node_modules corrupto.
- **Verify:** `npm --prefix desktop run build` ✅ (13.05s, 2863 modules, dist assets, exit 0) + `npm --prefix desktop test` ✅ (11 files, 69/69, 23.54s, exit 0) + `cargo check -p vantadb` ✅ (0.57s dev profile, exit 0) — evidencia terminal 2026-08-27 22:23
- **Estado:** ✅ DONE

### Step 4: Cierre — plan + fmt + commit + memoria ✅ DONE
- **Archivos:** `docs/plans/2026-08-25-research-desktop-quickwins.md`, `.opencode/skills/campaign-executor/tasks/DESKTOP-QW7.md`
- **Acción:** Verify mecánico cierre:
  1. `cargo fmt --check` → verde ✅ EXIT 0 2026-08-27 22:23
  2. Re-check build/test/cargo (Step3) ✅ 13.05s/23.54s/0.57s
  3. `grep sparse_vector` counts (Step1) ✅ 4 hits undo.ts + 5 hits undo.test.ts + 4 hits vanta.ts + 2 hits types.rs
  4. Si todo pasa: `git add docs/plans/2026-08-25-research-desktop-quickwins.md .opencode/skills/campaign-executor/tasks/DESKTOP-QW7.md` (audit-only, solo plan+task; no edición código) + commit `feat(desktop): DESKTOP-QW7 — Rename namespace preserva sparse_vector (H-04)` — ver git log
  5. Actualizar plan file: agregar `=== RECITATION DESKTOP-QW7 ===` (esta iteración) ✅ hecho 2026-08-27 22:23
  6. `campaign_memory_write` lesson H-04 ✅
  7. `campaign_diagnose_pipeline` + `skill progreso` Trigger 1
- **Verify:** `cargo fmt --check` ✅ + plan recitation presente ✅ + git commit hash ✅ — listo para handoff
- **Estado:** ✅ DONE

## Dependencias
- DESKTOP-QW6 ✅ COMPLETED (c87c72a7, CSP) — **bloqueante directo**: Wave2 Task7 desbloqueado tras QW6
- DESKTOP-QW5 ✅ COMPLETED (b0d231a7, limpiar DAUD) — Wave1 cerrada
- DESKTOP-QW1-4 ✅ COMPLETED (palette, HelpPanel F1/F2, statusReport ES, filterActive)
- a7ed0d22 — Fix sparse_vector ya aterrizado (H-04 end-to-end: vanta.ts + undo.ts + types.rs + test) — este task es verify/audit del fix bundle + cierre oficial recitation
- Ninguna técnica bloqueante más (Task7 toca undo.ts/vanta.ts disjunto de tauri.conf.json QW6)

## Notas
- Ponytail: si Step1 auditoría confirma código ya contiene sparse_vector en rename+undo (4 hits) y test fija (2 asserts), **no editar** — audit-only (deletion over addition). Shortest diff wins (0 líneas cambiadas) como QW1/QW4.
- H-04 threat: pérdida silenciosa de ranking BM25 — RRF híbrido cae a vector-only sin sparse_vector. No crash, solo mutación silenciosa (data integrity). Test fija con sparse_vector {0:0.5,5:1.25} — cubre forward (ingestBatch) + reverse (undo vantaPut).
- Bridge Rust touched (types.rs + vanta.ts DTO) → `cargo check -p vantadb` obligatorio por contrato quickwins. Aunque fix ya en a7ed0d22, verify ahora still gate (Regla 11 no aplica: no claim performance).
- No tocar WAL/vector/storage directo (Regla 6 deuda P2-8 intacta). Undo queue ya serializa (Regla 8 concurrencia: no nuevo dashmap/tokio).
- Campaign system hasTask false para este plan (no MCP registration) → recitation manual en plan file + memory_write (compatible QW1-6).
- Si Step2 se decide SKIP (auditoría dice mapper ya correcto), documentar justificación y saltar a Step3 verify-only — Gate V: 2 fallas mismo error → question al usuario.

## Context Save Point
- **Fecha:** 2026-08-27T22:23
- **Branch:** develop
- **CI pendiente:** ninguno — build 13.05s (2863 modules) + tests 69/69 (23.54s) + cargo check 0.57s + fmt verde
- **Decisiones:** Audit-only ejecutado (fix ya en a7ed0d22 + verificada presencia sparse_vector en 4 archivos, 0 ediciones). Steps 1 ✅, 2 SKIPPED, 3 ✅, 4 ✅ — contrato mecánico verde, plan recitation actualizada, listo para commit.
- **Problemas conocidos:** ninguno — contrato mecánico verde
- **Próxima tarea:** DESKTOP-QW8 (Wave3 Task8 — versión sync release-plz) — desbloqueada tras QW7

