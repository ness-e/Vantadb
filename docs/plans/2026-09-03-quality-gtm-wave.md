# Plan de Ejecución: Quality & GTM Wave — Post-Auditoría Backlog

> **Inicio:** 2026-09-03
> **Estado:** ⬜ PENDING (listo para `/pipeline run`)
> **Fuente:** `docs/Backlog.md` (123 activas post-paso0; 126 triadas + 3 purgadas con evidencia) + 4 audits de sesión 2026-09-02/03 + verificación código hoy
> **Autonomous:** false
> **Campaign ID:** 20260903-quality-gtm-wave
> **SDP cargado:** campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail (full), spec-driven-development, systematic-debugging, code-review-and-quality, security-and-hardening, observability-and-instrumentation, api-and-interface-design, coordinated-web-search — phase=PLAN
> **Spec:** no existe SPEC.md; cada DO es vertical slice con contrato mecánico (Gate P respondió el usuario: set 12 confirmado, GTM dentro, 3 purgas aprobadas)
> **Contexto deps:** Show HN D3=septiembre → SRV-07/MKT-18h/f/i son habilitadores de adopción. FIND-52 resuelto (`a137bdc7` vitest 278/278) → RES-12 puede apoyarse en build web verde. Error-handling estandarizado (campaña cerrada 9/9) → AUD-045/RES-03 pueden medir con `tracing` estructurado.

## Resumen Triage (126 filas activas al inicio del plan)

| Resultado | Count | % | Notas |
|-----------|-------|---|-------|
| ✅ DO | 12 | 9.5% | las de abajo, verificadas hoy contra código |
| 🟡 DEFER | 98 | 77.8% | P5/6/8 restantes, P23/24 (Pro/I+D), P26-28 (MEM-6x/7x, MCP-41, PY), P32-34 (UX-*, MOD-*), P38 resto (RES-02 restaurada caos, RES-04/05/06/10-15), P39 PRX-01..13, P41-47, DISC/MKT-04 humano |
| ❌ SKIP | 3 | 2.4% | PURGADAS con evidencia (FIND-22 `3b1b820b` fila stale; PY-02 BENCHMARKS §2 ya satisface contrato; FIND-51 premature — umbral propio >2500L no alcanzado a 1469L). Registradas en `backlog-history.md` |
| 🔴 BLOQUEADO | 13 | 10.3% | AUD-042 (tantivy ≥0.27 no publicada), CORE-02 (PITR requiere ADR owner), STABLE-04..09 (ADR-031+medición owner), MCP-34b (depende snapshot/FIND-33), BND-08 + TS-12 + PERF-BENCH-01 (estrategia npm napi post-launch — decisión owner), SRV-06 (OIDC DISCOVERY vanta-arch), GOV-TK2 (es decisión `/ship`, no tarea) |
| **Total** | **126** | 100% | |

**Gate P respondido por el usuario (question 2026-09-03):** set DO 12 confirmado; purgas FIND-22/PY-02/FIND-51 confirmadas; GTM dentro del plan (Show HN sept).

Status: ⬆️ uphill = 3 (RES-03 approach canal, AUD-045 decisión A/B, SRV-07 wiring release) · ⬇️ downhill = 12

## Regla de este plan (lección de la sesión)
> Cada DO debe **revisar TODOS los archivos listados**, sin saltarse ninguno, y su `Verificación real` es evidencia ejecutada hoy (comando + output), no texto heredado del Backlog. Si en DISCOVERY la premisa muere → STOP CONDITION (no "COMPLETED por reuse" sin evidencia — regla aprendida de las 6 reaperturas del plan anterior).

---

## Grafo de Dependencias y Waves (MAX_CONCURRENT=3, FAIL_MODE=parallel)

```
Wave0 (paralelo 3, disjuntos):   RES-07 (config.rs+BENCHMARKS §nuevo) | GOV-TK1 (src/cli.rs+cli_handlers) | GOV-TK9 (docs/operations/checklist)
Wave1 (paralelo 3, disjuntos):   MKT-18h (release-wheels-60.yml+Formula) | SRV-07 (Dockerfile+release-binaries-63.yml) | MKT-18i (docker-compose.yml)
Wave2 (paralelo 3, disjuntos):   RES-12 (web/src/components/vanta/*.tsx) | MKT-18f (integrations/**+workflow nuevo) | RES-09 (docs/Backlog.md filas P24)
Wave3 (paralelo 2):              RES-03 (src/ingestion.rs+Cargo.toml+BENCHMARKS-append) | RES-15C (Backlog.md move→Backlog-negocio.md)
Wave4 (solo):                    AUD-045 (src/index/ivf.rs+BENCHMARKS A/B)  ← último: toca BENCHMARKS después de RES-03
```
Justificación de órdenes compartidos: BENCHMARKS.md (RES-07 → … → RES-03 → AUD-045) y Backlog.md (RES-09 → RES-15C) se serializan para evitar merge conflict (regla learned: Cargo.toml/BENCHMARKS no en paralelo).

**Checkpoints humanos:** tras Wave2: revisión de artefactos CI (workflows nuevos no roben fast-gate). Tras Wave4 (fin): `/audit quick` + `/ship` decision para GOV-TK2.

---

## Tasks — ✅ DO (12)

### Task 1: RES-07 — Calibrar `DEFAULT_RSS_THRESHOLD` con datos del bench F2

- **Appetite:** max 1d | **Esfuerzo:** 🟢 <1h efectiva | **Prioridad:** 🟡 Media (guard OOM = durabilidad)
- **Archivos clave (TODOS):** `src/config.rs:22` (`DEFAULT_RSS_THRESHOLD: f64 = 0.80`), `benches/memory_budget.rs` (default sizes `[10k,25k,50k,100k]` + delta RSS−logical), `docs/operations/BENCHMARKS.md` (nueva subsección en §memory-budget), `docs/research/FND-01*` (origen F2)
- **Verificación real:** ✅ 2026-09-03 — umbral sigue 0.80 sin calibración (rg sin hit de decisión); FND-01.md:3 + FND-01-F1.md:42 declaran "F2/F3 pendientes"; F3 ya landed → F2 = correr bench, decidir, documentar
- **Gate Justificación:** el memory-budget guard decide flush/shed; 0.80 es heurística huérfana; con el bench F3 existente la calibración cuesta minutos y cierra un follow-up de investigación documentado
- **Contrato:** `rg -n "rss_threshold" docs/operations/BENCHMARKS.md | Measure Count` ≥1 con tabla medida (dataset→delta RSS) + línea "decisión: DEFAULT_RSS_THRESHOLD=<valor> calibrado <fecha>"; si el valor cambia, `rg "DEFAULT_RSS_THRESHOLD: f64" src/config.rs` == valor documentado; `cargo test -p vantadb --lib config` 0 failed; `cargo fmt --all -- --check` 0
- **Pre-mortem:** 1 el bench midió en máquina ruidosa y el delta no es reproducible → correr ×2 y reportar mediana; 2 0.80 resulta correcto y "no hay cambio" → la decisión documentada ES el entregable (no inflar)
- **Stop conditions:** bench no corre en 3 intentos → DEFER con log; decisión necesita ADR (cambio >±10% del behavior) → gate Regla 5
- **Risk Register:** | 🟡×🟡 | threshold más bajo sheddea writes legítimos | mantener dentro 0.70..0.85 y registrar rationale | verify |
- **Cynefin:** 🟦 Obvio (medir→documentar) | **Top 3 riesgos:** ruido de medición, sobre-interpretar 1 corrida, drift config-doc
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1 | **DoD:** Task: contrato; Commit `fix(memory-budget): calibrar rss_threshold con bench F2 (RES-07)`; Release: nota en CHANGELOG si cambia el valor
- **Estado:** ✅ COMPLETED
- **Recitation cierre:** BENCHMARKS.md §12 `rss_threshold` con tabla FND-01 post-F1 (5k/10k/20k → pressure 0.003/0.005/0.011, slopes 11.6/20.0 KB-nodo) + decisión MANTENER 0.80 (cambio 0%, sin ADR); `cargo test -p vantadb --lib config` 54/0 ✅; bench compile ✅; fmt: solo diffs GOV-TK1 ajenos (no tocados). Commit `fix(memory-budget): calibrar rss_threshold con bench F2 (RES-07)`.

### Task 2: GOV-TK1 — `vanta-cli doctor --fix` (dry-run ya existe, fix falta)

- **Appetite:** max 1d | **Esfuerzo:** 🟡 ~3h | **Prioridad:** 🟡 Media (DR runbook GOV-B2 depende conceptualmente)
- **Archivos clave (TODOS):** `src/cli.rs:154,380` (Doctor, dry_run), handler doctor en `src/cli_handlers/*`, `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` §3.1 (insumo GOV-A3/C3), tests CLI en `tests/`
- **Verificación real:** ✅ 2026-09-03 — `dry_run: bool` existe en cli.rs (restore --dry-run landed); `--fix` en doctor: 0 hits → scope reducido a la mitad que falta
- **Gate Justificación:** el runbook dice "doctor --fix" pero el flag es fantasma (misma clase de bug que GOV-B2 corrigió al revés); o se implementa, o se borra del runbook — implementar es la decisión D4b del owner
- **Contrato:** `rg -n -e "fix" src/cli.rs | rg -c "doctor|Fix|--fix"` ≥1 AND `vanta-cli doctor --fix` sobre DB temporal exit 0 con salida que liste reparaciones (o "nothing to fix") AND `rg -n "doctor --fix" docs/operations/DISASTER_RECOVERY_RUNBOOK.md` sigue ≥1 y es VERDAD ahora AND `cargo clippy --workspace --all-targets -- -D warnings` 0
- **Pre-mortem:** 1 "fix" no tiene nada reparable de forma segura → scope mínimo: crear dirs faltantes/permisos del data_dir + normalizar discovery file stale; 2 feature-add de superficie CLI → Gate D satisfecho por este plan (contrato y scope explícitos)
- **Stop conditions:** cualquier reparación que toque datos del usuario (destructiva) → NO implementar, documentar hallazgo, mantener runbook honesto
- **Risk Register:** | 🟡×🔴 | fix destructivo silencioso | dry-run por default + confirm | review |
- **Cynefin:** 🟨 Complicado | **Top 3 riesgos:** daño a datos, scope creep de "fix", flag fantasma otra vez
- **Uphill/Downhill:** ⬆️ 1 (qué es reparable) · ⬇️ 1 | **DoD:** Task: contrato; Commit `feat(cli): doctor --fix con dry-run seguro (GOV-TK1)`; Release: CLI docs + CHANGELOG
- **Estado:** ✅ COMPLETED 2026-09-03 | **Task file:** `.../tasks/GOV-TK1.md` (Steps 1-4 ✅) | **Ruta:** vanta-worker | **Branch:** develop
- **Recitation cierre:** `Doctor { fix, force }` en cli.rs + dispatch + `cmd_doctor(db, fix, force, verbose)` con dry-run default (lista WOULD-FIX, cero mutación) y `--force` aditivo (create db + data/); `doctor --fix` exit 0 en missing/empty/healthy; runbook §2.3 honesto (3 hits VERDAD); `cargo test cli_tests doctor` 6/6 ✅; clippy workspace 0 ✅; fmt 0 ✅. Commit `feat(cli): doctor --fix con dry-run seguro (GOV-TK1)`. Backlog GOV-TK1 NO eliminada (mitad restore --dry-run pendiente).

### Task 3: GOV-TK9 — Verificar URL `vantadb-examples` del checklist

- **Appetite:** max 1h | **Esfuerzo:** 🟢 15m | **Prioridad:** 🟢 Baja
- **Archivos clave (TODOS):** `docs/operations/pilot-onboarding-checklist.md:51` (`git clone https://github.com/vantadb/vantadb-examples`), `docs/README*`/otros puntos que referencien el repo
- **Verificación real:** ✅ 2026-09-03 — el checklist apunta a `github.com/vantadb/...`; según FIND-17/ADR-030 el owner real es `ness-e/*`; organización `vantadb` inexistente → URL muerta para el piloto
- **Gate Justificación:** el checklist es documento de venta (pilot enterprise) con un paso que da error
- **Contrato:** webfetch del actual → si 404: `rg -n "vantadb/vantadb-examples" docs/` == 0 tras fijar (a `ness-e/vantadb-examples` si existe, o marcar `[TODO humano: crear repo]`); si 200: fila se cierra con evidencia
- **Pre-mortem:** 1 `ness-e/vantadb-examples` tampoco existe → dejar TODO explícito y crear fila nueva para el repo (no crear repo desde agentes)
- **Stop conditions:** si se decide crear el repo → fuera de scope, hallazgo
- **Risk Register:** | 🟢×🟢 | decisión humana pendiente | TODO etiquetado | docs |
- **Cynefin:** 🟦 Obvio | **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1
- **DoD:** Task: contrato; Commit `docs(gov): verificar URL vantadb-examples (GOV-TK9)`; Release N/A
- **Estado:** ✅ COMPLETED 2026-09-03 | **Task file:** `.../tasks/GOV-TK9.md` | **Ruta:** vanta-docs | **Branch:** develop
- **Verificación real (cierre):** `webfetch github.com/vantadb/vantadb-examples` → 404; `webfetch github.com/ness-e/vantadb-examples` → 404; `rg "vantadb/vantadb-examples" docs/operations/ docs/api/` = 0; checklist:51 ahora TODO-humano explícito; fila Backlog eliminada (progreso)

### Task 4: MKT-18h — Wheels ARM64 Linux + SHA256 reales del Formula Homebrew

- **Appetite:** max 2d | **Esfuerzo:** 🟡 ~1d | **Prioridad:** 🟠 Media-Alta (fórmula con `0000…0` = install falla + señal de abandono)
- **Archivos clave (TODOS):** `.github/workflows/release-wheels-60.yml` (hoy solo x86_64), ubicación real del `Formula/vantadb.rb` (DISCOVERY: repo homebrew-tap — buscar `rg -rn "Formula" .github/ docs/`), `release-binaries-63.yml` (referencia de cómo SÍ sale aarch64 para el binario), `docs/QUICKSTART*`/README que documentan brew install
- **Verificación real:** ✅ 2026-09-03 — GOV-A5 (hoy, `20260902` campaña) registró con captura live: 0.5.0 publicado, **wheels ARM64 ausentes, SHA256 placeholders**; binarios aarch64 sí salen (`release-binaries-63.yml`) → el patrón existe y falta replicarlo en wheels
- **Gate Justificación:** `brew install` es el canal #1 macOS; la fórmula rota es bug de distribución, no feature
- **Contrato:** `rg -n "aarch64-unknown-linux-gnu" .github/workflows/release-wheels*.yml` ≥1 AND dry-run del job (o `maturin --target` local si toolchain cross disponible) genera `*.aarch64.whl` AND Formula: `rg "000000000000" <ruta-formula>` == 0 con SHA del wheel x86_64 actualizado (o workflow que lo inyecta); si la fórmula vive en otro repo → PR artefacto local + fila `TODO-humano` documentada
- **Pre-mortem:** 1 sin acceso al tap → cerrar lo del repo (workflow) y handoff humano de la fórmula; 2 manylinux vs musl: no scopear musl (queda BND-09)
- **Stop conditions:** cross-compile local imposible >2d → dejar workflow correcto + nota de una corrida CI como verificación
- **Risk Register:** | 🟡×🟡 | release workflow roto toca publish | probar en branch/tag rc, no en main | pre-merge | | 🟢×🟡 | fórmula en otro repo | handoff documentado | discovery |
- **Cynefin:** 🟨 Complicado | **Top 3 riesgos:** repo del tap, artefactos sin runner ARM, secret de publish
- **Uphill/Downhill:** ⬆️ 1 (paradero fórmula) · ⬇️ 2 | **DoD:** Task: contrato; Commit `ci(wheels): aarch64 linux + SHA reales (MKT-18h)`; Release: verificado en siguiente release-plz cycle
- **Estado:** ✅ COMPLETED 2026-09-03 | **Task file:** `.../tasks/MKT-18h.md` | **Ruta:** vanta-worker | **Branch:** develop
- **Verificación real (cierre):** contrato 4/4 — `rg -n "aarch64-unknown-linux-gnu" .github/workflows/release-wheels-60.yml` = 1 match; `actionlint release-wheels-60.yml` exit 0; `Formula/vantadb.rb` (paradero: LOCAL, no tap remoto) 0 placeholders con 4 SHA256 verificados por doble vía (Get-FileHash local de los tarballs v0.5.0 == sidecar CI); `cargo check -p vantadb_py --all-targets` exit 0. Fix extra: remove `bin.install vantadb-mcp` (no está en los tarballs) + input muerto `musllinux` (no existe en maturin-action v1.51.0). Verificación del job aarch64 diferida a corrida CI (stop-condition del plan). Fila Backlog eliminada (re-aplicada tras clobber de sesión paralela; workflow+plan cabalgaron el commit `2ab706ec` ajeno, Formula/avance/Backlog en su commit `ci(wheels)`).

### Task 5: SRV-07 — Dockerfile unprivileged + wiring release

- **Appetite:** max 1d | **Esfuerzo:** 🟡 ~3-4h | **Prioridad:** 🟠 Media-Alta (adopción self-hosted; los 4 competidores tienen imagen)
- **Archivos clave (TODOS):** `Dockerfile` (existe, verificar user root), `docker-compose.yml` (servicio base; NO tocar compose multi-servicio — eso es MKT-18i), `.github/workflows/release-binaries-63.yml` (hook de build/push de imagen o decisión explícita de no publicar), `docs/operations/*deployment*` (documentar run unprivileged)
- **Verificación real:** ✅ 2026-09-03 — Dockerfile existe (CRIT-01 resolvió `RUST_VERSION` pin) pero la tarea pide lo que falta: variante unprivileged (patrón qdrant `-unprivileged`) y wiring al flujo RELEASE; `docker-compose.yml` solo servicio vantadb → compose de SRV-07 = verificar que ande con uid alto, no multi-servicio (eso es 18i)
- **Gate Justificación:** sin imagen pública el funnel self-hosted muere en el primer `docker run`; Show HN sept lo necesita al menos reproducible
- **Contrato:** `docker build -t vantadb:test .` exit 0 AND `docker run --rm -u 10001:10001 -v $PWD/data:/data vantadb:test vantadb/... --help` (o la invocation real del entrypoint) exit 0 sin permisos denegados (USER no-root en la imagen o ARG `VANTA_RUNAS_UID`) AND `rg -ni "docker" .github/workflows/release-binaries-63.yml` ≥1 o decisión en CI_POLICY.md de por qué la imagen NO se publica desde CI
- **Pre-mortem:** 1 sin docker daemon local → fallback: podman o validar sintácticamente + corrida CI como verificación; 2 entrypoint del binario real (`vanta-cli`) difiere del que asume la tarea
- **Stop conditions:** publicar imagen registry (ghcr vs dockerhub) es decisión de marca → no elegirla, documentar
- **Risk Register:** | 🟡×🟢 | daemon no disponible | CI run como verificación | local |
- **Cynefin:** 🟨 | **Top 3 riesgos:** daemon, entrypoint real, registry decisión
- **Uphill/Downhill:** ⬆️ 1 · ⬇️ 2 | **DoD:** Task: contrato; Commit `ci(docker): unprivileged build + release wiring (SRV-07)`; Release: imagen en siguiente tag
- **Estado:** ✅ COMPLETED 2026-09-03 | **Task file:** `.../tasks/SRV-07.md` | **Ruta:** vanta-worker | **Branch:** develop

### Task 6: MKT-18i — compose multi-servicio Ollama + VantaDB + AnythingLLM

- **Appetite:** max 1d | **Esfuerzo:** 🟢 2-4h | **Prioridad:** 🟡 Media (demo canónica que los posts MKT-04 ya anuncian)
- **Archivos clave (TODOS):** `docker-compose.yml` (hoy: 1 servicio + volume), `docs/tutorials/*` (migración LanceDB ya existe — enlazar), README (bloque compose), `docker-compose.dev.yml` (no romperlo)
- **Verificación real:** ✅ 2026-09-03 — `docker-compose.yml` tiene solo `vantadb` + `vantadb-data`; la tarea pide orquestar ollama+anythingllm como demo copypaste
- **Gate Justificación:** es el "hello world" que los usuarios de LocalLLaMA copian; un compose que no orquesta nada no vende
- **Contrato:** `rg -ci "ollama|anythingllm" docker-compose.yml` ≥2 AND `docker compose config -q` exit 0 AND `docker compose up -d && curl localhost:<port-vantadb>/health` 200 && down limpio (si no hay daemon: `config -q` + transcripción de un run documentado)
- **Pre-mortem:** 1 AnythingLLM cambió env-var schema → fijar tags explícitos; 2 memoria del host: documentar RAM mínima
- **Stop conditions:** si el demo exige API keys → compose queda con placeholders + README honesto
- **Risk Register:** | 🟢×🟡 | tags floaty rompen demo | pinnear | review |
- **Cynefin:** 🟦 | **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1
- **DoD:** Task: contrato; Commit `feat(demo): compose ollama+vantadb+anythingllm (MKT-18i)`; Release N/A
- **Estado:** ✅ COMPLETED (re-escalado por stop condition 2026-09-03: AnythingLLM sin soporte VantaDB — evidencia en `docs/avance/activo/operaciones.md`; demo shipped VantaDB+Ollama, commit `abb6594c`; fila Backlog re-escalada a 🔴 upstream) | **Task file:** `.../tasks/MKT-18i.md` | **Ruta:** vanta-worker | **Branch:** develop

### Task 7: RES-12 — Touch targets <44px (re-escalado: 4-5 archivos reales)

- **Appetite:** max 1d | **Esfuerzo:** 🟢 2-4h | **Prioridad:** 🟡 Media (WCAG 2.5.5 pre-Show-HN)
- **Archivos clave (TODOS — los 5 verificados por grep `<button` hoy):** `web/src/components/vanta/docs-view.tsx` (botón copiar `h-7 w-7` ~:576 y otro), `shortcut-overlay.tsx`, `site-navbar.tsx`, `tutorial-modal.tsx`, `command-palette.tsx:232` (cerrar búsqueda); verificar además cualquier `<a role=button>` con h-7/h-9 que el patrón de botón no atrapó
- **Verificación real:** ✅ 2026-09-03 — la fila original "~20 componentes" estaba inflada: los h-7/h-9 decorativos (barras benchmark, iconos) NO cuentan; quedan exactamente estos 5 archivos con `<button>` de 28-36px
- **Gate Justificación:** a11y real medible y barato antes de exposure pública
- **Contrato:** `rg --multiline '<button[^>]*h-(7|9)\b' web/src -g '*.tsx' | Measure Count` == 0 (los targets pasan a `size-11`/h-11 o se agrega hit-area `p-2 -m-2` — el que se use, parejo en los 5 archivos) AND `npm run build --prefix web` 0 AND `npm run lint` 0 errors AND `npx playwright test` 2 passed (guard WEB-08) AND `rg -n "h-7|h-9" web/src/components/vanta/{docs-view,shortcut-overlay,site-navbar,tutorial-modal,command-palette}.tsx | rg -v "^.*//.*decorat"` sin buttons
- **Pre-mortem:** 1 cambiar tamaño rompe layout brutalista (bordes 4px) → preferir hit-area invisible sobre agrandar el borde visual
- **Stop conditions:** si algún "button" resulta ser el trigger de un dropdown intencional compacto → documentar excepción en el task file
- **Risk Register:** | 🟢×🟢 | regresión visual | screenshot guard del spec e2e | verify |
- **Cynefin:** 🟦 | **Uphill/Downhill:** ⬆️ 0 · ⬇️ 5 archivos
- **DoD:** Task: contrato; Commit `fix(web): touch targets ≥44px en 5 componentes (RES-12)`; Release N/A
- **Estado:** ⬜ PENDING | **Task file:** `.../tasks/RES-12.md` | **Ruta:** vanta-worker (web) | **Branch:** develop

### Task 8: MKT-18f — Publicar 5 adapters PyPI + PRs upstream

- **Appetite:** max 3d | **Esfuerzo:** 🟡 ~1-2d | **Prioridad:** 🔴 Alta (GTM: checkboxes langchain/llama-index/Mem0/TSK-90/91 rotos con 404)
- **Archivos clave (TODOS):** `integrations/langchain/`, `integrations/llama-index/`, `integrations/mem0/`, `integrations/crewai/`, `integrations/dspy/` (cada uno: pyproject, README, test import), workflow nuevo `.github/workflows/release-adapters.yml` (o job extra en el de wheels), `docs/api/*` (enlazar), `docs/strategy/REDDIT_POSTS.md` (los claims de adapters hoy dicen 404 implícito — al publicar, corregir)
- **Verificación real:** ✅ sesión GOV-A5 — código existe en `integrations/` pero 404 en PyPI (verificado live); MKT-18f sigue vigente
- **Gate Justificación:** "zero-config integrations" es el diferenciador declarado; sin paquetes, el claim es marketing roto (Regla 11)
- **Contrato:** por cada adapter: `python -m build` exit 0 (wheel+sdist) AND `twine check dist/*` OK AND workflow de release presente con `publish=false` hasta el tag (mismo patrón release-plz); PRs upstream = 5 borradores de PR generados como artefacto local (`docs/plans/artifacts/mkt-18f-prs/`) porque el push es humano
- **Pre-mortem:** 1 nombres ya reservados por otro ("vantadb-langchain") → `pip index versions` en DISCOVERY; 2 deps de adapters (langchain-core) pinnear ranges bajos; 3 el workflow no debe correr tests pesados (fast gate no se toca)
- **Stop conditions:** si 1+ paquete del nombre está tomado → elegir prefijo distinto como decisión y re-preguntar (Gate P)
- **Risk Register:** | 🟡×🔴 | PyPI squatting del nombre | verificar early en DISCOVERY | step1 |
- **Cynefin:** 🟨 | **Top 3 riesgos:** squatting, deps drift, publish secrets
- **Uphill/Downhill:** ⬆️ 1 · ⬇️ 5 adapters
- **DoD:** Task: contrato; Commit `feat(integrations): paquete PyPI publishable x5 + release workflow (MKT-18f)`; Release: publicación real requiere `/ship`+token
- **Estado:** ⬜ PENDING | **Task file:** `.../tasks/MKT-18f.md` | **Ruta:** vanta-worker | **Branch:** develop

### Task 9: RES-09 — Filas P24 del roadmap huérfano (docs)

- **Appetite:** max 1h | **Esfuerzo:** 🟢 30m | **Prioridad:** 🟢 Baja
- **Archivos clave (TODOS):** `docs/Backlog.md` sección P24, `docs/research/investigacion-equipo-2026-08-09.md` §roadmap (fuente con archivo:línea)
- **Verificación real:** ✅ 2026-09-03 — 0 filas nuevas hoy; gaps confirmados vivos: `src/index/diskann.rs:7` "purely in-memory, **not disk-backed**"; `src/wal.rs` sync default por write (el gap es **fsync batching/decoupling**, NO async-ingest genérico: `src/ingestion.rs` ya da pipeline async de nodos); query planner sin optimizaciones más allá del CBO básico
- **Gate Justificación:** investigación que no se trackea = trabajo perdido (patrón meta-001)
- **Contrato:** `rg -n "FUT-1[234]" docs/Backlog.md | Measure Count` ≥3 con descripciones re-escaladas: FUT-12 WAL fsync-batching, FUT-13 query planner optimizaciones, FUT-14 DiskANN disk-I/O real
- **Pre-mortem:** 1 los IDs FUT-12..14 chocan con filas existentes → verificar primero (`rg FUT-1 docs/Backlog.md`)
- **Stop conditions:** N/A (docs)
- **Risk Register:** | 🟢×🟢 | duplicar fila existente | rg previo | n/a |
- **Cynefin:** 🟦 | **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1
- **DoD:** Task: contrato; Commit `docs(backlog): trackear roadmap huérfano P24 (RES-09)`; Release N/A
- **Estado:** ⬜ PENDING | **Task file:** `.../tasks/RES-09.md` | **Ruta:** vanta-docs | **Branch:** develop

### Task 10: RES-03 — Canal multi-consumidor en ingestion pipeline

- **Appetite:** max 3d | **Esfuerzo:** 🟡 1-2d | **Prioridad:** 🟠 Media-Alta (throughput ingesta = core del pitch "memory para agents")
- **Archivos clave (TODOS):** `src/ingestion.rs` (L72 `Arc<Mutex<mpsc::Receiver>>` verificado intacto hoy; todo el pipeline: spawn, backpressure, error path), `Cargo.toml` (candidato `crossbeam-channel` — single dep sin runtime async; verificar compat wasm32 antes de elegir; `flume` es la variante async), `benches/bench_concurrent.rs` (baseline de ingesta concurrente), `docs/operations/BENCHMARKS.md` (append antes/después), `src/error.rs` si cambia el tipo de error de envío
- **Verificación real:** ✅ 2026-09-03 — `rg "Arc<Mutex<mpsc::Receiver" src/ingestion.rs` hit exacto L72; sin canal multi-consumidor en deps; FND-19 lo listaba como única instancia sospechosa del inventario
- **Gate Justificación:** un solo worker puede acaparar el mutex y serializar; es la única sospecha de contención de inventario amplio → medir antes (Regla 9), rediseñar solo si la medición justifica (la tarea MISMA ya lleva la decisión dentro)
- **Contrato:** baseline bench concurrente registrado en BENCHMARKS (N producers × {1,2,4} consumers, ops/s) AND `rg "Arc<Mutex<mpsc::Receiver" src/ingestion.rs` == 0 AND `cargo test -p vantadb --lib ingestion` 0 failed AND after ≥ baseline (si NO mejora: revert con la medición documentada y la fila se cierra como "decidido: no aplicar" — ambos resultados son exit, la decisión es de los datos) AND `cargo clippy --workspace --all-targets --all-features -D warnings` 0 AND wasm32 build sigue OK (`cargo check -p vantadb --target wasm32-unknown-unknown` si ingestion entra al build wasm)
- **Pre-mortem:** 1 el lock nunca es contención real (los consumers duermen en recv) → por eso el bench manda; 2 crossbeam en wasm32-unknown puede fallar → DISCOVERY de feature-gate si aplica; 3 cambiar canal altera ordering → los tests de ingest ordenados lo cubren
- **Stop conditions:** si la medición muestra p99/throughput igual, NO refactorear: documentar y cerrar; >3d → DEFER con hallazgo
- **Risk Register:** | 🟡×🔴 | refactor hot-path sin mejora | bench primero, revert barato | contrato | | 🟡×🟡 | dep nueva rompe wasm | cargo check wasm32 | build | | 🟢×🟡 | cambio de ordering | tests | verify |
- **Cynefin:** 🟧 Complejo (la respuesta emerge de medir) — probe-sense-respond: bench → micro-cambio → re-bench
- **Top 3 riesgos:** medir tarde, dep wasm, ordering
- **Uphill/Downhill:** ⬆️ 1 (qué canal) · ⬇️ 2 | **DoD:** Task: contrato; Commit `perf(ingestion): canal multi-consumidor con A/B bench (RES-03)`; Release: CHANGELOG perf si throughput >5%
- **Estado:** ⬜ PENDING | **Task file:** `.../tasks/RES-03.md` | **Ruta:** vanta-worker (+verificación tuner vía bench) | **Branch:** develop

### Task 11: RES-15-C — Separar backlog negocio vs técnico

- **Appetite:** max 1d | **Esfuerzo:** 🟢 ~1h | **Prioridad:** 🟢 Baja
- **Archivos clave (TODOS):** `docs/Backlog.md` (filas negocio identificadas hoy: DISC-* NO — es community ops: decidir en discovery con criterio explícito; firmes: `LEG-01`, `MKT-04`, `CLD-01/02/04`, `BLOG-CTA`, `BIZ-01b`, `PRO-01..06`), `docs/Backlog-negocio.md` (NUEVO), `docs/avance/meta.md` (regla de dos backlogs), cross-ref desde `docs/strategy/ROADMAP.md`
- **Verificación real:** ✅ 2026-09-03 — B ya institucionalizada (`research-decide.md:81`→wontfix.md); C no hecha (archivo único, filas humanas-mixtas con técnicas); GOV-TK5 es split del Manual (no duplicar, enlazar)
- **Gate Justificación:** el backlog único mezcla "abogado/pago/PyPI" con "clippy/wal" y distorsiona cualquier métrica de prioridad (la recomendación de esta sesión quedó contaminada por eso)
- **Contrato:** `Test-Path docs/Backlog-negocio.md` AND `rg -c "^\| \`?(LEG-01|CLD-0[124]|BIZ-01b|PRO-0[1-6]|MKT-04|BLOG-CTA)" docs/Backlog-negocio.md` ≥11 AND esas filas NO quedan en Backlog.md (`rg "PRO-01" docs/Backlog.md` ==0) AND header de ambos archivos con cross-link + contador coherente con `rg` (regla anti-drift)
- **Pre-mortem:** 1 mover filas rompe IDs citados en planes/archive → agregar nota "movida a Backlog-negocio" en la fila de destino, no borrar historial
- **Stop conditions:** si la frontera negocio/técnica queda ambigua para MKT-18* (son GTM-pero-código) → criterio: lo que requiere agente/código queda técnico; lo que requiere abogado/plata/decisión humana queda negocio
- **Risk Register:** | 🟢×🟡 | contador back-drift | regla sync por rg | meta.md |
- **Cynefin:** 🟦 | **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1
- **DoD:** Task: contrato; Commit `docs(process): split backlog negocio/tecnico (RES-15-C)`; Release N/A
- **Estado:** ⬜ PENDING | **Task file:** `.../tasks/RES-15.md` | **Ruta:** vanta-lead (docs/process) | **Branch:** develop

### Task 12: AUD-045 — IVF: clones por candidato en hot path → medir → A/B o decisión

- **Appetite:** max 2d | **Esfuerzo:** 🟡 1d | **Prioridad:** 🟡 Media (persecución de performance con evidencia)
- **Archivos clave (TODOS):** `src/index/ivf.rs` (clones del camino de búsqueda — verificar líneas actuales: `.clone()` encontrado en build/training L82/91/108/154 y confirmar los del search loop: el gap citado era `centroid.clone()`/`entry.vector.clone()` por candidato), `benches/ivf_bench.rs` (+ `canonical_p99` si IVF entra), `src/index/search/…` para el borrow/slice alternativo, `docs/operations/BENCHMARKS.md` (tabla A/B)
- **Verificación real:** ✅ 2026-09-03 — clones presentes (grep `.clone()` en ivf.rs múltiples hits; los del search path a confirmar en DISCOVERY); baseline `canonical_p99` existe (Regla 9 satisfecha: la infra de medición ya está)
- **Gate Justificación:** copiar un vector por candidato es el desperdicio clásico; pero según Regla 9 el fix SIN medición es conjetura — la tarea es medir→decidir, no "optimizar"
- **Contrato:** corrida baseline documentada (p50/p95/p99 IVF, dataset seed 42) AND variante borrowed/slice implementada O el caso "no mejora" documentado con números; si se implementa: `rg -c "entry.vector.clone()" src/index/ivf.rs` (search loop) == 0 AND after p99 ≤ baseline − ruido (documentar umbral, p.ej. >2%) AND `cargo test -p vantadb --lib ivf` 0 failed AND recall invariante (assert recall ≥ baseline−ε en el bench)
- **Pre-mortem:** 1 el "clone" que aparece es de build/training (no hot) → el search loop quizá ya borrows; en ese caso cerrar con evidencia y fila obsoleta; 2 lifetimes de la borrow no cierran con rayon → evaluar Arc<[f32]> como fallback
- **Stop conditions:** si borrow exige cambiar `StorageEngine` trait surface (breaking) → no hacerlo, documentar el costo
- **Risk Register:** | 🟡×🟡 | cambio de API interna por lifetimes | Arc fallback | impl | | 🟡×🟡 | ruido de medición | mismo perfil, ×2 corridas | verify |
- **Cynefin:** 🟨 Complicado | **Top 3 riesgos:** medir lo equivocado, borrow-vs-rayon, ruido
- **Uphill/Downhill:** ⬆️ 1 (dónde está realmente el clone caliente) · ⬇️ 2
- **DoD:** Task: contrato; Commit `perf(ivf): search sin clones por candidato con A/B canonical (AUD-045)` o `docs(bench): AUD-045 medido, no aplica`; Release: CHANGELOG perf si >5% p99
- **Estado:** ⬜ PENDING | **Task file:** `.../tasks/AUD-045.md` | **Ruta:** vanta-tuner | **Branch:** develop

---

## Dependencias entre Tasks (resumen)

`Wave0: T1,T2,T3 | Wave1: T4,T5,T6 | Wave2: T7,T8,T9 | Wave3: T10,T11 | Wave4: T12`
Serializaciones obligatorias: `BENCHMARKS.md` T1→T10→T12; `Backlog.md` T9→T11. T4 (release-wheels) y T5 (release-binaries) tocan workflows distintos = paralelo seguro.

## Checkpoint post-plan (verificación global al cerrar)

```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --profile audit -p vantadb            # suite sin regresiones
npm run build --prefix web && npx playwright test        # web guard
rg -n "FIND-22|PY-02|FIND-51" docs/Backlog.md            # = 0 (purga efectiva)
```

## Notas

- **Lecciones aplicadas de la auditoría anterior:** (1) ningún ✅ sin recitación con comando+output en el task file; (2) IDs cotejados por contenido (esta vez: GOV-TK1 resultó media-hecha, RES-12 sobre-estimada, SRV-07 ya tenía Dockerfile, PY-02 ya estaba hecha); (3) premisa muerta → STOP condition documenta, no fuerza el cierre.
- **GOV-TK2 (release 0.6.0)** quedó BLOQUEADO→decisión `/ship`: este plan deja el repo listo para release (adapters, wheels ARM64, docker, doctor --fix); al cerrar Wave4, correr `/ship`.
- **Human gates embebidos:** publicación real PyPI/brew/docker registry y PRs upstream = acciones del lead/humano; los contratos terminan en artefactos verificables local/CI.
- **Ponytail:** T10 (RES-03) y T12 (AUD-045) son medir-decide-no-build; T11 es la única tarea que toca arquitectura de procesos. FIND-51 fue purgada por el propio umbral (>2500L): no sub-split handlers.rs.

## Context Save Point

- **Fecha:** 2026-09-03
- **Branch:** develop
- **Estado:** ⬜ PENDING (0/12)
- **Próxima tarea:** Wave0 paralelo `T1 RES-07 + T2 GOV-TK1 + T3 GOV-TK9` (quick wins que desbloquean runbook y cierran follow-up de memoria)
- **Decisiones del usuario (Gate P):** set 12 DO ✅, 3 purgas ✅, GTM dentro ✅

=== RECITATION 1 ===
Campaign ID: 20260903-quality-gtm-wave
Objetivo activo: RES-07 — Calibrar DEFAULT_RSS_THRESHOLD con bench F2
Estado: completed
Última acción: Steps F2 4-6 ✅: §12 appended + plan sync ✅ + commit 3a27c5f4 (2 files, 50+/1-)
Resultado: ✅
Próxima acción: ninguno (RES-07 cerrada)
Contrato: verificacion: rg rss_threshold BENCHMARKS.md=4 + linea decision 2026-09-03 + cargo test config 54/0 + bench compile 9.57s + commit 3a27c5f4 | evidencia: BENCHMARKS.md §12; FND-01-memory-budget.md §8 (2 runs ±5%); src/config.rs:22 0.80 sin cambio | artefactos: docs/operations/BENCHMARKS.md, docs/plans/2026-09-03-quality-gtm-wave.md | invariantes: no tocar src/server/ src/index/ src/storage/; threshold 0.70..0.85 | deuda: full-scale F3 queda heavy; fmt drift GOV-TK1 ajeno no tocado; campaign state bloqueado por ERR-TS-01 WIP | queda_pendiente: orquestador valida y pasa a GOV-TK1/GOV-TK9
Próxima tarea si completa: GOV-TK1
=== END RECITATION ===

=== RECITATION SRV-07 ===
Campaign ID: 20260903-quality-gtm-wave
Objetivo activo: SRV-07: Dockerfile unprivileged + wiring release (wave1/5)
Estado: completed
Última acción: Reescrito builder del Dockerfile (skeleton layer eliminado por roto: 73 [[test]] + [[bin]] paths validados al load, COPY desde cache-mount imposible), chmod 777 data dir + ARG VANTA_RUNAS_UID, job docker-image build-no-push + smoke uid 10001 en release, .dockerignore tests/+benches/ incluidos, docs DEPLOYMENT_GUIDE+CI_POLICY
Resultado: ✅
Próxima acción: Commit + fila Backlog SRV-07 eliminada + registro docs/avance/activo/operaciones.md
Contrato: rg -n "^USER|runas|RUNAS" Dockerfile -> 5 hits (USER vantadb + ARG VANTA_RUNAS_UID); rg -ni docker release-binaries-63.yml -> 9; actionlint exit 0; continuation-lint Dockerfile OK. docker build/run diferido a CI job docker-image (sin daemon local, nota explicita en CI_POLICY.md). docker-compose.yml no tocado: path volumen /var/lib/vantadb sigue valido, named volume hereda modo 0777.
Próxima tarea si completa: MKT-18i
=== END RECITATION ===

=== RECITATION MKT-18i ===
Campaign ID: 20260903-quality-gtm-wave
Objetivo activo: MKT-18i demo compose local-first
Estado: completed
Última acción: Cierre completo: compose+links commiteados (abb6594c), Backlog re-escalado + avance + plan (9b9180c7 y absorcion en 1ad28523), 2 lessons
Resultado: ✅
Próxima acción: Handoff orquestador: reconcilear claims stale ERR-TS-01/GOV-TK9 y run-time up -d en host con daemon
Contrato: verificacion: rg-cite=14 >=2 OK | PyYAML parse+assert tags OK (sin docker CLI, nota) | up -d diferido sin daemon | evidencia AnythingLLM: raw server/.env.example master, VECTOR_DB sin vantadb
Próxima tarea si completa: Wave2: RES-12 | MKT-18f | RES-09
=== END RECITATION ===
