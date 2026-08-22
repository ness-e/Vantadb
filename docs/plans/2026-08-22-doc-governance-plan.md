# Plan de Ejecución: Gobernanza Documental — corrección integral post-auditoría

> **Campaign ID: 3f955847-b362-422f-a281-005bfbb62e0d
> **Inicio:** 2026-08-22
> **Estado: completed
> **Fuente:** `docs/reviews/auditoria-documentacion-2026-08-21.md` (Volumen I+II+Addendum) + decisiones del owner D1-D14 + respuestas de confirmación T0.x/T1.x/T3.x/T4.x/T6.x/T7.x registradas en §"Plan de Revisión" del informe.
> **Predecesores:** ninguno bloqueante. Convive con `2026-08-22-vanta-final-cierre.md` (P31) sin solaparse.
> **Condicionante:** Show HN Sept 2026 fecha próxima (D3) — Wave B bloqueante del lanzamiento.
> **Skills base cargadas al ejecutar:** writing-plans, planning-and-task-breakdown, progreso, ponytail(full); tareas docs cargan además writing-guidelines; GOV-B4/B5 cargan source-driven-development (API surface).

---

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 30 |
| 🟡 DEFER | 2 (fuera de este plan: corte release 0.6.0 [D5]; borrado físico de artefactos [D12] — ambos quedan como tickets) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 4 incógnitas abiertas · ⬇️ downhill = 96 steps pendientes

**Incógnitas (uphill) agregadas:**
1. GOV-B4 — approach del gate de paridad openapi↔router (script node vs gh-action grep).
2. GOV-B3 — lenguaje del guard de snippets (python vs powershell) para gate-docs-21.
3. GOV-D2 — corte exacto de campañas dentro del monolito progreso (líneas límite por campaña).
4. GOV-F2 — destino del Manual Estratégico 164KB (canonizar / archivar / dividir) — decisión owner tras auditoría.

**Tickets generados (van al Backlog, NO son tareas de esta campaña):**
- ACID 4a-4d rollback multi-capa (INV-010) — primera campaña post-launch, gate vanta-arch (D14/T7.1).
- CLI flags `restore --dry-run` / `doctor --fix` / subcomando `verify` backups (D4b).
- Release triage semver 0.6.0 (D5 — diferido hasta decisión del owner).
- MKT-18h wheels ARM64 — re-verificar contra GOV-A8 antes de ejecutar.

---

## Tasks

### Task 0 — Micro-fixes task-system (D13: ejecución directa)

#### Task 1: GOV-T01 — TIR-02a: recovery time en evals/dora.mjs
- **Appetite:** max 1h
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠
- **Archivos clave:** `evals/dora.mjs`, `.opencode/task-system/enforcement/verify-log.jsonl` (read-only)
- **Verificación real:** ✅ CÓDIGO-REAL — `verify-log.jsonl` existe y se puebla (23 entradas, campos ts/taskId/command/passed); `dora.mjs` ya lee el log para CFR pero no calcula recovery (TIR-02 investigación cerrada 08-17); 3 pares fail→pass medibles.
- **Gate Justificación:** decisión ya tomada en investigación; ~30 líneas sobre datos existentes; cero riesgo producto.
- **Gate Result:** ✅ DO
- **Contrato: verificacion: npx markdownlint-cli2 docs/operations/CONFIGURATION.md → 0 issues, EXIT=0 | evidencia: [rate_limit_rpm default=600 → src/config.rs:659 parse_env_or("VANTADB_RATE_LIMIT_RPM", 600u32), confianza alta] [0 desactiva rate limiter → src/cli_server.rs:263 if rpm>0, alta] [5 env vars faltantes añadidas con file:line → src/llm.rs:40,145,147; src/storage/engine/maintenance.rs:658; src/metadata.rs:22, alta] [fallback HOST real (config.rs:512), fallback PORT y flush_interval_ms ausentes del doc actual, alta] | artefactos: docs/operations/CONFIGURATION.md, .opencode/skills/campaign-executor/tasks/GOV-C6.md (tabla sweep) | invariantes: no tocar src/, docs/plans/, Backlog/progreso; sin git commit (orden orquestador) | deuda: commit pendiente — delegar al lead | queda_pendiente: commit del cambio por parte del orquestador
- **Pre-mortem:** (1) pairing ambiguo con taskId:null → filtrar entradas sin taskId y documentar caveat; (2) exitCode:-1 contaminando métrica → clasificar como no-ejecutado, no fallo.
- **Stop conditions:** appetite excedido → DEFER; si pairing produce >50% no-pareable → simplificar a conteo, no Δt.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟢 | log rotado/vacío en otra máquina | manejar archivo ausente con warning | primer run |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **DoD:** task=contrato verde · commit=`perf(task-system): dora recovery time (TIR-02a)` · release=N/A (tooling interno)
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-T01.md` (bajo demanda)
- **Notas:** decisión IMPLEMENTAR-recovery/DEFER-rework ya registrada en TIR-02 doc.

#### Task 2: GOV-T02 — TIR-04b: formalizar contenedor tasks/closed/
- **Appetite:** max 1h
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠
- **Archivos clave:** `.opencode/task-system/RULES.md`, `.opencode/skills/campaign-executor/SKILL.md`, `tasks/closed/`
- **Verificación real:** ✅ — convención existe con solo 2 ejemplos (DEVOPS-10/15); SARL ESCALATE no obliga mover task file; sin índice único de fallidas (TIR-04 doc).
- **Gate Justificación:** decisión tomada (contenedor citado, WONTFIT DLQ infra); es convención documentada, no código.
- **Gate Result:** ✅ DO
- **Contrato:** sección "Failed-task container" en RULES.md con las 3 reglas (mover task file al ESCALATE / re-procesamiento vía pending / índice `rg "❌ FAILED"`); grep "tasks/closed" matchea ≥2 archivos del sistema.
- **Pre-mortem:** (1) regla entra en conflicto con flujo archive→complete/ existente → aclarar que closed/ es solo para FAILED-escaladas; (2) prompts duplicados entre RULES y pipeline-run → citar fuente única.
- **Stop conditions:** si requiere tocar campaign-server.mjs (código) → escalar diseño, sale del appetite.
- **Risk Register:** 🟢×🟢 divergencia terminológica closed/ vs archive/ → glosario de una línea en la sección.
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda
- **Notas:** —

#### Task 3: GOV-T03 — TIR-08c: criterios de research en research-agent.md
- **Appetite:** max 30min
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢
- **Archivos clave:** `.opencode/task-system/prompts/research-agent.md`
- **Verificación real:** ✅ — los 3 criterios viven solo en agent-02-task-execution.md (no cargado por agentes); research-agent.md tiene 28 líneas hoy (TIR-08 doc verificado).
- **Gate Justificación:** mejora comportamiento runtime de sub-agentes research; ~6 líneas; jitter ya resuelto WONTFIT.
- **Gate Result:** ✅ DO
- **Contrato:** research-agent.md contiene criterio saturación<20% + broadening/narrowing + nota WONTFIT-jitter citando TIR-08.
- **Pre-mortem:** criterios genéricos sin acción → formularlos como stop-condition mecánica ("si ronda nueva añade <20% fuentes nuevas → STOP y sintetizar").
- **Stop conditions:** appetite excedido → DEFER.
- **Risk Register:** 🟢×🟢 ruido de prompt → mantener ≤8 líneas nuevas.
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1 step
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

---

### Wave A — Medición dinámica (D10: PRIMERO; alimenta Waves B-C)

#### Task 4: GOV-A1 — Coverage canónico: medir y fijar
- **Appetite:** max 2h (incluye build)
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `docs/architecture/adr/ADR-018*`, `docs/TEST_MAP.md`, `docs/operations/CI_POLICY.md`, `docs/avance/COBERTURA.md`, comando `cargo llvm-cov --workspace --summary-only`
- **Verificación real:** ✅ AUDITORÍA — 4 valores contradictorios coexistiendo: ≥59% (TEST_MAP:91, CI_POLICY), ≥80% (ADR-015/018), 80.55% CII Silver (progreso README:32), 81.40% root (coverage review 08-09).
- **Gate Justificación:** D7 decidió "medir y fijar"; sin cifra única, todo claim de coverage queda inválido (Regla 11 aplicada a docs internas).
- **Gate Result:** ✅ DO
- **Contrato:** llvm-cov corre exit 0; tras la tarea, exactamente UNA cifra aparece en los 4 documentos (grep cruzado sin contradicciones) con fecha y entorno.
- **Pre-mortem:** (1) build coverage falla por RAM/LLD SIGBUS histórico → usar flags documentados en TEST_MAP (runner 8GB); (2) cifra real <59% gate actual → NO cambiar el gate en esta tarea, registrar hallazgo y ticket aparte.
- **Stop conditions:** 2 intentos fallidos de llvm-cov → registrar imposibilidad, adoptar valor ADR-018 provisional marcado "por re-medir".
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟠 | cifra medida ≠ todas las declaradas | documentar delta, no maquillar; ticket si gap >10pts | resultado |
  | 🟡×🟡 | build coverage lento/flaky local | usar flags --build-jobs 2; timeout amplio | 1er fallo |
- **Cynefin:** 🟦 obvio (medición)
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 4 steps (medir → ADR → TEST_MAP+CI_POLICY → COBERTURA)
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 5: GOV-A2 — Reconciliar cifras de tests
- **Appetite:** max 1h
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** `docs/TEST_MAP.md`, `docs/reviews/*` (citas "2568+", "1902 passed", "1492 tests")
- **Verificación real:** ✅ AUDITORÍA — 3 cifras distintas citadas sin fuente única; ningún run reciente registrado como referencia canónica.
- **Gate Result:** ✅ DO
- **Contrato:** un run audit registrado (fecha+entorno+número) en TEST_MAP §Coverage; las citas viejas actualizadas o contextualizadas ("N al <fecha>").
- **Pre-mortem:** suite tarda >appetite → correr perfil default (excluye heavy) y documentar qué perfil es la cifra.
- **Stop conditions:** flakiness masivo → abortar medición, ticket de estabilidad primero.
- **Risk Register:** 🟡×🟢 número cambia cada semana → citar SIEMPRE fecha+perfil junto al número.
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 6: GOV-A3 — Probes CLI reales (doctor / backup / restore)
- **Appetite:** max 1h
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠
- **Archivos clave:** binario `vanta-cli`, DB temporal desechable; evidencia para GOV-B2
- **Verificación real:** ✅ AUDITORÍA — `Restore` solo acepta --input/--force/--rebuild (cli.rs:130-144); Doctor sin flags (cli.rs:144, diagnostics.rs:17); manifest CRC32C existe (backup.rs:30-100) sin comando que lo valide.
- **Gate Result:** ✅ DO
- **Contrato:** transcripción adjunta al task record: backup → cat manifest.json → restore a dir temporal → doctor sobre tmp → conteo records; procedimiento validado paso a paso.
- **Pre-mortem:** restore sobrescribe path equivocado → SIEMPRE en sandbox temp con --force explícito; nunca apuntar a db real.
- **Stop conditions:** cualquier comportamiento inesperado del CLI → capturar y detener (alimenta ticket, no se improvisa).
- **Risk Register:** 🟢×🔴 error humano sobre DB real → directorio de trabajo exclusivo temp/, doble-check path.
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 7: GOV-A4 — Harness de snippets de documentación
- **Appetite:** max 3h
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴
- **Archivos clave:** `dev-tools/validate_doc_snippets.py` (nuevo), `docs/tutorials/*.md`, `vantadb-python` (venv existente)
- **Verificación real:** ✅ AUDITORÍA — graph_bfs firma rota en 2 tutoriales (lib.rs:1612 vs doc); ef_search fantasma en glosario/hnsw.md:121; sin mecanismo alguno que valide snippets hoy.
- **Gate Result:** ✅ DO
- **Contrato:** script extrae bloques python etiquetados de tutorials/ + QUICKSTART, los ejecuta contra DB temporal y reporta PASS/FAIL por snippet; corrida inicial detecta las roturas conocidas (test negativo incluido).
- **Pre-mortem:** (1) snippets requieren API keys/embeddings → marcar esos bloques con directiva skip explícita en el doc; (2) falsos positivos por imports contextuales → cada snippet autocontenido con header común generado.
- **Stop conditions:** >40% snippets irreproducibles → reducir alcance a quickstart+tutorial01 y documentar el resto como no-ejecutable.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | mantenimiento del harness vs docs | directivas inline mínimas (# vanta-skip) | 2do falso positivo |
  | 🟡×🟠 | rompe gate-docs en CI | empezar local-only, integración a gate-docs como step separado GOV-B3 | pre-PR |
- **Cynefin:** 🟨 complicado — diseño del extractor depende del formato real de los md
- **Top 3 riesgos:** falsos positivos / deps externas / drift formato
- **Uphill/Downhill:** ⬆️ 1 (lenguaje/formato guard definitivo) · ⬇️ 5 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 8: GOV-A5 — Registros live crates.io / npm / PyPI
- **Appetite:** max 45min
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** consultas web a registries; salida anotada en filas MKT del Backlog
- **Verificación real:** ✅ AUDITORÍA — publicación 0.5.0 afirmada solo por docs internas; wheels ARM64 ausentes según MKT-18h (sin verificar live).
- **Gate Result:** ✅ DO
- **Contrato:** JSON/HTML de respuesta por registro archivado en task record; filas RELEASE-02/MKT-18h actualizadas con estado verificado + fecha.
- **Pre-mortem:** rate-limit o geo-bloqueo → reintentar con provider alternativo (Argus/firecrawl).
- **Stop conditions:** registry inaccesible 2 intentos → marcar "no verificado" explícito (Regla: no asumir).
- **Risk Register:** 🟢×🟢 datos cambian entre consulta y uso → timestamp en cada captura.
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

---

### Wave B — 🔴 Bloqueantes Show HN (público)

#### Task 9: GOV-B1 — case_studies ficticios → archive interno
- **Appetite:** max 1h
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴
- **Archivos clave:** `docs/case_studies/{rag_edge_device,agent_local_memory_ollama}.md` → `docs/archive/case-studies-unverified/`; `docs/master-index.md`, `docs/README.md`
- **Verificación real:** ✅ AUDITORÍA — clientes ficticios presentados como deployments reales sin disclaimer (EdgeSense/CodexAgent); riesgo reputacional directo pre-lanzamiento.
- **Gate Justificación:** D6 decidió eliminar del público; T0.1 precisó archivar internamente.
- **Gate Result:** ✅ DO
- **Contrato:** git mv preserva historial; README de carpeta destino con nota "material interno no-público, escenarios ilustrativos SIN verificación"; 0 referencias vivas fuera de archive (método AUD-007); gate-docs verde.
- **Pre-mortem:** (1) links externos/web apuntan a case_studies → grep global antes de mover; (2) markdownlint falla en nuevos paths → frontmatter copiado intacto.
- **Stop conditions:** descubrir que algún case study ES real verificable → separarlo, mantenerlo público con evidencia citada (Regla 11).
- **Risk Register:** 🟢×🔴 link roto en sitio web público → grep en web/src también | 🟡×🟢 SEO pierde páginas → redirect stub opcional.
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 4 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda
- **Notas:** commit `docs!: retire unverified case studies to internal archive`.

#### Task 10: GOV-B2 — Runbook de DR sin comandos fantasma
- **Appetite:** max 2h
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴
- **Archivos clave:** `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` (:142,:233,:266)
- **Verificación real:** ✅ AUDITORÍA+CÓDIGO — `restore --dry-run` y `doctor --fix` NO existen; runbook los usa como verificación DIARIA de backups; procedimiento sustituto validado en GOV-A3.
- **Gate Result:** ✅ DO
- **Contrato:** cada comando del runbook verificado contra `cli.rs` (grep bidireccional); procedimiento diario nuevo = restore a tmp + doctor + conteo; tabla de comandos alineada a CLI canónica; nota "verificación nativa pendiente → ticket CLI".
- **Pre-mortem:** (1) otro doc cita los mismos comandos fantasma → grep global "dry-run|doctor --fix" en docs/operations; (2) procedimiento nuevo demasiado lento para daily → ofrecer variante ligera checksum-manifest documentada como alternativa aceptada.
- **Stop conditions:** si la validación exige tocar código CLI → cortar aquí, solo docs (el flag va al ticket).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🔴 | operador sigue el runbook viejo cacheado | banner de revisión + changelog entry | merge |
  | 🟢×🟡 | procedimiento tmp consume disco | ruta temp configurable documentada | redacción |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda
- **Notas:** severidad elevada 🟡→🔴 en addendum de auditoría.

#### Task 11: GOV-B3 — Fix de snippets + guard anti-regresión
- **Appetite:** max 3h
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴
- **Archivos clave:** `docs/tutorials/03-migrating-from-chromadb.md:178`, `docs/tutorials/migration-from-lancedb.md:281`, `docs/glosario/hnsw.md:121`, `docs/FAQ.md:183,242`, URLs GitHub; `dev-tools/validate_doc_snippets.py` (de GOV-A4)
- **Verificación real:** ✅ AUDITORÍA — firmas reales verificadas: `graph_bfs(roots, max_depth)` (lib.rs:1612); SyncMode default = sync-per-write (wal.rs:338) vs FAQ "cada 5s"; URL GitHub inconsistente FAQ vs QUICKSTART.
- **Gate Result:** ✅ DO
- **Contrato:** harness de GOV-A4 pasa 100% sobre docs corregidos; grep "ef_search" en glosario = 0; una sola URL de GitHub en docs/ usuario; guard conectado como step opcional de gate-docs-21 (local-first, activable).
- **Pre-mortem:** (1) fix de fsync cambia el sentido técnico (¿default realmente sync-per-write?) → verificar wal.rs:338 y config default antes de redactar; (2) guard da falso verde → test negativo mantenido del A4.
- **Stop conditions:** discrepancia código-vs-doc no resoluble por docs (ej. bug real de SyncMode) → parar, ticket de código, doc marca "known issue".
- **Risk Register:** 🟡×🟡 tutorial Chroma usa API vieja en más lugares que los detectados → harness full-run revela resto | 🟢×🔴 URL incorrecta pública → elegir la canónica (ness-e/Vantadb usada en QUICKSTART/blog) y confirmar con owner si duda.
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 6 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 12: GOV-B4 — Regeneración completa openapi.yaml + gate de paridad
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴
- **Archivos clave:** `docs/api/openapi.yaml`, `src/cli_server.rs:215-260` (read), `.github/workflows/gate-docs-21.yml`
- **Verificación real:** ✅ AUDITORÍA — yaml define 3 paths vs ~29 reales; gate check-api-version solo valida campo version (gate-docs-21.yml:56-81), jamás paths.
- **Gate Justificación:** contrato REST público desprotegido semanas antes del lanzamiento.
- **Gate Result:** ✅ DO
- **Contrato:** count(paths yaml) == count(routes /api/v2/* en cli_server.rs); script de paridad (extrae routes del .rs, compara contra yaml) incluido y llamado desde gate-docs-21; schemas mínimo viable por endpoint (200/error shape).
- **Pre-mortem:** (1) router dinámico dificulta extracción estática → parseo por regex de .route("...") con lista manual de excepciones documentada; (2) schemas completos = rabbit hole infinito → contract-first: paths+métodos+respuestas top-level, schemas detallados solo endpoints core (query/search/records); (3) yaml gigante rompe markdownlint/lint CI → validar con swagger-cli bundle.
- **Stop conditions:** paridad no alcanzable estáticamente en appetite → generar yaml manual + TODO en gate, escalar approach.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🔴 | yaml diverge de nuevo en silencio | gate de paridad OBLIGATORIO en mismo PR | diseño |
  | 🟡×🟡 | endpoints sin shape estable aún (conversation/skill) | documentar como experimental (x-experimental) | generación |
  | 🟢×🟡 | conflictos con AUD-005 (sync previa) | leer historial del gate antes de editar | DISCOVERY |
- **Cynefin:** 🟨 complicado — elección de approach de extracción y nivel de schema
- **Top 3 riesgos:** extracción frágil / schemas infinitos / gate sin dientes
- **Uphill/Downhill:** ⬆️ 1 (approach gate) · ⬇️ 8 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda
- **Notas:** delegable a vanta-worker (lee cli_server.rs) + vanta-docs (redacción yaml).

#### Task 13: GOV-B5 — HTTP_API.md completo
- **Appetite:** max 4h
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴
- **Archivos clave:** `docs/api/HTTP_API.md`, `docs/api/openapi.yaml` (fuente generada en GOV-B4)
- **Verificación real:** ✅ AUDITORÍA — documenta 4 de ~29 endpoints; ejemplo LISP muerto en :108 (sintaxis declarada no-soportada).
- **Gate Result:** ✅ DO
- **Contrato:** todos los endpoints agrupados por dominio con request/response example real (derivado del yaml de B4); 0 ejemplos con sintaxis LISP; curl probado contra server local para ≥5 endpoints representativos (transcripción en task record).
- **Pre-mortem:** (1) duplicar yaml en md genera doble-mantenimiento → md narra y enlaza yaml como spec formal, ejemplos generados; (2) endpoints experimentales confunden → sección separada "experimental" con banner.
- **Stop conditions:** curl real falla en endpoint documentado → es bug de servidor: ticket, no doc-falso.
- **Risk Register:** 🟡×🟠 drift futuro md↔yaml → regla escrita "yaml es spec, md es guía" en cabecera | 🟢×🟡 auth examples con tokens fake claros.
- **Cynefin:** 🟦 obvio (una vez B4 listo)
- **Uphill/Downhill:** ⬆️ 0 (depende B4) · ⬇️ 5 steps
- **Dependencias:** GOV-B4
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 14: GOV-B6 — Skill MCP como fuente única (33 tools) + MCP.md stub
- **Appetite:** max 4h
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `skills/vantadb-mcp/references/api-reference.md`, `SKILL.md`, `.opencode/skills/vantadb-mcp/*` (hash-SAME), `docs/api/MCP.md` → stub
- **Verificación real:** ✅ CÓDIGO — dispatch registra exactamente 33 tools: 15 core (tools.rs) + 6 skill_* (skills.rs) + 8 code_* (code.rs) + 4 wiki_* (wiki.rs); tools/list extiende las 4 fuentes (tools.rs:180-184). Hoy: MCP.md=21, skill="15", real=33.
- **Gate Result:** ✅ DO
- **Contrato:** conteo "33" presente y correcto en api-reference.md; cada tool con schema resumido; hash-SAME skills↔.opencode/skills verificado; MCP.md = stub ≤20 líneas con link; test-mcp.py 4/4 sigue verde.
- **Pre-mortem:** (1) code_* tools requieren contexto codegraph no disponible en todos los hosts → documentar precondición por grupo; (2) stub MCP.md rompe links entrantes (master-index, backlog AUD-006) → grep de inbound links antes.
- **Stop conditions:** si expandir 33 schemas excede appetite → priorizar core 15 completos + tablas resumen de los otros 18, iteración 2 después.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟠 | divergencia futura skill↔server | regla P22 hash-SAME ya vigente + conteo en gate | cierre |
  | 🟡×🟡 | wiki_*/code_* cambian rápido (P30 reciente) | sección "volátil" con fecha de sync | redacción |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 6 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda
- **Notas:** decisión D8; delegable a vanta-docs con verificación lead.

---

### Wave C — Sincronización maestros

#### Task 15: GOV-C1 — Filtro nextest inefectivo + TEST_MAP binarios
- **Appetite:** max 1h
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴
- **Archivos clave:** `.config/nextest.toml:27`, `docs/TEST_MAP.md:83,86`
- **Verificación real:** ✅ AUDITORÍA — filtro `binary(python_sdk_boundary)` no matchea nada (binario real: `tests/api/python.rs`); TEST_MAP cita `hnsw_recall_certification` cuando el binario es `hnsw_recall`.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest list --profile default` muestra el filtro aplicando (binario excluido); TEST_MAP sin binarios inexistentes (verificado contra ls tests/**).
- **Pre-mortem:** renombrar binario en vez del filtro podría ser mejor largo-plazo → decidir menor-cambio: corregir filtro+doc ahora; rename como opción en ticket.
- **Stop conditions:** si el filtro era intencional para un binario planeado → confirmar en git history antes de cambiar.
- **Risk Register:** 🟡×🟠 exclusión era deseada por tiempo de ejecución → verificar duración del binario python al aplicar; si lento, mover a heavy profile explícito.
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda
- **Notas:** SYNC-01 del Volumen I — único hallazgo con impacto CI real.

#### Task 16: GOV-C2 — Backlog ↔ campañas P29/P30/P31 + MEM-43
- **Appetite:** max 2h
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠
- **Archivos clave:** `docs/Backlog.md` (:703-705, Exec Summary), `docs/plans/2026-08-22-vanta-final-cierre.md`, `docs/plans/archive/*p29*,*p30*`, `docs/progreso/BACKLOG_HISTORY.md`
- **Verificación real:** ✅ AUDITORÍA — MEM-43 merged (commit a0bcb112) pero ❌ Pendiente en Backlog:703; P29/P30/P31 sin sección pese a "single source of truth".
- **Gate Result:** ✅ DO
- **Contrato:** fila MEM-43 ✅ con hash; MEM-44 estado real contrastado con plan; secciones P29/P30 (cerradas, puntero a archive) y P31 (activa, 8 tasks) presentes en Backlog; contador actualizado coherentemente (ver GOV-C7).
- **Pre-mortem:** (1) MEM-44 avanzó mientras tanto → verificar git log antes de escribir estado; (2) numeración P29/P30 informal en planes vs Backlog → normalizar nombres exactos citando plan files.
- **Stop conditions:** conflicto con edición simultánea del Backlog (P31 activa) → coordinar con sesión P31 antes de editar.
- **Risk Register:** 🟡×🟡 doble-edición concurrente → hacer en sesión dedicada, no durante /pipeline run de P31.
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 4 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 17: GOV-C3 — Purga de referencias muertas del Backlog
- **Appetite:** max 1h
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠
- **Archivos clave:** `docs/Backlog.md` (10 refs audit-reports/, REPORTE_EVALUACION_COMPLETO.md ×2, reviews FULL_CODEBASE_AUDIT_2026-07-11 y PROJECT_FULL_REVIEW_2026-07-13)
- **Verificación real:** ✅ AUDITORÍA — Test-Path falla para todas las rutas citadas.
- **Gate Result:** ✅ DO
- **Contrato:** método AUD-007 (regex links + Test-Path) = 0 rotas en Backlog.md; contenido histórico reescrito como mención textual sin link o apuntando a su ubicación real (reviews/ o ARCHIVO_HISTORICO).
- **Pre-mortem:** borrar contexto histórico útil → preferir reescribir a "reporte archivado, ver X" sobre eliminación seca.
- **Stop conditions:** referencia necesaria para trazabilidad de tickets vivos → crear REDIRECT.md en vez de quitar.
- **Risk Register:** 🟢×🟡 perder origen de tickets antiguos → mapear destino real antes de reemplazar cada ref.
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 18: GOV-C4 — Regeneración completa master-index.md
- **Appetite:** max 3h
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `docs/master-index.md`, árbol docs/ real (30+ carpetas)
- **Verificación real:** ✅ AUDITORÍA — congelado 07-21; 2 enlaces rotos (:184,:192); no indexa 15 carpetas ni 3 docs api nuevos; frase blog incorrecta (:161).
- **Gate Result:** ✅ DO
- **Contrato:** método AUD-007 = 0 enlaces rotos; todas las carpetas de primer nivel indexadas o explicadamente excluidas (_templates/.obsidian); frontmatter last_reviewed actualizado.
- **Pre-mortem:** (1) indexar todo lo vuelve obsoleto enseguida → incluir regla de mantenimiento en cabecera; (2) carpetas internas (.obsidian) no deben listarse → sección "no indexado" deliberada.
- **Stop conditions:** —
- **Risk Register:** 🟡×🟡 drift recurrente → añadir check de enlaces al gate-docs (reusa método AUD-007) como follow-up.
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 4 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 19: GOV-C5 — operations/master-index.md completar
- **Appetite:** max 45min
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** `docs/operations/master-index.md` (lista 26/32)
- **Verificación real:** ✅ AUDITORÍA — faltan 7 archivos (chaos-testing, ci-cd-guide, pilot-* ×3, TEST_MAP, self).
- **Gate Justificación:** índice canónico de operations incompleto; fix mecánico de bajo riesgo.
- **Gate Result:** ✅ DO
- **Contrato:** listing dir == index (diff vacío).
- **Pre-mortem:** (1) nuevos archivos llegan a operations/ sin indexar de nuevo → añadir regla "todo doc nuevo se agrega al índice en el mismo PR" en la cabecera.
- **Stop conditions:** —
- **Risk Register:** 🟢×🟢 drift recurrente → regla cabecera mitiga.
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1 step
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 20: GOV-C6 — CONFIGURATION.md sincronizada
- **Appetite:** max 2h
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `docs/operations/CONFIGURATION.md`, `src/config.rs:299,659`, `src/llm.rs:40,132,147`, `src/metadata.rs:22`
- **Verificación real:** ✅ AUDITORÍA — rate_limit_rpm doc=100 vs real=600; 4 env vars usadas no documentadas (VANTA_EMBEDDING_PROVIDER, VANTA_OPENAI_API_KEY/MODEL, VANTADB_REPORTED_VERSION); PORT fallback y flush_interval_ms fantasma.
- **Gate Result:** ✅ DO
- **Contrato:** cada env var documentada tiene grep hit en src/ y viceversa para defaults críticos; tabla de defaults spot-checkeada (≥10 vars) con config.rs como fuente.
- **Pre-mortem:** (1) más drift del encontrado (auditoría fue muestra) → sweep completo de `VANTA*_`/`VANTADB*_` vs doc en la misma pasada; (2) env var deprecated aún soportada → marcar "legacy" no borrar.
- **Stop conditions:** —
- **Risk Register:** 🟡×🟠 operador dimensiona capacidad con número falso → rate_limit fix prioritario dentro de la tarea.
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 4 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 21: GOV-C7 — Contador de tareas del Backlog: corrección + regla
- **Appetite:** max 30min
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** `docs/Backlog.md` header + ROADMAP banner
- **Verificación real:** ✅ AUDITORÍA — "~24 abiertos" declarado vs 45 filas ❌ reales (conteo rg).
- **Gate Result:** ✅ DO
- **Contrato:** header dice el número correcto con fecha; regla escrita "actualizar en cada sync contando filas ❌"; ROADMAP banner apunta al Backlog sin cifra puntual.
- **Pre-mortem:** número vuelve a diverger mañana → la regla + vínculo a GOV-C2 (sync ritual) mitiga; automatización completa descartada (T1.4 decisión).
- **Risk Register:** 🟢×🟢 —
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda
- **Dependencias:** GOV-C2 (para contar sobre estado ya sincronizado)

---

### Wave D — Estructura / taxonomía (D9)

#### Task 22: GOV-D1 — avance/activo catch-up + dominios faltantes
- **Appetite:** max 4h
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `docs/avance/activo/{vanta-memory,vanta-proxy,context-engine}.md` (nuevos), `meta.md`, commits git agosto
- **Verificación real:** ✅ AUDITORÍA — mirror roto respecto a su contrato: dominios enteros ausentes; congelado 20/08; MEM-43/44 sin registrar en ningún lado.
- **Gate Result:** ✅ DO
- **Contrato:** cada commit MEM-*/proxy de agosto aparece en su dominio (muestreo cruzado git log ↔ archivos); contrato de meta.md actualizado a la realidad (frecuencia, dominios); MEM-43/44 registrados.
- **Pre-mortem:** (1) reconstruir retroactivo enorme → catch-up por campaña (P27/P29/P30/P31) no por commit individual; (2) mirror vuelve a pudrirse → regla de actualización en ritual progreso.
- **Stop conditions:** si catch-up retroactivo supera appetite → documentar solo estado actual + puntero a BACKLOG_HISTORY, retro-completa queda ticket.
- **Risk Register:** 🟡×🟡 triple-fuera (Backlog/planes/avance) diverge → GOV-C2 va antes; este task consume su salida.
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 5 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda
- **Dependencias:** GOV-C2

#### Task 23: GOV-D2 — Split del monolito progreso/README.md por campaña
- **Appetite:** max 1d
- **Esfuerzo:** 🔴 | **Prioridad:** 🟡
- **Archivos clave:** `docs/progreso/README.md` (372KB, 4302 líneas) → `progreso/campanas/*.md` + índice
- **Verificación real:** ✅ AUDITORÍA — append-log sin TOC, duplicaciones (evento residuo ×3), resumen ejecutivo stale ("FASE 4").
- **Gate Result:** ✅ DO
- **Contrato:** README ≤50KB = índice + resumen vivo; cada campaña ≥1 archivo en campanas/; dedup del evento triplicado; 0 links rotos hacia progreso/ desde otros docs (grep inbound antes de mover).
- **Pre-mortem:** (1) cortes de campaña difusos en el monolito → usar headers de fase/sync como delimitadores, no fechas; (2) scripts/gates que leen README (validate-docs-coverage?) asumen estructura → verificar consumidores con grep antes de partir.
- **Stop conditions:** consumidores automáticos rotos no arreglables en appetite → revertir a índice-sin-split y escalar approach.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟠 | pérdida de contenido en el movimiento | diff neto de bytes por campaña + spot-check aleatorio | post-split |
  | 🟡×🟡 | wikilinks obsidian rotos | sweep AUD-007 extendido a [[..]] | cierre |
- **Cynefin:** 🟨 complicado — límites de campaña exigen juicio editorial
- **Uphill/Downhill:** ⬆️ 1 (corte exacto de campañas) · ⬇️ 8 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 24: GOV-D3 — Revivir bitácora.md
- **Appetite:** max 30min (+ tiempo del owner)
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢
- **Archivos clave:** `docs/progreso/bitacora.md`
- **Verificación real:** ✅ AUDITORÍA — última narrativa 27/07; muerta durante la actividad más intensa del proyecto.
- **Gate Result:** ✅ DO (revivir, T3.3)
- **Contrato:** entrada narrativa nueva fechada (draft preparado por lead, articulada/editada por owner — Regla 5 forcing function) + regla de uso en frontmatter.
- **Pre-mortem:** revivir y abandonar de nuevo → regla explícita de cuándo escribir (cierre de campaña, no daily).
- **Risk Register:** 🟢×🟢 —
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda
- **Notas:** el texto narrativo lo escribe el owner (human-in-loop), el lead prepara bullet points de hechos.

#### Task 25: GOV-D4 — Migración física Investigaciones/ → research/
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟡
- **Archivos clave:** `docs/Investigaciones/**` (~58 archivos) → `docs/research/`; citas en Backlog/planes/informe auditoría
- **Verificación real:** ✅ AUDITORÍA — dos convenciones conviviendo; cargo-check-optimizacion.md citado e inexistente; ID INV-019 colisionado.
- **Gate Result:** ✅ DO (T3.4: migración física)
- **Contrato:** 0 .md sueltos fuera de research/; convención documentada (campañas PLAN/NN/SYNTHESIS vs single-file) en research/README; método AUD-007 = 0 rotas globales; INV-019 renumerado con nota.
- **Pre-mortem:** (1) git mv masivo rompe blame/history percibido → git mv preserva; comunicar en commit message; (2) citas con path viejo en ~30 archivos → sweep AUD-007 + sed dirigido.
- **Stop conditions:** conflicto con P31 escribiendo en Investigaciones/ ese día → coordinar ventana.
- **Risk Register:** 🟡×🟡 referencias en .opencode/prompts a Investigaciones → grep allí también | 🟢×🟡 Obsidian vault links [[..]] → sweep específico.
- **Cynefin:** 🟨 complicado (convención de clasificación por archivo)
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 6 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 26: GOV-D5 — ADR-026 a adr/
- **Appetite:** max 30min
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** `docs/architecture/ADR-026-vanta-studio-fase3-rest-dashboard.md` → `docs/architecture/adr/`; citas en Backlog DESKTOP-27 e informe auditoría
- **Verificación real:** ✅ AUDITORÍA — único ADR fuera de adr/ (los demás 34 viven en adr/)
- **Gate Justificación:** convención de ubicación rota; fix trivial con git mv + grep de citas
- **Gate Result:** ✅ DO
- **Contrato:** archivo vive en adr/ junto a sus pares; método AUD-007 = 0 citas con path viejo
- **Pre-mortem:** (1) wikilinks Obsidian al path viejo → sweep específico [[ADR-026]]
- **Stop conditions:** —
- **Risk Register:** 🟢×🟢 —
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 27: GOV-D6 — wasm/CRASH_MODEL.md actualizar vs PERF-08
- **Appetite:** max 1h
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** `docs/wasm/CRASH_MODEL.md`, `vantadb-wasm/src/lib.rs:261-268,749` (read)
- **Verificación real:** ✅ AUDITORÍA — doc afirma "serialize ALL records / no incremental persistence"; PERF-08 introdujo persistencia diferencial (solo records cambiados); APIs connect_idb/save_idb/db_state.json sí existen ✅
- **Gate Justificación:** claim técnico falso sobre modelo de crash/persistencia WASM
- **Gate Result:** ✅ DO
- **Contrato:** §persistencia describe modelo diferencial con evidencia file:línea; sin claims "ALL records" residuales (grep)
- **Pre-mortem:** (1) PERF-08 tiene gaps conocidos documentados en WASM_STANDALONE.md → citar tabla de gaps en vez de sobreprometer
- **Stop conditions:** si el modelo real difiere de ambos docs → parar y escalar a vanta-worker para verificación de comportamiento
- **Risk Register:** 🟡×🟢 drift futuro → fecha de sync en cabecera
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 28: GOV-E1 — Documento de propuestas de borrado/aprobación
- **Appetite:** max 45min
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢
- **Archivos clave:** `docs/reviews/propuesta-limpieza-artefactos-<fecha>.md` (nuevo)
- **Verificación real:** ✅ AUDITORÍA — 7 candidatos: book/book/ (build commiteado), __pycache__/, TDAM-VANTADB/ vacía, _run_stdout.md (log crudo), DESIGN_RULES.md duplicado, .obsidian/ decisión, book/src/blog stubs.
- **Gate Result:** ✅ DO (la propuesta es la tarea; el borrado NO)
- **Contrato:** documento con impacto por ítem (Regla 0: contenido, refs entrantes/salientes, veredicto) y checklist de aprobación por ítem; NINGÚN borrado en este PR.
- **Pre-mortem:** aprobar en bloque sin leer → presentación item-por-item con recomendación explícita.
- **Stop conditions:** —
- **Risk Register:** 🟢×🟢 —
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

---

### Wave F — Auditoría segunda ola (D11; DESPUÉS de correcciones, T6)

#### Task 29: GOV-F1 — Auditoría raíz pública (README ×2 + governance files)
- **Appetite:** max 3h
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `/README.md`, `/README_ES.md`, `/CONTRIBUTING.md`, `/SECURITY.md`, `/SUPPORT.md`, `/CLA_*.md`
- **Verificación real:** 🟡 VERIFICAR — nunca auditados; claims de features/badges/links por contrastar contra código y registries (consume GOV-A5/A8 outputs).
- **Gate Result:** ✅ DO
- **Contrato:** reporte `docs/reviews/auditoria-raiz-publica-<fecha>.md` con tabla finding/evidencia/severidad; fixes triviales aplicados inline, resto tickets.
- **Pre-mortem:** scope creep hacia marketing copy → límite: exactitud técnica y links, no tono.
- **Risk Register:** 🟡×🔴 claim público falso pre-Show-HN → priorizar findings 🔴 primero.
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 4 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

#### Task 30: GOV-F2 — Auditoría zonas internas (Manual Estratégico, SKILLS-MANIFEST, .opencode/, integrations/)
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟡
- **Archivos clave:** `VantaDB_Manual_Estrategico_Unified.md` (164KB), `SKILLS-MANIFEST.md`, `.opencode/{AGENTS.md,agents/,rules/,references/}`, `integrations/*/README*`, `providers/*/tests` claims, `.github/workflows/*` profundo, `docs/plans/archive/` (46)
- **Verificación real:** 🟡 VERIFICAR — nunca auditados.
- **Gate Result:** ✅ DO
- **Contrato:** reporte consolidado `docs/reviews/auditoria-zonas-intocadas-<fecha>.md`; para el Manual Estratégico: recomendación explícita canonizar/archivar/dividir (uphill #4 → decisión owner); SKILLS-MANIFEST: conteo real vs 154 declarado; workflows: cada CATEGORY tag justificado vs Regla 2.
- **Pre-mortem:** (1) 164KB imposible en appetite → auditoría estructural (TOC, fechas, claims mayores) + veredicto, no línea-por-línea; (2) .opencode audit toca sistema vivo → read-only estricto, findings a tickets.
- **Stop conditions:** hallazgo que invalide premisa de este plan (ej. convención distinta en .opencode) → plan-adjust event documentado.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | hallazgos nuevos expanden alcance | van a tickets, NO a este plan | cierre |
  | 🟢×🟡 | SKILLS-MANIFEST drift masivo | regeneración automática propuesta como ticket | conteo |
- **Cynefin:** 🟨 complicado
- **Uphill/Downhill:** ⬆️ 1 (destino Manual Estratégico) · ⬇️ 7 steps
- **Estado:** ⬜ PENDING
- **Task file:** bajo demanda

---

## DoD multi-nivel (aplica a TODAS las tareas)

| Nivel | Gate |
|-------|------|
| Task | contrato mecánico ✅ via `campaign_verify_cmd` · task file sync · recitation |
| Commit | conventional commit · verify_changed.ps1 (docs: markdownlint gate) · deuda neta ≤0 |
| Release | N/A salvo GOV-B* (changelog entry vía commit type docs!/docs:) |

## Eventos plan-adjust

```
plan-adjust [2026-08-22]: creación inicial — triaje 30 DO / 2 DEFER (release, borrados físicos)
- ⬆️ uphill inicial: 4 (gate-paridad, guard-lang, cortes-campaña, destino-manual)
- ⬇️ downhill inicial: 96 steps
```

## Al finalizar cada wave

`skill progreso` → migrar filas GOV completadas a progreso → recitation → próximo wave.

=== RECITATION ===
Campaign ID: por asignar (`campaign_session_track create` al arrancar)
Objetivo activo: GOV-C6 — Sweep bidireccional env vars en CONFIGURATION.md
Estado: completed
Última acción: Corregido rate_limit_rpm 100→600 (config.rs:659) + nota 0=disabled (cli_server.rs:263); corregido flush_threshold 10000→None (config.rs:727); añadida sección 'Environment Variables Outside VantaConfig' con VANTA_EMBEDDING_PROVIDER/VANTA_OPENAI_API_KEY/VANTA_OPENAI_MODEL/VANTA_BACKUP_DIR/VANTADB_REPORTED_VERSION; sweep bidireccional 44 vars código vs doc (5 añadidas, 0 eliminadas, 2 fantasmas verificados reales/inexistentes); spot-check 14 defaults; markdownlint exit 0; task record creado con tabla resumen
Resultado: OK
Próxima acción: Orquestador: commitear docs/operations/CONFIGURATION.md y asignar próxima tarea del plan
Contrato: node evals/dora.mjs exit 0 + seccion Recovery con pares
Próxima tarea si completa: siguiente según plan file

---

## Estado de ejecución (live)

| ID | Estado | Evidencia |
|----|--------|-----------|
| GOV-T01 | ✅ | commit 1c7660dc — node evals/dora.mjs exit 0, Recovery Time con 12.56h/28.59h/16.8s (dora.md:304-310) |
| GOV-T02 | ✅ | commit 1c7660dc — RULES.md Apéndice B + subagent-recovery ESCALATE cita tasks/closed (2 archivos) |
| GOV-T03 | ✅ | commit 1c7660dc — research-agent.md criterios saturación<20%/broadening/WONTFIT-jitter TIR-08 |
| GOV-A1 | ⬛ CANCELADO por stop condition | 2× llvm-cov ICE rustc 0xc0000409 Windows (intentos: default, -j 2 mal aplicado corregido a flag llvm-cov, interrumpido) → fallback pre-autorizado aplicado: cifra canónica ADR-018 (root ≥80%, baseline 81.40%) fijada en TEST_MAP+CI_POLICY+progreso con marca "re-medición pendiente". Commit  6d8c619. Ticket: llvm-cov ICE local. |
| GOV-A2 | ✅ | commit 4fc8be24 — cifra canónica 2034/2034/1 skip (nextest default, 122s, Windows 2026-08-22) en TEST_MAP:92 |
| GOV-A3 | ✅ | probes validados end-to-end en sandbox temp (put→backup manifest 36 files→restore --force→doctor exit 0→get recupera); procedimiento diario listo para GOV-B2; sin cambios de archivo |
| GOV-A4 | ✅ | commit d147df5d — validate_doc_snippets.py: 21 PASS/31 FAIL/6 SKIP determinístico ×2; detecta graph_bfs ×2 + hallazgos extra (add_edge string IDs, IndentationError 05:133, input() interactivo 01:169) → insumo GOV-B3 ampliado |
| GOV-A5 | ✅ | read-only: 0.5.0 live confirmado crates/PyPI/npm (2026-08-01 coordinado ~13h); MKT-18h wheels ARM64 ausentes estructuralmente; MKT-18f adapters 404 ×4; hallazgo nuevo: descripción PyPI recomienda TestPyPI; Formula SHA placeholders vigentes |

plan-adjust [2026-08-22]: GOV-A1 gate DO→⬛ CANCELADO (stop condition appetite/ICE) — fallback documentado en la tarea misma; sin impacto downstream (B3 usa harness de A4, no coverage).

### Wave B cierre (2026-08-22)

| ID | Estado | Evidencia |
|----|--------|-----------|
| GOV-B1 | ✅ | 8b21733 — mv a archive/case-studies-unverified + README disclaimer + stubs book + refs graphrag/skills limpias |
| GOV-B2 | ✅ | 9ce238cd — runbook sin fantasmas, 11 comandos verificados bidireccional, +3 fantasmas extra corregidos (CONFIGURATION:330, DEPLOYMENT:499/502), §3.1 Daily Backup Verification |
| GOV-B3 | ✅ | 79ae8556 — harness 31 FAIL→0 FAIL (34 PASS/24 SKIP justificados); hallazgos extra: put_batch str-only, \ no soportado py-sdk, incidente 68GB temp resuelto |
| GOV-B4 | ✅ | 465673af — openapi 35 paths/40 ops paridad exacta + scripts/check_openapi_parity.mjs (test negativo OK) + gate en gate-docs-21.yml |
| GOV-B5 | ✅ | (commit arriba) — 35/35 endpoints, 13+ curl reales, regla yaml-spec/md-guía; tickets drift: IQL grammar yaml, GraphTraversalBody roots/max_depth, search requiere rebuild-index previo |
| GOV-B6 | ✅ |  8e272d8 — 33 tools documentadas (hash-SAME ×3 pares), test-mcp.py 4/4, MCP.md stub 12 líneas; caveat: binario publicado v0.5.0 expone solo 15 core — los 18 llegan con próximo release |

Tickets derivados Wave B: drift yaml↔real (B5) · harness temp-leak (B3) · put_batch str-only doc-code gap · URL vantadb-examples · binario release pendiente tools nuevos.

### Wave C cierre (2026-08-22)

| ID | Estado | Evidencia |
|----|--------|-----------|
| GOV-C1 | ✅ | 67384785 — nextest.toml filtra binarios reales (python/hnsw_recall), verificado con cargo nextest list; TEST_MAP alineado |
| GOV-C2 | ✅ | 89b0b484 — sección P29/P30/P31 en Backlog (MEM-43/44/45 ya migradas por sesión SDKB en 48016b89 — sin duplicar) |
| GOV-C3 | ✅ | 89b0b484 — nota GOV-C3 en Referencias Cruzadas: 0 links md rotos (las 15 refs eran backticks históricos); reportes disueltos documentados |
| GOV-C4 | ✅ | 1655fede — master-index regenerado: 136 links verificados, 0 rotos, todas las carpetas indexadas o excluidas con motivo |
| GOV-C5 | ✅ | 89b0b484 — operations/master-index +7 archivos + regla same-PR |
| GOV-C6 | ✅ |  7c330c9 — sweep 44 env vars (+5 añadidas), rate_limit_rpm 600, flush_threshold None, spot-check 14 defaults |
| GOV-C7 | ✅ | 89b0b484 — contador ~45 activas + regla sync en Backlog header; ROADMAP banner alineado |

plan-adjust [2026-08-22]: GOV-C2 parcialmente pre-ejecutado por sesión SDKB (48016b89 limpió MEM-36/43/44/45 como pagadas) — se registró P29/P30/P31 compacto en vez de duplicar.
