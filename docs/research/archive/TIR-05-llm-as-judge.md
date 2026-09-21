# TIR-05: LLM-as-judge (0.0–1.0) para salidas sintéticas del task-system

> **Fuente:** `docs/Backlog.md` P18 · **Tipo:** Investigación/Decisión · **Fecha:** 2026-08-17
> **Callees:** `evals/` (northstar.mjs, eval-metrics.mjs, dora.mjs) · `campaign-server.mjs` (verify_cmd, detect_task_type) · `agent-03-orchestration.md` #35

## 1. Inventario: salidas del task-system sin ground-truth determinista

| Salida | Mecanismo de verificación HOY | ¿Ground-truth determinista? | Riesgo no cubierto |
|---|---|---|---|
| **Bloque `RESULTADO`/recitation** de sub-agentes (pipeline-full §3, agent-03 §12.3): `resumen`, `hallazgos`, `claim`+`evidencia`, `confianza`, `status` | El orquestador valida **parseabilidad** (formato) y que cada claim tenga campo `evidencia` | ❌ No — el *contenido* (¿el resumen refleja lo que se hizo? ¿la evidencia respalda el claim? ¿el status OK es honesto?) no es chequeable por regex | Fabricación: claim con evidencia vacía o que no respalda la afirmación; status `OK` maquillando `PARTIAL` |
| **Resúmenes condensados** de sub-agentes (1–2K tokens, agent-03 §12.3) | Ninguno sobre el contenido; el lead sintetiza a ojo | ❌ No | Pérdida/fabricación de información en la condensación (anti-patrón #6: "sub-agentes que no verifican") |
| **Categorizaciones** (`campaign_detect_task_type`, `deriveType` de los evals) | Regex determinista sobre `Archivos clave` / taskId (`TYPE_PATTERNS`, `campaign-server.mjs:843`) | ✅ Sí — regex sobre input fijo | Ninguno (no hay categorización LLM hoy) |
| **Verify de comandos** (`campaign_verify_cmd`) | `execSync` + compare exit code (`campaign-server.mjs:792`) | ✅ Sí | Ninguno (mecánico puro) |
| **Evals North Star / DORA** (northstar.mjs, eval-metrics.mjs, dora.mjs) | Parsean `verify-log.jsonl` + plan files + budget.json: tasa primer intento, FP, regresión, CFR, lead/cycle | ✅ Sí (miden el *proceso*, no el *contenido*) | Ninguno — son telemetría de proceso, no judges de contenido |
| **URLs citadas (GATE CITAS TSYS-13)** | Resolución mecánica de URL (webfetch/HEAD) | ✅ Parcial — resuelve si la URL vive, NO si respalda el claim | Cita viva pero irrelevante/fabricada |

**Conclusión del inventario:** el hueco real es **uno solo** — validación de contenido de los bloques `RESULTADO`/recitation (fabricación de claims y honestidad del status). Categorizaciones y verifies ya son deterministas.

## 2. Análisis: LLM-as-judge vs costo vs alternativas

### 2.1 Qué aportaría un judge (0.0–1.0)

Sobre el hueco identificado (bloques RESULTADO), un judge pointwise con rúbrica (3 criterios: *evidencia respalda claim*, *status honesto*, *resumen fiel al artefacto*) detectaría exactamente el anti-patrón #6 de agent-03 §10 y el escenario "Sospecha de fabricación" de §12.4 — hoy cubierto solo por ojo humano del lead. Es el patrón recomendado por Anthropic (agent-03 §5.3/§11): rúbrica 0.0–1.0 + pass/fail, empezar con ~20 muestras.

### 2.2 Costo por llamada (modelo barato, tier judge)

Volumen real del task-system: decenas de tareas/trimestre (no cientos de miles). Con judge en GPT-4o-mini ($0.15/$1M input, $0.60/$1M output) o Haiku 4.5 ($0.80/$4.00), una llamada judge de ~2K tokens input + ~200 output cuesta **~$0.001–0.002**. Aun con panel de 3 judges sobre 100 salidas/trimestre: **< $1/trimestre**. El costo NO es blocker.

### 2.3 Riesgos del judge (documentados)

- **Position bias** (favorece la 1ª/última posición en comparativas), **verbosity bias** (respuestas largas mejor puntuadas), **self-preference** (mismo modelo generando y juzgando), **scoring bias** (orden de rúbrica, IDs de score) — sesgos conocidos y medidos (ICJNLP 2025, arXiv 2506.22316).
- **Falsa confianza**: sin rúbrica calibrada contra ground-truth humano (~20 casos etiquetados), el score 0.0–1.0 parece objetivo y no lo es.
- Mitigaciones estándar: judge de familia distinta al generador, anchors en la rúbrica ("peor 0.2, típico 0.5, mejor 0.9"), panel de 2–3 judges si el costo lo permite.

### 2.4 Alternativas ya existentes (orden de barato → caro)

1. **Checks mecánicos (ya operativos):** regex de formato + GATE CITAS TSYS-13 (URLs resuelven) + `campaign_verify_cmd` (exit codes). Cubren ~95% de los fallos del pipeline: contrato roto, cita muerta, build roto.
2. **Evaluator-optimizer humano (ya operativo):** `vanta-review` (verdict approve/changes) + el lead como validador final del bloque RESULTADO — el "judge" de facto hoy, gratis y calibrado.
3. **LLM-as-judge proactivo** (lo evaluado): añade detección de fabricación sin ojo humano, pero requiere rúbrica calibrada + corpus etiquetado para no dar confianza falsa.

## 3. Recomendación

**DEFERIR** la implementación de un harness LLM-as-judge proactivo sobre salidas del task-system. Adoptar el patrón **reactivo** ya contemplado en agent-03 §12.4: ante sospecha de fabricación, correr un judge ad-hoc sobre la evidencia; si persiste, escalar a humano.

**Justificación:**
1. **No es costo** — el judge cuesta <$1/trimestre. Es falta de calibración: sin ~20 casos etiquetados contra ground-truth humano, el score 0.0–1.0 produce falsa confianza (sesgos de position/verbosity/scoring medidos en la literatura).
2. **Volumen bajo** — decenas de salidas/trimestre: el ojo humano del lead + vanta-review ya cubre el riesgo con mejor precisión que un judge no calibrado. La regla de agent-03 §3.2 aplica: "si la salida no paga el costo (aquí: el riesgo de fabricación es bajo y ya mitigado), es mala inversión".
3. **El 95% del pipeline ya es determinista** — categorizaciones (regex), verifies (exit code), citas (TSYS-13), evals North Star/DORA (parseo). El hueco restante (fabricación en RESULTADO) está cubierto por review humano; solo se vuelve relevante si la escala o la frecuencia de fabricación suben.

**Trigger de revisión (cuándo reabrir):** (a) el task-system supere ~100 salidas sintéticas/trimestre, o (b) aparezca un caso real de fabricación no detectado por el review humano, o (c) exista un corpus etiquetado de ≥20 RESULTADO blocks con veredicto humano para calibrar la rúbrica. En ese punto, implementar: judge pointwise barato (GPT-4o-mini/Haiku, familia distinta al generador), rúbrica de 3 criterios con anchors, ejecutado post-hoc sobre tareas completadas (modo observacional, sin gate) antes de volverlo gate.

**WONTFIT explícito:** judge sobre *categorizaciones* — ya son deterministas por regex; un LLM ahí sería reemplazar código por azar.

## Fuentes

- agent-03-orchestration.md §5.2/§10/#6/§11/§12.4 (patrón judge 0.0–1.0, sospecha de fabricación)
- `campaign-server.mjs:758-889` (verify_cmd mecánico, detect_task_type regex)
- `evals/northstar.mjs`, `evals/eval-metrics.mjs`, `evals/dora.mjs` (evals 100% parseo mecánico)
- Position bias en LLM-as-judge: aclanthology.org/2025.ijcnlp-long.18.pdf
- Scoring bias (orden de rúbrica, IDs de score): arXiv 2506.22316
- Panel-of-LLMs (PoLL) reduce self-preference; costo judge ~$0.001–0.01/llamada en modelos baratos (GPT-4o-mini $0.15/$0.60 por M tokens; Haiku 4.5 $0.80/$4.00): medium.com/adnanmasood rubric-based-evals, openai.com/gpt-4o-mini, anthropic.com/pricing