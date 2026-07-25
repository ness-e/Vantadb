# Lessons Learned

> Persistencia estructurada de lecciones del Campaign Executor.
> Cada entrada: `YYYY-MM-DD | Task ID | Contexto | Lección | Acción tomada`

---

- 2026-07-15 | 1 | Task 1 (REL-02 — Publicar vantadb-ts en npm) → completed | Contract: `npm view vantadb version` muestra nueva versión
- 2026-07-15 | 2 | Task 2 (WEB-02 — Corregir claims falsos en landing) → completed | Contract: `grep -c "50x\|SQL support\|auto-embedding\|cloud" web/src/` devuelve 0
- 2026-07-15 | 3 | Task 3 (MKT-14 — Publicar 2 case studies) → completed | Contract: Ruta `/case-studies/` responde 200
- 2026-07-15 | 4 | Task 4 (MKT-03 — Show HN post) → completed | Contract: Post existe en news.ycombinator.com
- 2026-07-15 | 5 | Task 5 (LEG-01 — Registrar trademark "VantaDB") → completed | Contract: No verificable desde código. Marca registrada en USPTO + EUIPO.
- 2026-07-15 | 24 | Task 24 (REV-014 — 24 stale dependabot branches) → completed | Contract: `git branch -r | grep dependabot | wc -l` es 0 tras merge
- 2026-07-16 | COMP-001→030: Added 30 competitive features to Backlog.md (TIER 6). Prioritized as 7 🔴 + 17 🟠 + 6 🟡 from analyzing 27 VANTADB DOC OLD files covering 9 vector DBs, 8 graph DBs, and 10 architecture docs. Full reports in docs/audit-reports/.
- 2026-07-21 | DOC-API-01 | Task DOC-API-01 → completed
- 2026-07-21 | DOC-API-02 | Task DOC-API-02 → completed
- 2026-07-21 | DOC-API-03 | Task DOC-API-03 → completed
- 2026-07-21 | DOC-API-04 | Task DOC-API-04 → completed
- 2026-07-21 | DOC-API-05 | Task DOC-API-05 → completed
- 2026-07-21 | DOC-API-06 | Task DOC-API-06 → completed
- 2026-07-21 | ARCH-01 | Task ARCH-01 → completed
- 2026-07-21 | ARCH-02 | Task ARCH-02 → completed
- 2026-07-21 | ARCH-03 | Task ARCH-03 → completed
- 2026-07-21 | ARCH-04 | Task ARCH-04 → completed
- 2026-07-21 | ARCH-05 | Task ARCH-05 → completed
- 2026-07-21 | ARCH-06 | Task ARCH-06 → completed
- 2026-07-21 | OPS-01 | Task OPS-01 → completed
- 2026-07-21 | OPS-02 | Task OPS-02 → completed
- 2026-07-21 | OPS-03 | Task OPS-03 → completed
- 2026-07-21 | OPS-04 | Task OPS-04 → completed
- 2026-07-21 | OPS-05 | Task OPS-05 → completed
- 2026-07-21 | OPS-06 | Task OPS-06 → completed
- 2026-07-21 | OPS-07 | Task OPS-07 → completed
- 2026-07-21 | OPS-11 | Task OPS-11 → completed
- 2026-07-21 | OPS-08 | Task OPS-08 → completed
- 2026-07-21 | OPS-09 | Task OPS-09 → completed
- 2026-07-21 | OPS-10 | Task OPS-10 → completed
- 2026-07-21 | OPS-15 | Task OPS-15 → completed
- 2026-07-21 | OPS-12 | Task OPS-12 → completed
- 2026-07-21 | OPS-13 | Task OPS-13 → completed
- 2026-07-21 | OPS-14 | Task OPS-14 → completed
- 2026-07-21 | OPS-18 | Task OPS-18 → completed
- 2026-07-21 | OPS-16 | Task OPS-16 → completed
- 2026-07-21 | OPS-17 | Task OPS-17 → completed
- 2026-07-21 | OPS-19 | Task OPS-19 → completed
- 2026-07-21 | OPS-20 | Task OPS-20 → completed
- 2026-07-22 | Adapter research complete: 7/9 Python wrappers lack vector search (keyword match only). Framework adapters (LangChain, LlamaIndex, etc.) MUST be Python because their APIs are Python ABCs. Provider adapters (OpenAI, Ollama, LiteLLM) can be Rust because they're REST APIs. The Rust crates at root for framework adapters are architecturally wrong — PyO3 can't subclass Python abstract classes.
- 2026-07-23 | DRV-001 | Task DRV-001 → completed
- 2026-07-24 | 2026-07-23: Batch TIER 1 gate check: 7/8 already fixed as side effects. Always gate-check before scheduling implementation work.
- 2026-07-24 | 1 | Task 1 (`INT-01` — LangChain adapter → PyPI (tag push)) → completed | Contract: none
- 2026-07-24 | 2 | Task 2 (`INT-02` — LlamaIndex adapter → PyPI (tag push)) → completed | Contract: Mismo tag pusheado → CI publica ambos.
