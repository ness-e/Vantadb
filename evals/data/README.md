# evals/data — datasets versionados para el harness (VER-08)

Sub-muestras commiteadas + procedencia. **Regla:** solo entra a este directorio
lo que se puede redistribuir con licencia compatible con el repo (Apache-2.0
core). Lo demás queda como runner opcional con descarga local (ver tabla).

## `longmemeval_s_subset.json` (commiteado)

| Campo | Valor |
|---|---|
| Dataset origen | `xiaowu0162/longmemeval-cleaned` — reemplaza al LongMemEval original (ICLR 2025, arXiv 2410.10813) |
| Split | `longmemeval_s_cleaned` ("LongMemEval_S": ~115K tokens / ~40 sesiones por pregunta) |
| Licencia | **MIT** (metadata HF `license:mit` + tag `license:mit`) — redistribuible |
| Revisión pinneada | `98d7416c24c778c2fee6e6f3006e7a073259d48f` |
| Archivo fuente | `longmemeval_s_cleaned.json` — 277,383,467 B — sha256 `d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442` (LFS oid) |
| Sub-muestra | 4 preguntas no-abstención + 1 abstención (`_abs`), primeras en orden del dataset; objetos **object-identical** (deep-equal, sin edición de campos ni truncado) |
| IDs | `e47becba`, `118b2229`, `51a45a95`, `58bf7951`, `0862e8bf_abs` |
| Artefacto | 2.70 MB, sha256 en `longmemeval_s_subset.json.meta.json` |
| Regenerar | `python evals/data/fetch_longmemeval_subset.py` (descarga con sha256 verificado) |

El sidecar `*.meta.json` es self-describing (fuente, selección, sha256).
Tamaño chico a propósito: es un **smoke reproducible offline**, no un leaderboard.

### LoCoMo — **NO commiteable** (disposición)

`snap-research/LoCoMo` (arXiv 2402.17753) usa **CC BY-NC 4.0**
(`LICENSE.txt`: "Attribution-NonCommercial 4.0 International"; GitHub lo reporta
como `NOASSERTION`). NonCommercial no es compatible con redistribuir datos
dentro de un repo de producto comercial → **no se commitea, no corre en CI**.
Si se necesita el smoke: descarga local del usuario aceptando la licencia,
fuera del repo. Fila `FIND` propuesta para VER-09 (dueño: vanta-tuner).

### BEAM — licencia MIT verificada; commit descartado por tamaño (disposición)

`mohammadtavakoli78/BEAM` (ICLR 2026, arXiv 2510.27246) = **MIT** (`LICENSE`,
GitHub `spdx_id: MIT`). El track 1M vive en el repo como `chats/1M/<n>/`
(`chat.json` ≈ 4.6 MB + `chat.pickle` ≈ 4.3 MB por conversación) → commitear
aunque sea 1 conversación excede el tamaño de los artefactos mayores del repo
(≤2.4 MB) y el evaluador upstream usa pickle/torch (peso y frágilidad).
Disposición: runner opcional con descarga local (documentado); nunca 10M en CI
(MGR-19 §3). Fila `FIND` propuesta (dueño: vanta-tuner, consumidor VER-09).

## Protocolo pinneado (aplica a todas las corridas del harness)

- **Juez determinista** (sin LLM): corrección = evidencia (`answer_session_ids`)
  dentro del top-k. El juez GPT-4o de LongMemEval (`evaluate_qa.py`) queda para
  el reporte manual (decisión owner Q1) — nunca para el gate.
- Dataset + sha256 + top_k + backend fijos; hardware en el reporte (Regla 11).
- Proxies declarados (token-economy, abstención) en `evals/memory_harness.py`.
