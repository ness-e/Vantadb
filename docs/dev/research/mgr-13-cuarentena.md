# MGR-13 — Cuarentena semántica + abstención: estados, transiciones y threat model write-time (Cierre MGR)

- **Fecha:** 2026-09-28 · **Tipo:** research/design documental (cero implementación productiva; `src/` no se toca)
- **Contrato:** "research-doc cerrado con estados + transiciones (entrada/promoción/expiración con dueño y trigger) + threat model write-time por superficie (API/dream/import), listo para SCH-01"
- **Plan:** `docs/dev/plans/2026-09-26-master-roadmap.md` Task 25 (F3) · **Origen:** `docs/dev/Backlog.md:847` · **Decisión owner:** migración única con MGR-10/12 (2026-09-14 en `Backlog:847`; re-articulada 2026-09-24 en `:934`)
- **Consume:** MGR-12 (clase asserted/derived, co-batch F3 — Task 24) · **Destraba:** SCH-01 (ADR) → SCH-02 (schema) → SCH-05 (operación) → SCH-06 (tests) · **Next:** SCH-01
- **Fuera de alcance (stop conditions del bloque):** motor de políticas (ABAC/namespaces trusted → MGR-04, fuera del plan → FIND); hash-chain/PROV-O de auditoría (VER-01, F4 — se cita, no se duplica); detección de poisoning por ML (v1.0)

## §0. Resumen ejecutivo

No existe cuarentena de contenido en VantaDB: el único uso del término está en el salvage de WAL corrupto (`src/wal.rs:614`, integridad de storage, no estado de contenido). Los registros no tienen estado de confianza, la inyección (`auto_recall`, `inject_context`, `memory_recall`) no filtra nada hoy, y `dream_promote` todavía es un stub de preview (`vanta-memory/src/core/dream/mod.rs:614-622`).

Propuesta (slice 0.8.0, sin motor de políticas):

1. **Estado binario explícito**: `active` (default implícito) → `quarantined` (campos nuevos en el record; §2).
2. **Entrada write-time** con señales medibles: flag explícito en el write, promoción de dream con origen derivado (default ON), opción de import (§3.4).
3. **Promoción revisable** (op dueño+trigger, separación reviewer/writer auditable) y **expiración** que nunca auto-promueve (§3.2) — un atacante no puede esperar el reloj.
4. **Default-exclude en retrieval** (search/list) con opt-in `include_quarantined`, y **gates duros de inyección**: `auto_recall`/`inject_context`/`recall` nunca inyectan cuarentenado (§5).
5. **Abstención selectiva** con señal explícita `abstained` en el wire (default OFF, solo con umbral configurado) (§5.3).
6. **Threat model write-time por superficie** (§4): API, dream/consolidación, import, MCP agent-facing — con las clases AgentPoison/MINJA/OWASP ASI06 y el patrón Quarantine de Azure.

Lo que este documento **NO** propone: namespaces trusted/tainted (MGR-04), motor ABAC, hash-chain de auditoría (VER-01), detección automática de poison por ML. Esas quedan como FIND/deuda v1.0 (§8).

## §1. Gap verificado (código real — 2026-09-26/28)

| # | Afirmación | Evidencia |
|---|-----------|-----------|
| 1 | No existe cuarentena de **contenido** | `rg 'quarantine\|quarantined'` en `src/` → solo salvage de WAL: `src/wal.rs:614-638` (`quarantine_corrupt_tail`: respalda el tail corrupto y trunca), `:640-642` (`quarantine_backup_path`); `src/wal_sharded.rs:259,287-313` (trunca a prefijo coherente + backup). Semántica de **storage**, no de contenido |
| 2 | Sin taxonomía de taint/trust de contenido | `rg 'tainted'` en `src/` = 0 hits de contenido (los `trusted_proxies` de `src/config.rs:246` son de red, no de contenido) |
| 3 | `MemoryRecord` (SDK) sin estado/trust | `src/sdk/types/record.rs:102-138` — 13 campos; los únicos estados "ocultables" hoy son `superseded_by`/`superseded_at_ms` (:129-137, ADR-028) y `expires_at_ms` (:126-128, TTL) |
| 4 | Sin abstención | plan Task 25 L658: `rg 'abstain\|abstention'` en `src/` = 0 hits |
| 5 | Superficies write-time sin gate de confianza | API: `src/server/middleware.rs:42-258` (auth) + `src/server/handlers.rs:248-265` (`records_put`/`records_put_batch`); dream: `vanta-memory/src/core/dream/mod.rs` (promotion stub :614-622, invariantes :13-27); import: `src/sdk/serialization/impl_export.rs:290-323` (`import_records`), `:326-364` (`import_file`), gate de schema en `src/sdk/serialization/mod.rs:522-531` |
| 6 | La inyección no filtra estado | `vanta-memory/src/core/hooks/auto_recall.rs:198-257` (`perform_auto_recall`); MCP `memory_recall` `vantadb-mcp/src/handlers/tools.rs:1818-1841`; `inject_context` (write para consolidación) :585-599 |
| 7 | `dream_promote` hoy es preview honestamente read-only | `vantadb-mcp/src/dreams.rs:108-126` (`readOnlyHint: true`) + `:252-267` (`{preview_count, mutated:false}`) — correcto para el stub; **checkpoint** cuando MEM-65 conecte el merge real (§4.2) |

MGR-12 co-batch: la clase de confianza asserted/derived y `confidence` por registro quedan definidas en ese research-doc/SCH-01; acá solo se **consume** como señal de retrieval (§5.4). No se toca `docs/dev/research/mgr-12-confianza.md`.

## §2. Precedentes internos y modelo de datos propuesto (para SCH-01/SCH-02)

### 2.1 Precedentes internos

| Precedente | Qué aporta | Diferencia con cuarentena |
|-----------|------------|---------------------------|
| `quarantine_corrupt_tail` (`src/wal.rs:614`) | Léxico "quarantine" ya usado en el proyecto + principio "nunca destruir evidencia" (backup antes de truncar) | Es integridad de storage; no hay estado por registro ni revisión |
| ADR-028 supersession (`record.rs:129-137`) | Estado ocultable, soft-dead, recuperable, con filtro opt-in (`exclude_superseded`), hasheado en fingerprint (`src/sdk/search/page.rs:157,317`) y roundtrip export (`impl_export.rs:211`) | Supersede ≠ trust: lo reemplazado fue válido; lo cuarentenado es **dudoso** y su default es **excluir** (inverso a `exclude_superseded: false`) |
| TTL `expires_at_ms` + `purge_expired` (MCP tool, `tools.rs:30`) | Semántica de expiración/recolección ya existente | TTL = ciclo de vida; cuarentena = ciclo de **revisión** |
| `confidence_score` de nodo (`src/node/unified.rs:41-42`, default 0.5 :92) | Confianza a nivel nodo ya ponderada en eviction/executor | Nivel registro, con semántica MGR-12 (asserted/derived) |

### 2.2 Campos propuestos (v2 — el ADR SCH-01 fija nombres finales)

| Campo | Tipo | Semántica | Migración |
|-------|------|-----------|-----------|
| `quarantined_at_ms` | `Option<u64>` | `None` = activo; `Some(t)` = en cuarentena desde t | `None` por default serde → determinista |
| `quarantine_reason` | `Option<String>` | Código estable (lowercase snake): `explicit_write` \| `unreviewed_import` \| `derived_promotion` \| `policy_match` (reservado v1.0) | `None` |
| `quarantined_by` | `Option<String>` | Principal que la aplicó, o `system:<op>` (ej. `system:dream_promote`) | `None` |
| `quarantine_review_due_ms` | `Option<u64>` | Deadline de revisión (señal de expiración T3). **No dispara auto-promoción** | `None` |

Wire de consulta (SCH-05): `include_quarantined: bool` con `#[serde(default)]` en `MemoryListOptions` (`src/sdk/types/record.rs:142-160`) y `MemorySearchRequest` (`src/sdk/serialization/vector_types.rs:100-145`), hasheado en el fingerprint del cursor (`page.rs:157` — mismo patrón que `exclude_superseded`).

Ops propuestas (contrato; implementación SCH-05):

```text
quarantine_apply(ns, key, reason?) -> MemoryRecord   # T1d: cuarentenar un registro existente (admin/reviewer)
quarantine_promote(ns, key)        -> MemoryRecord   # T2: salida aprobada por reviewer
quarantine_reject(ns, key)         -> ()             # T4: rechazo (delete con audit)
MemoryInput { ..., quarantine: bool }                # T1: entrada write-time (aditivo, default false)
import_* (opción quarantine: bool)                   # T1c: entrada por import (aditivo)
```

### 2.3 Nota de dos capas (integración crítica)

Hay dos records homónimos: el **storage record** (`src/sdk/types/record.rs:102-138`, con `payload: String`) y el **L1 payload record** de vanta-memory (`vanta-memory/src/core/abstractions/types.rs:71-130`, con `heat` y `superseded_by`, sin TTL). Decisión propuesta: el estado de cuarentena vive **canónico en el storage record** (es quien gobierna search/list/retrieval); el mirror L1 y los espejos de export (`MemoryExportLine`, `SnapshotRecord` en `src/sdk/version_history.rs:76-90`) se migran en SCH-02 para que el estado sobreviva export/import y snapshots. `auto_recall` lee L1 vía storage (`read_session_records`/`read_namespace_records`), así que el gate puede leer el estado del storage record sin cambiar el payload L1. **Riesgo de divergencia** (doble fuente) → test de consistencia en SCH-05, precedente de mapeo `src/sdk/api/graph.rs:231`.

### 2.4 Compatibilidad (Hyrum)

`include_quarantined` default `false` con **cero efecto sobre datos legacy**: ningún registro pre-0.8.0 puede estar `Some(quarantined_at_ms)` (campo nuevo) → el default-exclude es un cambio observable solo para registros que el propio 0.8.0 escriba en cuarentena. Igual que ADR-028, los campos se omiten en el wire cuando son `None` (`skip_serializing_if`) y viajan con `#[serde(default)]` en export/import. **Regla One-Version**: extender `MemoryInput`/`MemorySearchRequest`/`MemoryListOptions` con campos opcionales — nunca un `MemoryInputV2`.

## §3. Estados + transiciones (contrato para SCH-01/SCH-05)

### 3.1 Estados

- **`active`** — default implícito (`quarantined_at_ms = None`). Visible en search/list por defecto.
- **`quarantined`** — explícito (`quarantined_at_ms = Some(t)`). Excluido por defecto de search/list; **nunca** inyectado; visible con opt-in (`include_quarantined`) y por get directo con estado a la vista (§5.1).

No hay un tercer estado en 0.8.0. `superseded` y `expired` son ejes ortogonales (§3.3, I3).

### 3.2 Tabla de transiciones (dueño + trigger + efecto + registro)

| # | Transición | Trigger (señal medible) | Dueño | Efecto | Registro/evidencia |
|---|-----------|-------------------------|-------|--------|--------------------|
| **T1** | active → quarantined | `quarantine=true` en el write (SDK/HTTP/MCP put o put_batch) | Principal que escribe (caller) | setea campos, `reason=explicit_write` | Audit `quarantine_enter`; respuesta incluye estado |
| **T1b** | active → quarantined | Promoción dream→L1 (`promote_dream_run` real, MEM-65) escribe con **origen derivado** → gate **default ON** (`quarantine_derived_promotion=true`) | Sistema (política config; configurable por host) | `reason=derived_promotion`, `quarantined_by=system:dream_promote` | Audit + `DreamRun.runner_label` + `consolidated` provenance (`dream/mod.rs:175-181`) |
| **T1c** | active → quarantined | Import con opción `quarantine=true` (`import_records`/`import_file`/`bulk_import_*`/`import_v2`) | Caller/operador del import | `reason=unreviewed_import` | Audit (`impl_export.rs:328-334` ya audita import) + `ImportReport.quarantined` (campo aditivo propuesto) |
| **T1d** | active → quarantined | Op admin `quarantine_apply(ns,key)` (detección post-hoc, revisión manual) | Reviewer/operador | setea campos con `reason` provisto | Audit `quarantine_enter` con principal |
| **T2** | quarantined → active (**promoción**) | Op explícita `quarantine_promote(ns,key)` | **Reviewer** (operador/admin; por defecto ≠ principal que escribió — separación auditable) | limpia campos de cuarentena; `updated_at_ms=now`; **version NO cambia** (estado ≠ contenido) | Audit `quarantine_promote` (principal + timestamp) |
| **T3** | quarantined → purged (**expiración**) | `quarantine_review_due_ms` vencido + acción `quarantine_expire_action` | Sistema (`purge_expired` si `purge`; default `keep`) | `keep` (default): métrica/audit de vencido, el registro **sigue** cuarentenado; `purge` (opt-in): delete, como TTL | Métrica `quarantine_overdue` + audit |
| **T4** | quarantined → purged (**rechazo**) | Op explícita `quarantine_reject(ns,key)` (destructiva; requiere confirmación) | Reviewer | delete del registro | Audit `quarantine_reject` |
| **T5** | quarantined → quarantined (**re-validación**) | Revisión periódica/marca de re-revisión | Reviewer | actualiza `quarantine_review_due_ms` | v1.0 — FIND |

### 3.3 Invariantes de la máquina

- **I1 — Nunca auto-promoción.** Ni TTL ni tiempo promueven: el ataque "wait-it-out" (MINJA: la memoria envenenada espera a ser recuperada) no puede ganar por reloj. Toda promoción es un acto explícito y auditado.
- **I2 — Sticky.** Un `put` sobre una key cuarentenada **conserva** el estado (un re-write no limpia la cuarentena; solo T2/T4 salen). Cierra el bypass "sobrescribo la key y la saco de cuarentena".
- **I3 — Ortogonalidad.** `quarantined_at_ms` es independiente de `superseded_by` y `expires_at_ms`: combinables (un registro superseded puede estar cuarentenado y viceversa); cada gate se evalúa por separado.
- **I4 — Toda entrada tiene salida.** T2 (promover), T4 (rechazar), T3 `keep`+señal de vencido (revisar). No existe cuarentena permanente sin superficie de revisión (mitiga riesgo #2 del bloque: "estados sin transición de salida").
- **I5 — Observable, nunca silencioso.** Estado visible en get/export; audit events; métrica de cola. El usuario ve que un registro existe y está aislado — no desaparece sin señal.
- **I6 — Sin falsos positivos automáticos.** 0.8.0 solo cuarentena con señales explícitas (flag/op/derived/default configurable). Ninguna heurística de contenido aísla registros por sí sola (mitiga riesgo #1: aislamiento de contenido legítimo; ver §3.4 para las señales diferidas).

### 3.4 Señales de entrada — alcance 0.8.0 vs diferidas

| Señal | Superficie | 0.8.0 | Por qué |
|-------|-----------|-------|---------|
| Flag explícito (`quarantine=true`) | Todas las writes | ✅ | Control directo del host; cero falsos positivos |
| Promoción derivada de dream (default ON) | dream → L1 | ✅ | El vector deep-poisoning con mayor ganancia para el atacante (§4.2) |
| Opción de import | import/bulk/import_v2 | ✅ (opt-in; default = pregunta owner Q1) | Cadena de suministro de datos (§4.3) |
| Contradicción detectada en consolidación (`resolve_contradictions`, ownership :169-171) | dream | ⏸️ v1.0 / FIND | Solapa MGR-06 (conflict resolution); evitar doble semántica |
| Confianza baja (<umbral) | retrieval | ⏸️ opt-in de config, nunca default | Riesgo alto de falsos positivos; la abstención (§5.3) cubre el caso |
| Anomalías de tasa/comportamiento | API | ⏸️ v1.0 | Exige motor de políticas (MGR-04) → fuera del corte |

## §4. Threat model write-time por superficie

### 4.0 Modelo de adversario y clases de ataque (fuentes verificadas — §8)

- **Clase A — KB/memoria envenenada con trigger (AgentPoison):** contenido malicioso con trigger optimizado en la base recuperable; éxito >80% con <0.1% de poison rate y <1% de impacto benigno. Vulnera "sistemas que confían en knowledge bases no verificadas".
- **Clase B — Inyección query-only (MINJA):** el atacante **no** toca el storage: interactúa normalmente (queries) y logra que el agente escriba registros maliciosos en la memoria (bridging steps, indicación → acortamiento progresivo). Cualquier usuario puede influir la memoria.
- **Clase C — Persistencia por superficies confiables (OWASP ASI06 / MemoryTrap):** el contenido envenenado llega a una superficie que el sistema **sigue tratando como legítima** (memoria, hooks, config) y se propaga entre sesiones/proyectos/reboots. Claude Code v2.1.50 removió memorias del system prompt como mitigación.

Contraste con VantaDB: las tres clases aplican al camino **write → recuperación → inyección a prompt** (`auto_recall` → `<relevant-memories>`; MCP `inject_context`/`recall`). La cuarentena es la contención de **entrada**; la abstención es la señal de **salida**; el hash-chain (VER-01) será la evidencia encadenada de **transiciones** (no se implementa acá).

### 4.1 Superficie A — API HTTP / escrita directa (SDK y MCP)

- **Frontera:** `POST /api/v2/records*` (`src/server/router.rs:158,162` → `handlers.rs:248-265`), `import_v2` (`handlers.rs:769`), SDK embedded, y writes MCP (`tools.rs:30`, 37 tools no read-only).
- **Adversario:** principal autenticado con permiso de write en un namespace (o **dev mode sin API key**, warning explícito `middleware.rs:71-77`); un agente comprometido que usa tools legítimas; contenido externo que el extractor convierte en L1 (clase B).
- **Controles hoy:** auth en capas L1 (API key/JWT) → L2/L3 identidad deny-by-default (`middleware.rs:42-258`), RBAC por namespace en endpoints de records/search/list (:180-223), rate-limit de fallos de auth (:84-98), audit de eventos auth.
- **Gap:** la autenticación protege **quién** escribe, no **qué** se escribe; no hay clase de confianza por registro ni gate de contenido; un write autorizado entra directo a L1 y es inyectable por `auto_recall`.
- **Gate 0.8.0 (T1/T1d):** flag `quarantine` write-time para hosts que saben que el contenido es no verificado; `quarantine_apply` para aislamiento post-hoc; default-exclude en retrieval.
- **Residual:** contenido plausible envenenado sin flag sigue activo (detección automática = v1.0); MGR-04 (trusted/tainted namespaces) queda como FIND.

### 4.2 Superficie B — dream / consolidación (deep-poisoning)

- **Frontera:** `consolidate_session`/`write_dream_run` escriben en `dream/<session>/<run_id>` (`dream/mod.rs:509-527`); **el merge real a L1 será `promote_dream_run`** (hoy stub :614-622, preview MCP `dreams.rs:252-267`). Invariantes actuales: L1 **nunca** mutado por el módulo (:13-27), discard real (:593, MCP `dream_discard`).
- **Adversario:** contenido malicioso dentro de las sesiones consolidadas (clase B) que el runner LLM opcional reescribe, deduplica y "normaliza"; o un runner comprometido (**el runner es una extensión del host**: `Dreamer` trait :241-255). La consolidación **laundriza**: el contenido reaparece como "vista consolidada" con apariencia de curaduría.
- **Controles hoy:** aislamiento por namespace + originals intactos + `runner_label` auditable por run (:175-177) + provenance de contradicciones (:169-171) + discard barato. `dream_promote` MCP honestamente read-only mientras es stub.
- **Gap:** **no existe gate en la promoción**: cuando MEM-65 conecte el merge real, nada impide que la salida derivada (potencialmente envenenada) entre a L1 como activa. El atacante gana dos veces: el contenido anda por la memoria Y el reviewer lo ve "consolidado".
- **Gate 0.8.0 (T1b + T2):** la promoción escribe con **origen derivado → cuarentena default ON**; el reviewer aprueba por registro con T2. `DreamRun` queda como evidencia (inputs_scanned, merged/contradicted, runner_label, consolidated).
- **Checkpoint de contrato MCP:** al conectar el merge real, `dream_promote` debe cambiar `readOnlyHint: true` → `false` (`dreams.rs:111-117`) y `{mutated:false}` → recuento real; quien dependa del preview viejo es breaking **deliberado** (Hyrum: documentarlo en el ADR).
- **Residual:** un reviewer que aprueba sin leer (proceso humano, no código); runner comprometido con output limpio (indistinguible sin detección v1.0).

### 4.3 Superficie C — import (JSONL/archivo/stream)

- **Frontera:** `import_records` (`impl_export.rs:290-323`), `import_file` (:326-364), `bulk_import_*` (MCP, `openWorldHint: true` — host filesystem, `tools.rs:34`), `import_v2` HTTP (`handlers.rs:769`).
- **Adversario:** archivo de export malicioso (cadena de suministro: "importá esto"), export de una instancia comprometida re-importado en otra, o el propio agente escribiendo un export que luego importa (clase B cross-namespace/tenant).
- **Controles hoy:** `EXPORT_SCHEMA_VERSION` exacto (`mod.rs:35,522-531`), guard `read_only` (:294-298, 343-347), audit `import_file` (:328-334), errores contados por línea, inserts idempotentes por key (`:309-316`).
- **Gap:** el import es un canal de escritura **sin clasificación de procedencia** — todo lo importado queda activo; no hay forma de importar "en cuarentena" para revisar.
- **Gate 0.8.0 (T1c):** opción `quarantine` en las ops de import (reporte aditivo `quarantined`) + recomendación de uso para archivos externos; T1d permite aislar post-hoc lo ya importado.
- **Residual:** un import sin flag queda activo por decisión del operador (Q1); la revisión por delta de versión es v1.0.

### 4.4 Superficie D — MCP agent-facing (tools de escritura indirecta)

- **Frontera:** tools de escritura expuestas al agente: `memory_put*`, `import`, `bulk_import_*`, `dream_consolidate`, **`inject_context`** (`tools.rs:585-599`: "injects external state... for subsequent consolidation" — write a hilos para consolidación futura), `wiki_ingest` y `thread_*`.
- **Adversario:** prompt injection indirecta: el agente lee contenido externo (web, repo, archivo) y ejecuta tools legítimas (clase C / MemoryTrap: el payload no necesita un exploit, solo un flujo "normal").
- **Controles hoy:** validación de args en boundary por tool (patrón `dreams.rs:147-164`), hints MCP (:29-36), misma auth que la API cuando aplica.
- **Gap:** `inject_context` es un write de contexto externo sin marca de procedencia; su contenido alimenta consolidación → dream → (futuro) promoción.
- **Gate 0.8.0:** la contención efectiva es **en cascada**: el contexto inyectado no se promueve sin pasar por T1b/T2; los registros L1 originados por flujos MCP pueden marcarse con el flag write-time (el tool `memory_put` acepta `quarantine`). Documentar en `docs/api/MCP.md` (SCH-05) el circuito write→dream→promote.
- **Residual:** el contexto inyectado permanece en L0/hilos hasta la consolidación (aislamiento perfecto de L0 = v1.0, solapa EXE-07).

### 4.5 Riesgos residuales comunes

| Riesgo | Mitigación | Dueño/plazo |
|--------|-----------|-------------|
| Detección de poison sin señal (contenido plausible) | Fuera del slice; v1.0 (jueces/grounding — MGR-12 lo difiere igual) | FIND con dueño v1.0 |
| Falsos positivos por señales automáticas | 0.8.0 no incluye heurísticas automáticas (I6); señales diferidas son opt-in | SCH-05 review |
| Auditoría no encadenada (repudio) | VER-01 (F4) sobre WAL con PROV-O; citar, no duplicar | VER-01 |
| Separación reviewer/writer en single-user | Auditable (principal registrado); la separación dura llega con auth multiusuario/roles | Q3 |

## §5. Retrieval trust-aware + abstención (semántica para SCH-05)

### 5.1 Gates de lectura

- **Search/List:** excluir `quarantined` por defecto; `include_quarantined: bool` opt-in (default `false`), incluido en el **fingerprint del cursor** (`page.rs:157`) para que la paginación sea consistente. Implementación junto al filtro existente (`page.rs:317`).
- **Get por key:** devuelve el registro **con el estado visible** (acceso directo explícito; recomendado — Q5). Nunca 404 silencioso por cuarentena.
- **Export:** exporta el estado (los campos viajan); un export/import roundtrip preserva cuarentena (SCH-02/06).
- **CLI:** listados honran el default-exclude con flag `--include-quarantined` (SCH-05 opcional; la operación base es SDK/HTTP/MCP).

### 5.2 Gates duros de inyección (nunca cuarentenado, sin opt-in)

| Punto de inyección | Archivo | Regla |
|--------------------|---------|-------|
| `perform_auto_recall` (L1 → `<relevant-memories>`) | `vanta-memory/src/core/hooks/auto_recall.rs:198` | filtrar `quarantined` en la selección de records (read path) |
| MCP `inject_context` (write de contexto) | `vantadb-mcp/src/handlers/tools.rs:585` | no inyecta registros L1 cuarentenados; su propio write sigue T1 |
| MCP `memory_recall` (prepend_context) | `tools.rs:1818-1841` | hereda el filtro de `perform_auto_recall` |
| `context_assemble` (si toca L1) | `tools.rs` (readOnly) | verificar en SCH-05; mismo filtro |

### 5.3 Abstención selectiva

- **Semántica:** con umbral configurado (`confidence_threshold` / política equivalente — nombre final en ADR SCH-01), si tras filtrar no quedan candidatos suficientes, la respuesta lleva `abstained: true` + `reason` estable (`no_candidates_above_threshold` \| `all_quarantined`) en el wire de search/recall. **Nunca** "menos resultados" silenciosos.
- **Default OFF:** sin umbral configurado el comportamiento no cambia (cero breaking; Hyrum).
- **Ubicación:** campo aditivo `#[serde(default)]` en `MemorySearchRequest`/respuesta de página (`vector_types.rs`), propagado a HTTP (`handlers.rs` `SearchPageV2`), MCP y bindings (matriz de paridad SCH-07).

### 5.4 Interacción con MGR-12

La clase `asserted`/`derived` y el score por registro vienen de MGR-12/SCH-01/SCH-04. Acá se consumen así: (a) `derived` sin revisar es la señal de T1b (promoción de dream); (b) el retrieval trust-aware puede ordenar/filtrar por clase (opt-in) — sin cambiar el orden por defecto del ranking; (c) la abstención usa el score configurado, no clases hardcodeadas. Sin divergencia: canónico en el record (nota §2.3).

## §6. Preguntas owner (Cierre MGR)

1. **Default de import (Q1):** ¿`import*` cuarentena por defecto (seguro, cambia el flujo de import actual) o opt-in `quarantine=true` (compat, depende del operador)? *Recomendado: opt-in + recomendación documentada para archivos externos; bulk/stream con default ON a decidir.*
2. **Expiración (Q2):** ¿la expiración de cuarentena (`quarantine_review_due_ms`) con acción `purge` desechando el registro, o `keep` + señal de vencido (recomendado, evita pérdida por falso positivo)? ¿TTL default de review (ej. 30d) o sin deadline?
3. **Separación reviewer/writer (Q3):** ¿T2 exige principal ≠ escritor (hard) o basta registro auditable (recomendado para single-user; hard cuando haya roles)?
4. **Sticky vs overwrite (Q4):** ¿un `put` sobre key cuarentenada conserva la cuarentena (recomendado, I2) o un write del dueño con flag `quarantine=false` la limpia?
5. **Get por key (Q5):** ¿`get` devuelve cuarentenados con estado visible (recomendado) o requiere `include_quarantined` también?
6. **Alcance de `quarantine_apply` (Q6):** ¿op de 0.8.0 (aislar post-hoc, recomendado) o diferir a v1.0 junto con detección automática?

## §7. Plan de implementación (para SCH-01/02/05/06)

1. **SCH-01 (ADR, insumo):** fijar campos (§2.2), transiciones (§3.2), default-exclude + `include_quarantined`, ops promote/reject/apply, códigos de `reason`, semántica de abstensión y alcance 0.8.0 vs v1.0; incluir en el ADR de migración única los espejos de export/snapshot. Verificación: ADR revisado + tabla de reconciliación con MGR-10/12.
2. **SCH-02 (schema v2):** campos en `MemoryRecord`/`MemoryInput` + `MemoryExportLine`/`record_from_export_line` (`mod.rs:521-549`) + `SnapshotRecord`/`From` (`version_history.rs:76-148`); migración determinista `None`; roundtrip export/import.
3. **SCH-05 (operación):** `include_quarantined` en list/search + fingerprint; filtros duros en `auto_recall`/`inject_context`/`recall`; ops T1d/T2/T4 + audit; flag T1 y opción de import T1c; señal `abstained`; test de contención (`tests/quarantine_containment.rs`: dudoso no inyectado por defecto) + test de sticky + consistencia §2.3.
4. **SCH-06 (bordes/chaos):** TTL+quarantine, supersede+quarantine (I3), migración con cuarentenados (doble corrida byte-idéntica), crash mid-migración; comandos: `cargo nextest run --profile audit -p vantadb --test quarantine_containment` (+ suite de migración/chaos SCH-06).

## §8. Deuda, límites y notas

- **Deuda v1.0 (FIND):** motor de políticas ABAC/namespaces trusted/tainted (MGR-04 + Backlog:942 lo pedía → FIND con dueño); detección automática de poisoning (ML/jueces); re-validación periódica T5; aislamiento de L0; auditoría encadenada (VER-01 con hash-chain + PROV-O — `https://www.w3.org/TR/prov-o/`, citado, no duplicado).
- **Stop conditions respetadas:** sin motor de políticas en el corte (se recortó a estado + gates simples + FIND); hash-chain fuera (VER-01 lo cubre); no se tocó `src/` ni el research-doc de MGR-12 (co-batch intacto).
- **Contrato — verificación:** §3 (estados + transiciones con dueño/trigger) y §4 (threat model write-time por superficie API/dream/import) + §6 (preguntas owner) + §7 (plan de implementación) = Cierre MGR completo. Gap verificado con `rg`/lectura §1.
- **Fuentes externas (verificadas vía webfetch 2026-09-28 — GATE CITAS):**

| Fuente | URL | Uso |
|--------|-----|-----|
| AgentPoison (NeurIPS 2024) | https://arxiv.org/abs/2407.12784 | Clase A: backdoor vía poison de memoria/KB en agentes RAG |
| MINJA — Memory Injection Attacks (NeurIPS 2025; v5 2026-02) | https://arxiv.org/abs/2503.03704 | Clase B: inyección query-only — el atacante no toca storage |
| OWASP Top 10 for Agentic Applications 2026 (recurso) | https://genai.owasp.org/resource/owasp-top-10-for-agentic-applications-for-2026/ | ASI06: Memory & Context Poisoning |
| OWASP GenAI blog — "Memory Is a Feature. It Is Also an Attack Surface" (2026-05-13) | https://genai.owasp.org/2026/05/13/memory-is-a-feature-it-is-also-an-attack-surface/ | Persistencia = riesgo central; memoria como superficie de ataque |
| Cisco — MemoryTrap (2026-04-01) | https://blogs.cisco.com/ai/identifying-and-remediating-a-persistent-memory-compromise-in-claude-code | Caso real: compromiso persistente de memoria (write→persist→trust) |
| Azure Architecture Center — Quarantine pattern | https://learn.microsoft.com/en-us/azure/architecture/patterns/quarantine | Patrón de estados untrusted→validated→trusted; reporte con expiración; re-scan continuo |
| W3C PROV-O (Rec. 2013) | https://www.w3.org/TR/prov-o/ | Vocabulario de proveniencia para VER-01 (deuda citada) |

- **Skills (SDP):** `campaign-executor`, `progreso` (base, auto) + `deprecation-and-migration` (pin storage/schema) + `writing-guidelines`, `writing-plans` (base docs) + `documentation-and-adrs`, `security-and-hardening`, `source-driven-development`, `api-and-interface-design`, `spec-driven-development`, `coordinated-web-search`.
- **WIP ajeno PROHIBIDO:** `docs/dev/research/mgr-12-confianza.md` y demás archivos del co-batch F3 — no tocados. `src/` intacto (verificable con `git diff --name-only`).
