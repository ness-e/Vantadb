# Plan de Ejecución: CleanCA Fase 3 — cerrar el gate + trait-split storage↔index (2026-09-13)

> **Fuente:** plan Fase 2 archivado (`docs/plans/archive/2026-09-13-cleanCA-fase2.md` §Cierre y §Fase 3:
> gate de 4 condiciones) + análisis DEFER 2026-09-13 (1/4 verde pleno, 3/4 parciales) +
> 5 decisiones humanas vía `question` 2026-09-13 (ver §Gate P).
> **Alcance:** SOLO lo que deja la mudanza Screaming cotizable + futuros con dueño de Fase 2.
> La mudanza en sí sigue DEFER y NO parte de este plan.
> **Estado:** ✅ COMPLETED (4/4, 2026-09-14) · **Rama:** `develop`
> **Norma:** guía `.opencode/references/clean-code-clean-architecture.md` (Apéndice V manda) +
> `docs/architecture/BOUNDARIES.md` (BND-01…08) + §10 Adaptador Fase 1/2 (sin campaign MCP para
> IDs no registrados; glob-antes-de-Read; ≥10 skills vía tool `skill`; verify en bash;
> no commitea worker; Gate D GO) + Regla 5 (ADRs los escribe el humano).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 4 (F3G + F3X + F3C + F3B, todas diseño-primero) |
| 🟡 DEFER | 1 (reorg física Screaming — se cotiza solo si el gate queda 4/4 verde tras esta fase) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬇️ downhill = 4 (ninguna uphill: todo tiene precedente M1/M3/S3 o decisión humana tomada)

## Gate P — decisiones humanas (2026-09-13, UNA ronda `question`)

| # | Pregunta | Respuesta humana | Efecto en el plan |
|---|----------|------------------|-------------------|
| 1 | Alcance | **4 tasks** | F3G + F3X + F3C + F3B, orden a→b→c/d |
| 2 | Parcial accumulator | **Excepción firmada** | F3G firma el self-loop como artefacto en BOUNDARIES/BND-03 (~1h, riesgo 0) |
| 3 | Trait-split, hipótesis inicial | **Hoja neutral** (patrón M3) | F3X diseña hoja neutral; el arch la valida y la devuelve para aprobar |
| 4 | S-split-config | **Mantener B+B** | F3C: anidado + fachada + unificación `VANTA_*→VANTADB_*` breaking mismo cambio |
| 5 | Bench CI | **Informativo primero** | F3B: job no bloqueante + verificación previa de toolchain (tantivy) |

## Gate P — triage (beneficio verificado vs costo)

1. Reorg física → DEFER (sigue valiendo el digest Fase 2: el ciclo y los 30+ usos se mudarían, no se eliminarían; se cotiza tras el gate 4/4).
2. Todo lo demás es cerrar parciales medibles o ejecutar decisiones ya tomadas (D0 B+B, bench-en-CI) — costo acotado, payoff: gate cotizable + deuda con dueño eliminada.
3. Diseño-primero en F3X/F3C (esfuerzo 🟡/🔴 con tradeoff): vanta-arch diseña → humano aprueba (question/ADR) → worker implementa por slices → review. Sin diseño aprobado no hay código.
4. `Config` split solo con B+B vigente + `apply_to` hot-reload como constraint inviolable (precedente D0).

## Tasks

### Wave 0 — cerrar los 3 parciales del gate (disjunta interna por archivos; 1 worker)

**Task 1: F3G — cerrar gate Fase 2 a 4/4 (re-medición M1 + excepción accumulator firmada + firma A1)**
- Appetite 4h · 🟢 · 🔴 Alta · Archivos clave: `docs/reviews/dsm-baseline-slim.json`, `docs/architecture/BOUNDARIES.md` (BND-03), `docs/architecture/ARCHITECTURE.md`, `src/accumulator.rs` (solo lectura)
- Gate Justificación: el análisis DEFER midió 1/4 verde + 3/4 parciales; sin este cierre la Fase 3 no es cotizable y el gate queda en promesa.
- Contrato: (1) re-medición same-tool M1 post-Fase-2 archivada en `docs/reviews/` con tabla D antes/después de `storage/engine` y `sdk` (D debe bajar vs baseline: 26→24→20 campos); (2) excepción `GraphAccumulator↔new` firmada en BOUNDARIES/BND-03 como artefacto del analizador (decisión humana Q2); (3) revisión vanta-arch de A1 firmada en el doc (cierra el "pendiente del lead" de C2A1). Todo verificable por archivo.
- Task file `docs/tasks/F3G.md` · ✅ COMPLETED · Ruta vanta-worker (con revisión vanta-arch para la firma A1).
- Slices: G1 re-medición (rust-dsm pin `5950a18` + `cargo modules dependencies --lib --acyclic`, comparar same-tool, tabla D) → G2 excepción firmada (párrafo BND-03 + justificación artefacto-ctor + firma) → G3 firma A1 (arch revisa BOUNDARIES contra código actual, firma + fecha; si halla deriva → hallazgo, no reescritura).
- Skills (≥10 vía `skill`): campaign-executor, progreso, codebase-memory, systematic-debugging, test-driven-development, code-review-and-quality, doubt-driven-development, source-driven-development, planning-and-task-breakdown, observability-and-instrumentation, documentation-and-adrs.
- MCP/tools: `codebase-memory-mcp_get_architecture`, `check_index_coverage`; bash: `node dist/cli/commands.js . -f json`, `cargo modules dependencies --lib -p vantadb --acyclic`, `cargo fmt --check` (si toca `src/`, no previsto), `jq`/`ConvertFrom-Json`.
- Investigación (obligatoria en DISCOVERY): releer C2M1.md (sesgos same-tool), C2S3/C2S3b post-números, C2A1 (qué falta firmar); web solo si el arch duda del modelo de artefacto (citar o `[cita NO VERIFICADA]`).
- Revisión/análisis/comprobación/validación: auto-revisión del diff (docs-only salvo re-medición) + `question` si la re-medición contradice la hipótesis D-a-la-baja (Gate V) + verify contrato (3 artefactos presentes y firmados).
- Dependencias: ninguna (primera). Impacto: docs + `docs/reviews/`; cero `src/` (salvo lectura). Habilita la cotización de la mudanza.
- Verify: JSON post archivado + tabla D + párrafo de excepción con firma + firma A1 con fecha + task file con las 3 evidencias.

### Wave 1 — trait-split storage↔index (diseño-primero; tras F3G)

**Task 2: F3X — diseño + implementación trait-split storage↔index (hoja neutral, patrón M3)**
- Appetite 2–3d · 🔴 · 🔴 Alta · Archivos clave: `src/storage/archive.rs:4`, `src/storage/engine/mod.rs:33-34`, `src/storage/engine/init.rs:17`, `src/storage/engine/maintenance.rs:9` (storage→index: `CPIndex`/`IndexBackend`); `src/index/mod.rs:20`, `src/index/serialize/file.rs:2`, `src/index/graph/types.rs:5`, `src/index/flat.rs:18` (test-only), `src/index/search/layer.rs:13` (index→storage: `vfile`/`MmapMut`/`FLAG_TOMBSTONE`)
- Gate Justificación: es el prerrequisito real de la mudanza (bidireccional verificado en A1 y Fase 2) y el único DEFER sin task/dueño; hipótesis inicial humana: hoja neutral (Q3), validada por el arch.
- Contrato: `cargo modules dependencies --lib -p vantadb --acyclic` sin aristas storage↔index (artefacto accumulator exceptuado por F3G) + edge-audit `rg` de los 8 usos a ruta neutral + suites storage/index verdes + sin cambio semántico.
- Task file `docs/tasks/F3X.md` · ✅ COMPLETED · Ruta vanta-arch (diseño + ADR-datos) → vanta-worker (slices).
- Slices: X0 diseño arch (hoja neutral: qué traits, qué dirección consumen, qué se mueve vs qué se queda; ADR-datos; `question` al humano para aprobar el diseño — sin aprobación no hay código) → X1 hoja + migración storage→index → X2 migración index→storage → X3 suites + acyclic + edge-audit + cierre.
- Skills: base 9 (como F3G) + `api-and-interface-design` (contract-first traits), `performance-optimization` (hot path: sin optimizar, solo mover; bench en CI si toca hot path).
- MCP/tools: `codegraph_explore` (callers de `CPIndex`/`IndexBackend`/`vfile`/`MmapMut`), `detect_changes`; bash: `cargo check -p vantadb --tests --all-targets`, `clippy -D warnings`, `fmt`, `nextest -p vantadb --lib storage index`, `cargo modules dependencies --lib -p vantadb --acyclic`.
- Investigación: DISCOVERY re-mide el ciclo en el árbol actual (si difiere de A1 → HALLAZGO); internet solo si ambigüedad de patrón (sealed traits, hojas neutrales).
- Revisión/análisis/comprobación/validación: diseño aprobado por humano (Gate D) + por slice verify mecánico + `Box`/re-export que oculte el síntoma NO vale (criterio M3) + vanta-review visa el diff antes del commit del lead.
- Dependencias: F3G (números y excepción como base). Impacto: padres = engine + index; riesgo = firmas `pub(crate)` (verificar 1 línea en DISCOVERY; si algo es pub → ADR + semver). Habilita cotizar la mudanza.
- Verify: acyclic sin aristas storage↔index + `rg` 8 usos a neutral + suites verdes + diseño aprobado archivado en task file.

### Wave 2 — S-split-config + bench CI (disjuntas entre sí; tras F3X)

**Task 3: F3C — S-split-config B+B (anidado + fachada + unificación breaking, hot-reload intacto)**
- Appetite 2–3d · 🔴 · 🟠 · Archivos clave: `src/config.rs:250-466` (~52 campos), `parse_env_or` desde `:538`, `apply_to` `:203` (8 campos hot-reload); 102 ficheros con literales `Config {`; 42 env vars (7 `VANTA_*` legacy)
- Gate Justificación: decisión D0 B+B vigente (Q4 la ratifica); constraints verificados en C2D0.md (hot-reload 100% en config.rs; fachada conserva `Default`/`with_*`/acceso plano).
- Contrato: sub-structs por dominio (`StorageCfg`/`ServerCfg`/`LlmCfg`/`EvictionCfg`/`PoolCfg`/`RbacCfg`, nombres a validar en diseño) + fachada `Config` plana + `apply_to`/watcher intactos + env unificadas a `VANTADB_*` (breaking documentado en changelog) + `cargo check` verde en los 102 sitios + suites config verdes.
- Task file `docs/tasks/F3C.md` · ✅ COMPLETED · Ruta vanta-arch (diseño + ADR-datos) → vanta-worker (slices por dominio).
- Slices: C0 diseño arch (dominios, fachada, orden de migración, ADR-datos; `question`/ADR humano Regla 5 — sin decisión no hay código) → C1..Cn un dominio por slice (mecánico `rg` + check por slice) → Cn+1 unificación env + changelog + suites + cierre.
- Skills: base 9 + `documentation-and-adrs`, `doubt-driven-development`.
- MCP/tools: `codegraph_explore Config` (constructores), `detect_changes`; bash: `cargo check -p vantadb --tests --all-targets`, `clippy -D warnings`, `fmt`, `nextest -p vantadb config`, `rg -l "Config \{"`.
- Investigación: git-log-por-sección actualizado (si el churn cambió desde D0 → HALLAZGO);-docs `config-rs`/`figment` ya evaluados en D0 (no re-evaluar salvo divergencia).
- Revisión/análisis/comprobación/validación: ADR humano firmado + por slice check verde + suites config + `question` si un slice revela breaking no previsto (Gate V) + review del diff.
- Dependencias: F3X (evita colisionar en `storage/engine/init.rs` si el trait-split lo toca). Impacto: 102 sitios mecánicos; riesgo = breaking env (documentado, major/changelog). Habilita: nada (es fin en sí misma).
- Verify: fachada plana + `apply_to` intacto + 0 `VANTA_*` legacy + suites + ADR firmado en task file.

**Task 4: F3B — job CI informativo `canonical_p99` (verificar toolchain primero)**
- Appetite 4h · 🟢 · 🟡 · Archivos clave: `benches/canonical_p99.rs`, `docs/operations/BENCHMARKS.md`, `.github/workflows/` (nuevo job), `Cargo.toml` (features del bench)
- Gate Justificación: decisión humana Q5 (informativo primero) + deuda Fase 2 (bench-en-CI al mergear S3/S5); sin número no hay optimización medible (Regla 9).
- Contrato: job nuevo no bloqueante que (1) verifica que la toolchain de CI compila el bench en release (aquí tantivy rompe el build local — si en CI también rompe, el job lo reporta como hallazgo con dueño en vez de verde falso), (2) corre `canonical_p99` y publica el número vs baseline `BENCHMARKS.md`, (3) ante regresión abre hallazgo (no bloquea el merge en esta fase).
- Task file `docs/tasks/F3B.md` · ✅ COMPLETED · Ruta vanta-lead (CI) + vanta-worker (bench).
- Slices: B1 toolchain-check (compilar bench en release en CI; si falla → hallazgo + `question`: arreglar toolchain o documentar) → B2 job informativo + baseline publicado → B3 cierre (doc + task file).
- Skills: base 9 + `observability-and-instrumentation`.
- MCP/tools: bash: `cargo bench -p vantadb --bench canonical_p99 -- --help` (compila), `actionlint`, `yaml` parse; CI logs como evidencia.
- Investigación: leer `BENCHMARKS.md` baseline + workflow de bench existente (`heavy-bench-nightly-51.yml`) antes de crear el job (no duplicar).
- Revisión/análisis/comprobación/validación: job verde-en-CI (o hallazgo toolchain con dueño) + número publicado + `question` si la toolchain está rota (Gate V: arreglar toolchain vs documentar).
- Dependencias: ninguna dura (disjunta de F3C); corre en Wave 2 con F3C si hay RAM (archivos disjuntos: workflows/benches vs config).
- Verify: job existe + corre en CI + número publicado vs baseline (o hallazgo toolchain documentado).

## Cierre (2026-09-14, 4/4 + retrospectiva)

> Verificación de cierre: 4/4 con verify verde + 6 commits (F3G `dd892c4c`, F3X-diseño
> `0afa8181`, F3X-impl `13f0f729` feat!:+ADR-042, F3B `e33c307f`, F3C-diseño + F3C-impl
> `d75459fe` feat!:+BREAKING CHANGE) + `cargo fmt --check` limpio + tree limpio salvo
> ajeno. Gate Fase 2: 4/4 verde con evidencia. `skill progreso`: campaña registrada en
> `docs/avance/` + nota en `meta.md`; plan archivado a `docs/plans/archive/`.

### Retrospectiva Start/Stop/Continue
- **Start:** Gate V con `question` ante muros reales (F3X pub-sigs → ADR-042 en el acto;
  sin el gate, la hoja quedaba muerta o el breaking entraba sin ADR).
- **Stop:** asumir que el diseño sobrevive intacto a la implementación (F3X requirió
  ajuste H2 + traits `pub` por benches; F3C fachada-vistas en vez de sub-structs
  almacenados — registrar la divergencia, no esconderla).
- **Continue:** diseño-primero con Gate D humano + review P2-01 antes del commit del lead
  (H1 era cambio semántico real con suites verdes: solo el ojo fresco lo caza).
- **Acción medible:** todo hallazgo de implementación que contradiga el diseño "= HALLAZGO +
  Gate V, nunca migración silenciosa" (métrica: 0 divergencias no registradas por task).

### Estado DEFER tras Fase 3
- **Sigue DEFER:** reorg física Screaming — ahora COTIZABLE (gate 4/4 verde), no ejecutada
  (nunca fue parte de ningún plan). Próximo paso si se quiere: plan de mudanza incremental.
- **Futuro con dueño:** firma humana ADR-043 (Regla 5) + revisit FIND-89 (env::var directos).

## SKIP / DEFER / BLOQUEADO

- DEFER: reorg física Screaming (se cotiza solo con gate 4/4 verde tras F3G+F3X; incremental por dominio, nunca big-bang; lo estable no se toca).
- BLOQUEADO: nada (F3X-diseño y F3C-diseño requieren aprobación humana, pero eso es Gate D por diseño, no bloqueo).
- SKIP: tocar `server/` (sano), Rust core docs (~100%), TS prod, reescribir bench H2 local (tantivy roto aquí; se verifica en CI).

## Grafo / Waves (FAIL_MODE=parallel, MAX 3)

```
Wave 0: F3G (cierra el gate: números + excepción + firma)
Wave 1: F3X-diseño → F3X-impl (tras F3G; sin F3G no hay base de medición)
Wave 2: F3C + F3B (disjuntas: src/config.rs vs workflows/benches)
```

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| Traits pub = semver major | verificar visibilidad en DISCOVERY de F3X; si pub, ADR + major documentado (precedente ADR-041) |
| Hot path (F3X moves, F3C) | sin optimizar, solo mover; bench en CI (F3B) como red |
| Toolchain CI también rota (tantivy) | F3B-B1 lo detecta primero; `question`: arreglar toolchain vs documentar (no verde falso) |
| Estimaciones con confianza media | cada ejecutor re-mide en DISCOVERY y reporta divergencia como HALLAZGO |
| Re-medición M1 contradice D-a-la-baja | Gate V: `question` (investigar antes de declarar el gate verde) |

## Cierre

`/cleanCA` global (0 🔴) + gate 4/4 verde con evidencia + retrospectiva Start/Stop/Continue + archivar plan + `skill progreso` (Trigger 1 + D2). Tras el cierre, y solo entonces, cotizar la mudanza incremental por dominio.
