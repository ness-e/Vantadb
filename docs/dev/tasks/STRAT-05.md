---
title: "TASK STRAT-05: Ruta object storage (S3/blob) — research + spec + decisión (ADR)"
kind: task
description: "Research + spec de la ruta object storage (viabilidad, costo, tradeoffs snapshot-level vs segment-level vs backend nativo) con decisión documentada (ADR-0056); sin implementación; 16 fuentes fetch-verificadas 2026-10-06"
---

# TASK STRAT-05: Ruta object storage (S3/blob) — research + spec + decisión (ADR)

- **Fecha:** 2026-10-06 · **Tipo:** research/spec (docs-only — cero código, cero símbolos públicos) · **Cero implementación** (el contrato es research + spec + decisión)
- **Contrato (plan Task 65 L1867):** "research + spec de la ruta object storage (viabilidad, costo, tradeoffs: snapshot-level vs segment-level vs backend nativo) con decisión documentada (ADR si toca el modelo de storage); sin implementación; fuentes citadas (Regla 11)."
- **Origen:** plan `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` Task 65 (L1858-1884, bloque F0 expandido) + Backlog `docs/dev/Backlog.md:168` (fila STRAT-05, removida al cierre — Trigger 1 progreso) + `docs/dev/Backlog-negocio.md:103` (BIZ-14) + veredicto 2026 (`docs/dev/archive/research-old/feature-verdicts-2026.md:128-137,387`).
- **Alcance:** decidir viabilidad/costo/tradeoffs de la ruta object storage **antes** de que el trigger Pro/Cloud la fuerce sin diseño. **No** implementa backend, **no** re-litiga el veredicto 2026 ni BIZ-14 — los cita y declara la frontera (backup offsite vs storage path).

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 65, bloque F0)
- **Fuente:** Backlog `:168` (fila STRAT-05) + plan Task 65 L1858-1884 + `Backlog-negocio.md:103` (BIZ-14) + `feature-verdicts-2026.md:128-137,387` (veredicto previo)
- **Esfuerzo:** 🔴 1-2sem (research) | **Appetite:** max 1sem (⚠️ esfuerzo > appetite → contrato acotado a research+spec+decisión)
- **Prioridad:** 🟡 (plan) / 🔵 P3 (Backlog)
- **Tipo:** Research/spec (docs-only)
- **Turns estimados:** 8-12 (una sesión de sub-agente)
- **Creado:** 2026-10-06 | **last-synced:** 2026-10-06
- **Estado:** ⏳ IN PROGRESS (reservada por el orquestador; **server taskId: `65`** para el cierre — instrucción del orquestador; `analyze_task`/`validate_scope` resuelven por texto `STRAT-05`, el numérico no resuelve en esos parsers — precedente MEMG-19)
- **Incógnitas (uphill):** 0 — resueltas en DISCOVERY: (a) ¿snapshot-level vs segment-level vs backend nativo? → resuelto por evidencia multi-fuente (precedentes + restricción mmap + costos) → snapshot-level ahora / segment-level al trigger Pro-Cloud / nativo diferido con condiciones; (b) costos → modelo con supuestos marcados + precios fetch-verificados (S3 Price List API 2026-09-28, R2 2026-10-01); (c) crate para snapshot-level → `object_store` (Apache Arrow) recomendado `[a validar al implementar]`
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `65`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Lectores de docs (sin código):** master plan Task 65 (estado — **prohibido editar**), `Backlog.md:168` (fila STRAT-05 — se elimina al cierre, Trigger 1 progreso), `Backlog-negocio.md:103` (BIZ-14 — se cita, no se toca), `ADR README` (regenerado por `gen-index --write`), avance `operaciones.md` (registro de cierre), ADR-0004/0020/STORAGE-TIERS (citados como contexto, intactos). |
| Callees | Fuentes citadas (sin modificar): `src/backend.rs:105-132` (BackendKind) + `:227` (trait), `src/storage/engine/init.rs:275-283` (BackendRegistry), `src/cli.rs:146-168,409-419` (Backup/Restore/Snapshot), `src/sdk/serialization/impl_export.rs:315,:531` (`export_all`/`import_file`), `src/sdk/builder.rs:281,:309` (snapshot SDK), `src/storage/engine/mod.rs:647,:697,:795` (`create_snapshot`/`snapshot_restore`), `docs/dev/architecture/adr/ADR-0004`, `ADR-0020`, `STORAGE-TIERS.md`, `feature-verdicts-2026.md`, `research/archive/res02-backup-restore.md`, 16 URLs externas fetch-verificadas. |
| Implicaciones | **Aditivo docs-only:** `docs/dev/research/strat-05-object-storage.md` (nuevo; `kind: research`) + `docs/dev/architecture/adr/ADR-0056-object-storage-path.md` (nuevo; `kind: adr`) + `docs/dev/tasks/STRAT-05.md` (nuevo, este archivo) + ADR README/índices regenerados + fila `Backlog.md:168` eliminada + registro `avance` al cierre. Sin código, sin wire, sin deps, sin locks, sin migración. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-06, worktree sobre HEAD `b57bf445`, branch `develop`; WIP ajeno: master plan + `opencode.jsonc` modificados y `dev-tools/heavy-test-lock.ps1` untracked — **no se tocan ni se stagean**).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 65 L1858-1884 + reglas de ejecución L2080-2095 — regla investigación profunda).
  - `src/backend.rs` (:1-160 — `BackendPartition`, `BackendKind:105-132`, `BackendCapabilities`, roles ISP).
  - `docs/dev/architecture/adr/ADR-0004-storage-backend.md` (completo — Fjall default / RocksDB opt-in).
  - `docs/dev/architecture/adr/ADR-0020-storage-backend-default.md` (completo — evidencia `file:line` del default).
  - `docs/dev/architecture/STORAGE-TIERS.md` (completo — tiers L0-L3, `vstore_L*.vanta` mmap).
  - `docs/dev/architecture/adr/DECISIONS-NOT-TAKEN.md` (completo — registro de descartes).
  - `docs/dev/archive/research-old/feature-verdicts-2026.md` (:115-149 backup/restore + S3; :375-397 "fuera del roadmap").
  - `docs/dev/Backlog-negocio.md` (:103 — BIZ-14).
  - `src/cli.rs` (:140-199 Backup/Restore; :405-459 SnapshotCommand).
  - `docs/dev/research/archive/res02-backup-restore.md` (:1-40 — gap analysis backup físico; **archived**, citado como historia; `wal_archiver.rs` ya no existe — re-verificado).
  - Evidencia `file:line` vía rg: `src/sdk/serialization/impl_export.rs:315,:531`, `src/sdk/api/memory.rs:2515`, `src/sdk/builder.rs:281,:309`, `src/storage/engine/mod.rs:647,:697,:795`.
  - `.opencode/task-system/prompts/task.md` (formato canónico), `pipeline-full.md` (contrato de ejecución), `docs/dev/tasks/MEMG-19.md` + `docs/dev/research/memg-19-prospectiva-descartes.md` (formato de precedente F5).
- **Archivos referenciados hacia dentro (imports/deps):** n/a (docs; sin imports). El research-doc referenciará hacia afuera con enlaces relativos (documentation-skill §1).
- **Referencias entrantes (grep `STRAT-05|object storage|S3` HEAD `b57bf445`):** `Backlog.md:161` (nota DELTA) y `:168` (fila), master plan L1858-1884 (**prohibido editar**), `Backlog-negocio.md:103` (BIZ-14), `feature-verdicts-2026.md:128-137,387`, `DECISIONS-NOT-TAKEN.md` (sin fila S3 — la frontera se declara en el ADR). Sin referencias de código.
- **Veredicto impacto:** **BAJO (aditivo docs-only)** — 1 research doc + 1 ADR + 1 task file + índices regenerados + 1 fila Backlog eliminada al cierre + 1 registro avance. Sin cambios en código, contratos, wire, deps ni locks. Riesgos del pre-mortem mitigados: (1) contrato = research+spec+decisión, cero implementación; (2) veredicto 2026 / BIZ-14 citados con frontera declarada, no re-litigados; (3) costos con supuestos marcados + precios fetch-verificados y fechados.

## Contrato

"research + spec de la ruta object storage (viabilidad, costo, tradeoffs: snapshot-level vs segment-level vs backend nativo) con decisión documentada (ADR si toca el modelo de storage); sin implementación; fuentes citadas (Regla 11)." (plan Task 65 L1867).

**Verificación del contrato (cierre):** research-doc existe con (a) **viabilidad** por opción (snapshot-level / segment-level / backend nativo) contra el estado real del storage VantaDB (Fjall/RocksDb/InMemory, mmap vstore, WAL — con evidencia `file:line`); (b) **costo** con modelo + supuestos marcados y precios verificados (S3/R2, fechas); (c) **tradeoffs** explícitos (RPO/RTO, latencia, complejidad, blast radius); (d) **decisión documentada** en `ADR-0056` (status + opciones consideradas + consecuencias); (e) **16 fuentes fetch-verificadas y fechadas** (TSYS-13/Gate citas); gates docs 0 (`check-links` + `check-docs` + `gen-index --check`) + `validate-docs-coverage.ps1` 0 gaps; cero código tocado.

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): no disparado.** (a) Blast radius ≤ 8 archivos propios, docs-only, sin hot path/API pública; (b) sin símbolos públicos nuevos (cero código); (c) contrato sancionado por el plan F0 (Task 65, Gate Result ✅ DO) — sin ambigüedad nueva; (d) el veredicto 2026 y BIZ-14 ya están decididos/registrados — este run **decide la ruta** con evidencia y documenta, no re-litiga. Si la evidencia hubiera contradicho un descarte → BLOQUEO/nota, no decisión unilateral.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Hogar del research doc | A) **`docs/dev/research/strat-05-object-storage.md` (kind=research)** / B) sección en `STORAGE-TIERS.md` (mezcla tiers vectoriales con ruta cloud) / C) `docs/dev/strategy/` (no es hogar de investigaciones) | ✅ **A** — patrón `memg-*`/`mgr-*`; `kind: research` → ruta canónica (documentation-skill §2.1) |
| 2 | ADR | A) **ADR-0056 nuevo `docs/dev/architecture/adr/ADR-0056-object-storage-path.md` (MADR, status proposed)** / B) sin ADR (contra: el contrato pide decisión documentada y toca el modelo de storage) / C) editar ADR-0004/0020 (contra: inmutables) | ✅ **A** — siguiente número libre (último: ADR-0055); MADR con opciones consideradas |
| 3 | Ruta recomendada | A) **snapshot-level ahora (BIZ-14) + segment-level al trigger Pro/Cloud + backend nativo diferido con condiciones** / B) backend nativo ya / C) descartar object storage | ✅ **A** — evidencia §Notas: precedentes (Lance/SlateDB = motores nuevos para object storage; DuckDB = read-only; sqlite-s3vfs/CBS = block-layer con locking a cargo de la app), restricción mmap (`vstore_L*.vanta`, STORAGE-TIERS), costos (S3/R2), latencias 35-100ms/request (SlateDB/Chroma) |
| 4 | Abstracción para snapshot-level | A) **crate `object_store` (Apache Arrow)** / B) `aws-sdk-s3` (árbol de deps pesado) / C) `rust-s3`/OpenDAL | ✅ **A** — uniforme S3/GCS/Azure/R2/local, async, producción (crates.io, InfluxDB IOx); `[a validar al implementar]` (no es implementación en este run) |
| 5 | Modelo de costos | A) **Tabla con supuestos explícitos (tamaño DB, cadencia, retención) + precios fetch-verificados fechados** / B) números sin fuente | ✅ **A** — Regla 11; precios S3 (Price List API 2026-09-28) y R2 (página 2026-10-01) verificados 2026-10-06 |
| 6 | Verificación de fuentes (TSYS-13) | A) **Fetch de cada URL citada en DISCOVERY; solo se citan las resueltas, con fecha** / B) citar y marcar `[a verificar]` | ✅ **A** — red disponible; 16/16 externas resueltas (fetch 2026-10-06) |
| 7 | Tono / alcance | A) **No normativo para código hasta trigger; cero implementación; la implementación de BIZ-14 queda como fila/plan futura** / B) normativo inmediato | ✅ **A** — contrato "sin implementación"; el ADR propone y queda a ratificación del owner (Regla 5) |

## Invariantes de dominio (handoff — MUST)

1. **Local-first intacto:** esta ruta **no cambia** el modelo de storage actual — Fjall default / RocksDb opt-in / InMemory (ADR-0004 + ADR-0020 vigentes); ningún backend nuevo se declara en código en este run.
2. **No re-litigar:** el veredicto 2026 ("no S3 todavía; S3 es Fase 5"; "❌ S3 backup nativo cloud only") y BIZ-14 (trigger Pro/Cloud, Dep PRO-02/03) se **citan**; la frontera queda declarada: **backup offsite = habilitable (snapshot-level)** vs **storage path (DB viviendo en object storage) = diferido**.
3. **Cero código:** sin `pub fn`/tipos/bindings; sin deps nuevas; el ADR es propuesta (status `proposed`) hasta ratificación del owner.
4. **Regla 11:** todo número con fuente verificada + fecha; supuestos del modelo de costos marcados como supuestos; cero claims sin fuente.
5. **No tocar:** master plan, `opencode.jsonc`, `docs/pipeline-state.json` (prohibidos). WIP ajeno no se stagea; commit con **pathspec**.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** ≤0 — sin código, sin `unsafe`, sin deps. El run **elimina** deuda de decisión: la ruta object storage deja de ser un hueco sin diseño (riesgo de que el trigger Pro/Cloud la fuerce) y queda con opciones, costos y condiciones auditables. Deuda declarada: ninguna nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Research-doc con viabilidad/costo/tradeoffs por opción (3 opciones) + decisión documentada en ADR-0056 + 16 fuentes fetch-verificadas y fechadas, verificable 1:1 contra el contrato L1867; gates docs 0; task file completo (Impacto Regla 0 + Spec + Review P2-01) |
| **Commit** | Commit atómico conventional `docs(research):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | n/a (docs; sin changelog) |

## Herramientas necesarias

- `websearch`/`webfetch` (fetch-verificación TSYS-13) + skill `coordinated-web-search` (router obligatorio) + rg/read (evidencia `file:line` interna) + `campaign_*` (estado/scope/verify)
- Gates docs: `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --write` + `pwsh scripts/validate-docs-coverage.ps1`
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre) + fork `vanta-review` (P2-01)

**Skills cargadas (SDP v3):** base auto (`campaign-executor` · `progreso` · `ponytail`) · `writing-guidelines` + `writing-plans` (base type Documentation) · `documentation-skill` (obligatoria `docs/**`) · `documentation-and-adrs` (ADR) · `source-driven-development` (verificación de fuentes) · `coordinated-web-search` (router obligatorio — regla investigación profunda owner 2026-10-06) · `spec-driven-development` (lifecycle DEFINE — decisión/spec documental) · `doubt-driven-development` (plan Task 65 la sugiere — postura adversarial; el review P2-01 lo ejecuta `vanta-review` en contexto fresco) · `deprecation-and-migration` (policy pin storage/schema — declarada N/A: sin deprecación/migración en este run). Excluidas con justificación: `interview-me`/`idea-refine` (Gate D no disparado; contrato sancionado por el plan F0; sin requisitos ambiguos), `incremental-implementation`/`test-driven-development`/`context-engineering` (lifecycle BUILD — docs-only, sin lógica ni tests), `api-and-interface-design` (sin API nueva), `performance-optimization` (sin hot path — Regla 9 no dispara).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — N/A en este run (docs-only, sin trust boundary nuevo, sin input de usuario, sin deps, sin red en producto; los fetches de verificación son de DISCOVERY, solo lectura). El research-doc **documenta** los trust boundaries de la implementación futura (credenciales S3 por env/vault, IAM least-privilege, cifrado en reposo) como requisito de BIZ-14.
- [ ] **PERFORMANCE** — N/A: sin hot paths, sin código; Regla 9 no dispara (sin claim de optimización). Las cifras de latencia citadas (35-100ms/request object storage) son de fuentes externas fetch-verificadas, no claims propios.

## Steps

### Step 1 — DISCOVERY + task file (Regla 0 + Spec + contrato + investigación profunda)

- **Archivos:** `docs/dev/tasks/STRAT-05.md` (nuevo, este archivo)
- **Acción:** evidencia anclada (plan Task 65, `backend.rs`, ADR-0004/0020, STORAGE-TIERS, BIZ-14, veredicto 2026, CLI/SDK actuales); Gate D evaluado; SDP v3 (DEFINE); investigación profunda multi-fuente (16 fuentes fetch-verificadas 2026-10-06 — regla owner); formato canónico con Impacto Regla 0 + Spec + DoD.
- **Verify:** task file existe + `campaign_validate_scope` OK (advisory)
- **Evidencia:** ✅ este archivo; fuentes verificadas en §Notas (16/16 con fecha)
- **Estado:** ✅ COMPLETED

### Step 2 — Research-doc: ruta object storage (viabilidad / costo / tradeoffs)

- **Archivos:** `docs/dev/research/strat-05-object-storage.md` (nuevo, `kind: research`)
- **Acción:** doc en español con: §0 resumen ejecutivo (tabla de respuestas); §1 estado real del storage VantaDB (evidencia `file:line`); §2 las 3 opciones (snapshot-level / segment-level / backend nativo) con viabilidad técnica, precedentes y tradeoffs; §3 modelo de costos (supuestos marcados + precios verificados S3/R2); §4 decisión recomendada + condiciones/triggers + frontera con veredicto 2026/BIZ-14; §5 fuentes (tabla con fecha de verificación).
- **Verify:** doc existe; `rg -c "snapshot-level|segment-level|backend nativo"` > 0; secciones completas; gates docs corren al cierre.
- **Estado:** ✅ COMPLETED (2026-10-06 — doc escrito; `check-links`/`check-docs`/`markdownlint` verdes; fixes FIND-1/2/3/5 aplicados)

### Step 3 — ADR-0056 + regeneración de índices

- **Archivos:** `docs/dev/architecture/adr/ADR-0056-object-storage-path.md` (nuevo, MADR: Context and problem statement / Considered options / Decision outcome / Positive + Negative consequences, status `proposed`); `docs/dev/architecture/adr/README.md` (regenerado por `gen-index --write`)
- **Acción:** ADR en inglés (doc language split: `docs/dev/architecture/` = English) con las 3 opciones consideradas, la decisión (snapshot-level ahora; segment-level al trigger; nativo diferido con condiciones de revisión) y consecuencias (positivas/negativas) + ratificación pendiente del owner (Regla 5).
- **Verify:** `node scripts/docs/check-docs.mjs` (kind adr) + `node scripts/docs/gen-index.mjs --check` verde tras `--write`.
- **Estado:** ✅ COMPLETED (2026-10-06 — ADR-0056 `proposed` + índices regenerados: `adr/README.md`, `docs/index.md`, `llms.txt`)

### Step 4 — Verificación, review P2-01 y cierre

- **Archivos:** este task file (sync de estados) + índices regenerados si difieren
- **Acción:** gates docs 0 (`check-links` + `check-docs` + `gen-index --write`/`--check`) + `validate-docs-coverage.ps1`; OCR delegation (`ocr-review.ps1 -Format json`); review P2-01 (fork `vanta-review` — contexto fresco, adversarial); commit **LOCAL** `docs(research):` con pathspec; cierre campaign `taskId: "65"` con payload `review` (HARD-07); `skill progreso` (Trigger 1 — elimina fila Backlog `:168` + registro avance).
- **Verify:** gates 0 + veredicto review registrado + commit local + campaign `completed` (`updated:true`).
- **Estado:** ⬜ PENDING

## Dependencias

- Task 64 (STRAT-04, WASM threads — en vuelo, otra área): **sin dependencia mutua** (áreas disjuntas: WASM vs storage; releído fresco; pathspec en el commit).
- BIZ-14 (`Backlog-negocio.md:103`): **consumidor aguas abajo** — este research le da la decisión de diseño; no se toca su fila (trigger Pro/Cloud, Dep PRO-02/03).
- Task 66 (STRAT-06): independiente (licencias); próxima tarea del plan.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador (`vanta-review` — fork en contexto fresco). Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` — P2-01, contexto fresco (esta sesión solo leyó los artefactos; no participó de la implementación); tier **Fast** (paths del diff: `docs/dev/**` + `docs/dev/architecture/adr/**` → verify mecánico + adversarial de contenido; sin wire/API/código).
- **Enfoque:** ¿la decisión (snapshot-level ahora / segment-level al trigger / nativo diferido) está respaldada por la evidencia citada? ¿hay alternativas mejores no evaluadas? ¿la frontera con veredicto 2026/BIZ-14 es correcta (no re-litigio)? ¿los supuestos de costo están marcados y los precios fechados?
- **Cómo se probó:** gates re-corridos por el reviewer sobre HEAD `b57bf445`: `check-links` exit 0 (0 broken) · `check-docs` exit 0 (GATING all clear) · `gen-index --check` exit 0 · `validate-docs-coverage` exit 0 (0 gaps); **10/16 fuentes re-fetcheadas en vivo** (slatedb.io/index.md · litestream · CBS · sqlite-s3vfs · DuckDB · Lance · Chroma serverless · docs.rs/object_store · R2 pricing · DevZero S3) — 10/10 fieles a lo citado, quotes exactos ("stateless APIs instead of cursor based interfaces… Read or Seek" · "will probably become corrupt" · "35-100ms" · "~50–100ms per request" · R2 $0.015/$4.50/$0.36/egress $0); 6/16 no re-fetcheadas (declaradas fetch 2026-10-06; sin contradicción); anclas internas 10/10 exactas (`backend.rs:105-132` · `cli.rs:146-168,409-419` · `engine/mod.rs:647,697,795` · `builder.rs:281,309` · `impl_export.rs:315,531` · `memory.rs:2515` · STORAGE-TIERS mmap · `wal_archiver.rs` inexistente) + 2 imprecisas → FIND-2; veredicto 2026 (`:128,:135-137,:387`) y BIZ-14 (`Backlog-negocio.md:103`) exactos; contrato plan L1867 1:1; scope `git status` docs-only (WIP ajeno no atribuible, no stageado).
- **Hallazgos + disposición (FIND candidates — Medium/Low no bloquean):**
  1. [MEDIUM] Timing "**Ahora**" (research §4 L127) / "for v0.9.x" (ADR L44) puede leerse como implementar BIZ-14 en 0.9.x; BIZ-14 es "🔮 Futuro: trigger Pro/Cloud" (`Backlog-negocio.md:103`) y el veredicto dice "S3 es Fase 5" (`feature-verdicts-2026.md:137`). → Recomendado antes de ratificar el ADR: "decisión ahora; implementación sujeta al trigger Pro/Cloud de BIZ-14" (1 frase en research §4 + ADR outcome #1).
  2. [LOW] Evidencia: `src/backend.rs:20-21` citado para trait `pub(crate)` (declaración real `:227`; `:20-21` es doc-comment); `src/storage/engine/init.rs:269-289` etiquetado "dispatch" (el dispatch real es `:278-283`, BackendRegistry).
  3. [LOW] `ADR-0038` citado como estado WAL sin nota de status (`proposed`, spec-only).
  4. [LOW] Fuentes: sqlite-s3vfs desde fork `dpedu` (upstream `uktrade`); precios S3 vía blog DevZero (verificado: cita Price List API 2026-09-28 — etiqueta correcta; AWS primario sería más fuerte).
  5. [LOW] A3 "PUTs batched ≈ $0.15-0.30/mo" sin supuesto de writes/mes (A1/A2 sí lo declaran).
  6. [LOW] Spec de BIZ-14 (futuro): fijar artefacto offsite (archivo único vs directorio multi-archivo — la atomicidad "objeto entero" es por objeto), ubicación de la dep `object_store` (default vs feature opt-in — preocupación zero-config del veredicto) e integridad en restore.
  7. [LOW] Cierre: Steps 2-3 tienen artefactos pero siguen ⬜ PENDING (sync en Step 4); commit con pathspec excluyendo master plan (contiene el flip EN PROGRESO del orquestador) y `opencode.jsonc`.
- **Checklist anti-hábitos tóxicos:** ✅ verificado — (a) salidas de gates re-ejecutadas por el reviewer coinciden (no inventadas); (b) contrato L1867 verificado 1:1; (c) supuestos de costo marcados como supuestos (§3), precios fechados; (d) veredicto 2026/BIZ-14 citados sin reapertura; (e) steps 1-4 conectados al contrato (sin huérfanos); (f) SDP v3: pins justificados (`deprecation-and-migration` N/A aceptable — el run no deprecia ni migra nada).
- **Veredicto:** ✅ **APPROVE** — 0 Critical/High; contrato 1:1 (viabilidad + costo + tradeoffs ×3 opciones + decisión en ADR-0056 + 16 fuentes fechadas + cero implementación); decisión respaldada por 5/5 precedentes verificados (SlateDB/Lance/DuckDB/CBS/sqlite-s3vfs) + anclas internas; frontera declarada sin re-litigio; costos con supuestos marcados y aritmética correcta; gates 4/4 re-verdes. FIND-1 (Medium) recomendado antes de ratificar el ADR; FIND-2..7 no bloquean.
- **Disposición aplicada (2026-10-06, post-review):** FIND-1 corregido inline (research §4 + ADR outcome #1: "decisión ahora; implementación sujeta al trigger Pro/Cloud de BIZ-14"); FIND-2 corregido (`backend.rs:227` / `init.rs:275-283`); FIND-3 corregido (ADR-0038 *proposed*); FIND-5 corregido (supuesto A3 declarado); FIND-4 se descarta (se conserva la URL fetch-verificada); FIND-6 anotado en §Notas (spec futuro de BIZ-14); FIND-7 aplicado en el cierre (steps sincronizados + pathspec sin WIP ajeno). Gates re-corridos post-fix: verdes.

## Notas

### Investigación profunda — hallazgos clave (16 fuentes fetch-verificadas 2026-10-06)

**Precedentes de "DB sobre object storage" (qué hizo cada uno):**

1. **DuckDB**: soporte **read-only** sobre HTTPS/S3 (`ATTACH ... (READ_ONLY)`); el guide oficial dice "You can establish a **read-only** connection to a DuckDB instance via HTTPS or the S3 API" (discusión #10466 confirma que no hay connect r/w).
2. **SQLite Cloud Backed (CBS, oficial)**: divide la DB en **bloques fijos (default 4MB) como objetos separados + manifest**; soporta lectura y escritura; single-writer "mostly left to the application"; Azure/GCS nativos (S3 vía módulo custom).
3. **sqlite-s3vfs** (VFS comunitario): bloques como objetos; **"S3 does not support the partial replace of an object; to change even 1 byte, it must be re-uploaded in full"**; **sin locking** → "if multiple writes happen at the same time, the database will probably become corrupt".
4. **Litestream**: **replicación streaming** del WAL de SQLite → LTX files (TXID + checksums) → S3; es DR (RPO ~segundos), no storage remoto; restore = replay. Corre como proceso separado (no toca el motor).
5. **SlateDB** (Rust, Apache-2.0): LSM **construido para object storage** (S3/GCS/Azure/MinIO/R2); "object storage request latencies are an order of magnitude higher (**~50–100ms per request**)"; single-writer/multi-reader; en producción (Dropbox, ZeroFS, HelixDB); **near 1.0**. Su FAQ: "Can't I use S3 as a key-value store? ... **you pay one PUT per write, which gets expensive**" → de ahí SSTs/LSM.
6. **Lance/LanceDB**: formato de archivo **optimizado para object storage y lecturas selectivas** (evita row groups; random access vía encodings; fragmentos + manifests; catalog directory = cero infra sobre el bucket). Es un **formato nuevo**, no un backend sobre el formato existente.
7. **Chroma Cloud**: arquitectura serverless sobre object storage (log + índices en object storage, catálogo SQL, SSD local como caché); telemetría: clientes **"insensitive to moderate latencies of 35-100ms"**; access pattern power-law → object storage es costo-efectivo; compactors construyen índices async.
8. **Kuzu**: extensión httpfs — **read/write/glob de archivos** (Parquet etc.) sobre S3-compatible (R2 incluido); **no** es la DB viviendo en S3 (import/export de datos).
9. **crate `object_store`** (Apache Arrow): API async uniforme S3/GCS/Azure/local/HTTP; "atomic, conditional reads and writes, vectored IO, bulk deletion"; **"stateless APIs instead of cursor based interfaces such as Read or Seek"** — explícitamente NO es un filesystem (no mmap/seek); producción: crates.io, InfluxDB IOx.
10. **S3 semantics** (oficial AWS): multipart upload = partes independientes (hasta 10.000; retransmitir una parte sin tocar las demás) → lo más cercano a escritura parcial; **no hay replace parcial de bytes**.
11. **Precios S3** (Price List API us-east-1, publicación 2026-09-28): Standard $0.023/GB-mo (primeros 50TB); PUT/COPY/POST/LIST $0.005/1.000; GET $0.004/10.000; Standard-IA $0.0125/GB-mo + retrieval $0.01/GB; Glacier IR $0.004/GB-mo. **Egress** $0.09/GB (primeros 10TB, tras 100GB free), S3→CloudFront gratis, cross-region $0.02/GB (verificado 2026-09).
12. **Precios R2** (página actualizada 2026-10-01): $0.015/GB-mo; Class A $4.50/M; Class B $0.36/M; **egress gratis**; free tier 10GB/1M/10M.
13. **Costo de referencia EFS** (SlateDB FAQ): $0.30/GB-mo storage + $0.03/GB reads + $0.06/GB writes → 20x el storage de S3; refuerza "object storage para DR/cold, no para el camino caliente".

**Estado real VantaDB (evidencia interna):**
- Backend KV: `BackendKind { RocksDb, Fjall (default), InMemory }` (`src/backend.rs:105-132`); construcción vía `BackendRegistry` (`src/storage/engine/init.rs:275-283`); trait `pub(crate)` (`src/backend.rs:227`) — **sin variante object storage**.
- Vector store: segmentos `vstore_L0..L3.vanta` **mmap** (STORAGE-TIERS.md) — object storage **no se puede mmap** (ni seek; `object_store` lo declara por diseño).
- Backup **local ya existe**: CLI `Backup`/`Restore` (`src/cli.rs:146-168`), `Snapshot Create/List` (hard-links; `:409-419`), engine `create_snapshot`/`snapshot_restore` (`src/storage/engine/mod.rs:647,:697,:795`), SDK `create_snapshot`/`restore_from` (`src/sdk/builder.rs:281,:309`), lógico `export_all`/`import_file` (`src/sdk/serialization/impl_export.rs:315,:531`) + `bulk_import_file` (`src/sdk/api/memory.rs:2515`). **Lo que falta = el tramo "offsite" (subir/bajar a red)** — exactamente BIZ-14.
- RES-02 (archivado 2026-08-25) documentó el gap físico; re-verificado hoy: `wal_archiver.rs` **ya no existe**; snapshot restore ya existe. No se cita como estado actual sin esa nota.

**Modelo de costos (supuestos marcados — Regla 11):**
- **A1** (snapshot-level, caso típico): DB 10 GB; backup diario full; retención 7 diarios + 4 semanales (11 copias ≈ 110 GB-mo); 1 restore/mes.
  → S3 Standard: 110 GB × $0.023 ≈ **$2.53/mo** + PUTs despreciables (11-30/mo) + restore egress 10GB × $0.09 = **$0.90/restore** → **≈ $2.5-3.5/mo**.
  → R2: 110 GB × $0.015 ≈ **$1.65/mo** + egress **$0**.
- **A2** (segment-level, WAL shipping estilo Litestream): 1 segmento/min ≈ 43.2k PUTs/mo × $0.005/1k ≈ **$0.22/mo** + storage incremental (supuesto 50 GB) ≈ **$1.15/mo S3 / $0.75 R2** + GETs de restore (miles ≈ céntimos).
- **A3** (backend nativo): dominado por latencia (~50-100ms/request) y rediseño del motor (mmap/fsync/compaction), no por $ (PUTs batched ≈ $0.15-0.30/mo). El costo real es de **ingeniería** (multi-trimestre; precedentes construyen formatos/motores nuevos).
- **Supuestos marcados:** tamaño de DB, cadencia, retención y volumen incremental son supuestos de escenario (no mediciones); los **precios** son list prices verificados con fecha. Compresión/dedup no modeladas (a favor de snapshot-level con `export_all` comprimido o snapshots hard-link).

**Pre-mortem (plan Task 65) — mitigaciones aplicadas:** (1) scope a implementación → contrato ejecutado como research+spec+decisión, cero código; (2) duplicar BIZ-14/veredicto → citados con frontera declarada (backup vs storage path); (3) costos sin verificar → supuestos marcados + precios fechados.

### Notas del review P2-01 (para el spec futuro de BIZ-14 — FIND-6)

- Artefacto offsite: fijar si el backup sube **un archivo único** (p.ej. `export_all` comprimido) o un **set multi-archivo** (dir snapshot) — la atomicidad de S3 es *por objeto*, un set no es atómico; el manifiesto debe versionarse.
- Ubicación de la dependencia `object_store`: default vs feature opt-in (la preocupación zero-config del veredicto 2026 sigue vigente).
- Verificación de integridad en restore (CRC/checksum del artefacto antes de aplicar).
- OCR delegation (cierre): 0 archivos de la tarea reviewable (todos `.md` → `unsupported_ext`); el único reviewable del workspace es `dev-tools/heavy-test-lock.ps1` (WIP ajeno — **no se toca ni se stagea**).

### Fuentes verificadas (fetch 2026-10-06 — TSYS-13)

| # | Fuente | Qué se extrajo |
|---|--------|----------------|
| 1 | https://slatedb.io/index.md | SlateDB: LSM sobre object storage; 50-100ms/request; single-writer/multi-reader; Dropbox et al; near 1.0 |
| 2 | https://slatedb.io/docs/get-started/faq/ | "one PUT per write gets expensive"; WAL a object storage; EFS $0.30/GB-mo |
| 3 | https://litestream.io/how-it-works/ | WAL→LTX (TXID+checksums); replicación, no storage remoto; 64MiB max-sync-wal-bytes |
| 4 | https://sqlite.org/cloudsqlite/doc/trunk/www/index.wiki | CBS: bloques fijos (4MB) + manifest; single-writer por app; Azure/GCS |
| 5 | https://github.com/dpedu/sqlite-s3vfs | "S3 does not support the partial replace of an object"; sin locking → corrupción |
| 6 | https://duckdb.org/docs/current/guides/network_cloud_storage/duckdb_over_https_or_s3 | DuckDB: conexión **read-only** vía HTTPS/S3 |
| 7 | https://github.com/duckdb/duckdb/discussions/10466 | Confirmación: `ATTACH ... (READ_ONLY)`; sin connect r/w |
| 8 | https://lance.org/format/ | Lance: formato optimizado para object storage; fragmentos/manifests; directory catalog |
| 9 | https://docs.trychroma.com/reference/architecture/distributed | Chroma: log + índices en object storage; catálogo SQL; SSD caché |
| 10 | https://www.trychroma.com/engineering/serverless | Chroma serverless: power-law; "35-100ms"; compactors async |
| 11 | https://kuzudb.github.io/docs/extensions/s3/ | Kuzu: read/write/glob de **archivos** vía httpfs; multipart upload; R2 OK |
| 12 | https://docs.rs/object_store/latest/object_store/index.html | object_store: uniforme S3/GCS/Azure; stateless (no Read/Seek); crates.io/IOx |
| 13 | https://developers.cloudflare.com/r2/pricing/ (upd. 2026-10-01) | R2: $0.015/GB-mo; Class A $4.50/M; Class B $0.36/M; egress $0 |
| 14 | https://www.devzero.io/blog/aws-s3-pricing (Price List API, 2026-09-28) | S3: $0.023/GB-mo; PUT $0.005/1k; GET $0.004/10k; IA/Glacier |
| 15 | https://egresscost.com/aws/s3-egress-pricing/ (verif. 2026-09) | Egress $0.09/GB (10TB, 100GB free); S3→CloudFront gratis; cross-region $0.02/GB |
| 16 | https://docs.aws.amazon.com/AmazonS3/latest/userguide/mpuoverview.html | Multipart: partes independientes (10k), retransmisión por parte; no replace parcial |

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas en DISCOVERY (ruta por evidencia; costos modelados; crate recomendado) |
| Pendientes de ejecución (downhill) | 4 steps (1 ✅, 2-4 ⬜) |
| % completado | 25% (Step 1) |
