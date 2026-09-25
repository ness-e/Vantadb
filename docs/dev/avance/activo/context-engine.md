---
title: "Avance — Context Engine"
type: domain-log
status: active
tags: [vantadb, avance, context-engine, compression, budget, recall, mmd]
last_reviewed: 2026-08-22
aliases: []
---

# Avance — Context Engine

> Registro consolidado del trabajo completado sobre el context engine (F5): ensamblado de contexto con cascada de compresión LLM-free, estimación de tokens, MMD persistente y wiring productivo al pipeline worker. **IDs originales conservados.** Catch-up por campaña (no commit-por-commit). El crate fuente es `vanta-memory/` — su registro general vive en `vanta-memory.md`.

## Cobertura rápida

- **P29:** assemble con cascada mild/aggressive, token estimator chars/3 + emergency truncate, MMD persistente dedup pair-safe, assemble_with_recall como coordinador único de budget.
- **P31:** wiring productivo como fase post-L3 del pipeline con flag de config y budget compartido.

---

## Campaña P29 — Vanta Context Engine (F5)

### MEM-22/23/24 + MEM-37: Ensamblado, compresión y presupuesto — catch-up por campaña
- **Fecha:** 2026-08-20 → 2026-08-21
- **MEM-22** (`4d1363ec`): context engine assemble con cascada de compresión mild/aggressive — LLM-free, cursor-aware.
- **MEM-23** (`8de35359`): token estimator chars/3 + emergency truncate + `CompactionReport`.
- **MEM-24** (`ddc5671f`): MMD persistente sobre TaskMemory META — dedup por fingerprint, injector pair-safe.
- **MEM-37** (`ae7fe30b`): `assemble_with_recall` como coordinador único de budget (cursor boundary + test e2e).
- **Resultado:** ✅ Parte de la campaña 9/9 (cierre `00f18662`). Las piezas de memoria no-ensamblado viven en `vanta-memory.md` § P29.
- **Ids:** `MEM-22`, `MEM-23`, `MEM-24`, `MEM-37`

---

## Campaña P31 — Cierre Final

### MEM-43: Wiring productivo al pipeline worker
- **Fecha:** 2026-08-21
- **Resultado:** ✅ Commit `a0bcb112` — el context engine se ejecuta como fase post-L3 del pipeline de `vanta-memory`: flag de config para activarlo, presupuesto de tokens compartido entre compresión e inyección. Registro completo en `vanta-memory.md` § P31.
- **Id:** `MEM-43`
