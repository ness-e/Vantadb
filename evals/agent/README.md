# evals/agent — Eval de agentes propios (FIND-151)

Disciplina verificable para `vanta-research` (digests) y `vanta-review`
(veredictos), sin costo de LLM-judge.

## Contrato

- 10 casos dorados en `golden_cases.json` (5 `digest` + 5 `verdict`).
- Gate: **L9 de `/audit full`** corre `python evals/agent/run_eval.py`; regresión = NO-GO.
- Nunca en `/audit quick` (costo/latencia: la verificación de URLs tarda segundos).

## Judge

Determinista stdlib (conteo de palabras, resolución HTTPS, regex de evidencia):
**0 tokens**. Giskard OSS (Apache-2.0, `pip install giskard`, verificado v3.0.0;
`giskard.agents`: `Chat`, `ChatWorkflow`, `Generator`, `Tool`) queda reservado
como judge LLM futuro solo en `full` (Wave 1 con FIND-149). Refs:
https://github.com/Giskard-AI/giskard · https://www.giskard.ai

## Reglas que mide

| kind | checks |
|------|--------|
| `digest` | texto ≤500 palabras; ≥1 URL `https://`; cada URL resuelve (HEAD→GET, timeout 10s; `--offline` salta red) |
| `verdict` | veredicto en `{PASS, CHANGES-REQUIRED, FAIL, NEEDS-FIX}`; claim no vacío; ≥1 evidencia `file:línea` o URL |

Más 2 sondas negativas internas (digest de 600 palabras y veredicto sin
evidencia DEBEN fallar — anti-pase vacuo).
