# MGR-19: Benchmarks propios y externos (Track G — validación, cero implementación)

## Metadata
- **Plan file:** programa P49 (Track G Validación); consumido por VER-08/EXE-02 (plan post-investigación D3)
- **Fuente:** Backlog `P49:833` · Decisión owner 2026-09-14: **alcance completo** (canonical_p99 + LoCoMo/LongMemEval/BEAM)
- **Esfuerzo:** 🟡 3-5d · **Prioridad:** 🔴 · **Tipo:** validación/research
- **Turns estimados:** 15-25
- **Creado:** 2026-09-25 · **last-synced:** 2026-09-25
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 1 (jueces/datasets exactos → resuelve vanta-research) · **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Alcance | `benches/` (canonical_p99, wal_throughput, crash_recovery, param_sweep, …23), `benchmarks/*.py`, `docs/user/operations/BENCHMARKS.md` |
| Callees | Ninguno productivo — solo lectura + ejecución + reporte |
| Implicaciones | Cero cambios de código productivo; riesgo metodológico (jueces, scope Python-vs-Rust) y de costo (BEAM 1M/10M → subset) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `benches/canonical_p99.rs`, `docs/user/operations/BENCHMARKS.md` §1-§2, `benchmarks/vantadb_local_bench.py` (cabecera/contrato)
- **Referencias hacia dentro:** baseline canónico, README tabla perf (`README.md:355-359`)
- **Referencias entrantes:** EXE-01/02/03/05, MGR-20, VER-08 consumen estos números
- **Veredicto impacto:** bajo — docs + ejecución de benches; el riesgo es publicar números sin protocolo

## Contrato
"BENCHMARKS.md con baseline canónico + §2 reconciliada (0 claims sin comando reproducible); research-doc + Cierre MGR."

## Spec (SDD — Phase 1b)
Cero implementación, cero símbolos. No es feature-add, sin Spec. Cierre MGR obligatorio: research-doc + preguntas owner + plan de implementación.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** Regla 11 (número sin comando reproducible no existe); no tocar código productivo; BEAM solo subset (costo).
- **Comandos de verificación:** `cargo bench -p vantadb --bench canonical_p99` + `scripts/validate-docs-coverage.ps1`
- **Deuda pendiente:** ninguna al abrir

## Recitation
```
=== RECITATION ===
Objetivo activo: MGR-19 — benchmarks propios + externos
Estado: plan (desde: —)
Última acción: task file creado; brecha README↔§2 verificada (2.0ms vs 61.996ms)
Resultado: ⬜
Próxima acción: Step 1 — fork vanta-research (datasets/jueces) + correr canonical_p99
Contrato: ver ## Contrato
Invariantes: Regla 11; cero código productivo; BEAM-subset
Deuda: ninguna
Próxima tarea si completa: EXE-01 (demos con asserts)
last-synced: 2026-09-25
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto:** Sin deuda (research, no código).

## Definition of Done (3 niveles)
- **Task:** contrato + Cierre MGR (research-doc + preguntas + plan impl)
- **Commit:** `docs:` + MGR-19 (solo docs/benchmarks)
- **Release:** N/A (justificar: research sin cambio funcional)

## Herramientas necesarias
- `cargo bench`, `benchmarks/*.py`, `validate-docs-coverage.ps1`, webfetch (refs externas)
- **Skills cargadas (SDP):** writing-plans + writing-guidelines (números públicos — base campaign docs) + test-driven-development (harness = tests) + context-engineering

## Investigation Notes
- Brecha verificada: README p50 2.0ms vs BENCHMARKS §2 61.996ms (misma 10K/128d; mezcla scopes Rust vs Python+PyO3+GIL + tabla congelada).
- Refs externas: LongMemEval arXiv 2410.10813; Zep/Graphiti arXiv 2501.13956 (ya en Backlog).
- Destraba: EXE-01/02/03/05, MGR-20, VER-08.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 1 — datasets/jueces/harness exactos (resuelve vanta-research en Step 1) |
| Pendientes | 4 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — N/A: sin input externo, sin credenciales (input = exports propios; competidores solo su documentación pública).
- [x] **PERFORMANCE** — N/A como cambio; el benchmark ES la medición (Regla 9: before/after canónico; Regla 11: citar comando+entorno por número).

## Steps
### Step 1: Digest research + canonical_p99
- **Archivos:** `benches/canonical_p99.rs`, refs externas
- **Acción:** fork vanta-research (datasets LoCoMo/LongMemEval-S/BEAM-subset, jueces, protocolo) digest ≤500 palabras; correr `cargo bench -p vantadb --bench canonical_p99`
- **Verify:** baseline + entorno registrados
- **Estado:** ✅ DONE (digest recibido 2026-09-25, ver research-doc §2; compile-guard `--no-run` ✅ 6m46s; timed run en background `sh_0d9a857f` — baseline pendiente de transcribir al research-doc §1)

### Step 2: Reconciliar README↔§1↔§2
- **Archivos:** `README.md:355-359`, `docs/user/operations/BENCHMARKS.md`
- **Acción:** regenerar §2 (fósil pre-SIMD) o etiquetar scope Python-vs-Rust; cerrar brecha 2.0/62ms
- **Verify:** 0 claims sin comando reproducible (`validate-docs-coverage.ps1`)
- **Estado:** ✅ DONE (triple divergencia verificada: README 2.0/3.1ms vs artefacto local 2.78/5.70ms vs §2 61.996/179.810ms; §2 etiquetada FROZEN pre-SIMD con scope; README ahora cita §1 Rust canónico + 0 claims SDK-scope; `validate-docs-coverage.ps1` ✅ 0 gaps 2026-09-25)

### Step 3: Definir suites externas
- **Archivos:** `benchmarks/`, BENCHMARKS.md
- **Acción:** LoCoMo + LongMemEval-S + BEAM-subset con dataset commiteado y protocolo (hardware, juez, tokens)
- **Verify:** comandos de reproducción documentados
- **Estado:** ✅ DONE (protocolo + subset mínimo viable en research-doc §3; dataset commiteado = DEFER a VER-08 justificado: 3.03 GB + licencias LoCoMo/BEAM sin verificar; comandos reproducibles documentados)

### Step 4: Cierre MGR
- **Archivos:** `docs/dev/research/mgr-19-*.md`
- **Acción:** research-doc + preguntas owner + plan de implementación; commit `docs:`; progreso
- **Verify:** `campaign_verify_cmd` con el contrato
- **Estado:** ✅ DONE (research-doc + preguntas owner + plan impl; review P2-01 APPROVE tras Required; commit final `033f0cb0` docs-only 4 paths — `a31a9b8c` superseded por amend; lead verify: baseline timed transcripto §1)

## Dependencias
- Ninguna (destraba EXE-01/02/03/05, MGR-20, VER-08). Paralela libre a WIRE-09/01.

## Review (GATE P2-01)
- **Revisor:** vanta-review (sesión fresca, distinta del ejecutor)
- **Enfoque:** ¿protocolo reproducible? ¿scopes etiquetados? ¿jueces declarados?
- **Cómo se probó:** comandos re-ejecutables + baseline con entorno
- **Checklist anti-hábitos:** según plantilla
- **Veredicto:** APPROVE tras Required (2 fixes triviales aplicados: parentética stale MGR-19.md:105 + commit selectivo 4 paths excluyendo hunks ajenos CONTRIBUTING.md/RULES.md; opcionales aplicados: etiqueta recall README, frase deferral §11, scope wall-clock). Docs-only confirmado (`git diff --name-only HEAD -- src/ benches/ benchmarks/ evals/` vacío). Nits pre-existentes no tocados (BENCHMARKS §11 "p99 57ms" es de §4; scaling 4.88x vs cociente 5.08x).

## Notas
- WIP ajeno PROHIBIDO: archivos de API-01 — no tocar (esta tarea no toca código de todos modos).
- Costo BEAM: subset obligatorio; medir antes de escalar.
