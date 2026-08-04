---
taskId: MKT-15
title: "Add competitive benchmark table (VantaDB vs LanceDB vs ChromaDB + Pinecone/Weaviate) to /benchmarks"
status: completed
branch: develop
commit: 68e18405e9dcca1254d39f699d9e0ddaad70e483
---

## Summary

Added a competitive benchmark section (§03) to `/benchmarks` (BenchmarksView) comparing
VantaDB vs LanceDB vs ChromaDB (real measured numbers) + Pinecone/Weaviate (marked "Managed").
VantaDB column is neon-highlighted; run-locally section renumbered §03→§04.

## Files touched (only MKT-15 files committed)

- `web/src/components/vanta/vanta-data.ts` — added `CompetitiveRow` type + `COMPETITIVE_TABLE` constant.
- `web/src/components/vanta/benchmarks-view.tsx` — import + new §03 section + §04 renumber.

## Implementation

Data source (verified, no invented numbers):
- `benchmarks/competitive_bench.py` → `docs/blog/benchmarks_vs_lancedb_chroma.md` (glove-100-angular, 10K vecs, 100 queries, top_k=10, median of 3, `--batch-size 999`).
- VantaDB: 301.5 ingest QPS / 7,330.1ms index / 241.4 QPS / p50 4.124ms / p99 6.129ms / recall 100% / 434.4MB.
- LanceDB: 92,294.1 / 3,087.0 / 197.5 / 4.978 / 8.953 / 22.8% / 390.7MB.
- ChromaDB: 2,227.6 / N/A(inc) / 591.1 / 1.650 / 2.744 / 95.6% / 386.5MB.
- Pinecone/Weaviate: NOT measured by the harness → measured cells show "Managed" (no fabricated QPS/latency). Only factual architecture rows (Deployment / Pricing / Durability) carry values.

## Verify history

- `npx tsc --noEmit`: source files clean. Note: `.next/dev/types/routes.d.ts` (stale dev-cache artifact) errors are pre-existing/unrelated — `next.config.ts ignoreBuildErrors`.
- grep 'Pinecone' in benchmarks-view.tsx: present (line ~365).
- `npm run build`: PASS (exit 0), `/benchmarks` prerendered static.

## Retro

- TS object literal can't take `key: Type[] = value`; annotate the whole `export const` with an inline type instead.
- `as const` on a row-array union breaks `.highlight` property access — use an explicit row interface + typed array.
- Do NOT run `competitive_bench.py` fresh (downloads ~1GB datasets); the blog (MKT-05, published today) is the authoritative recent source. docs/operations/BENCHMARKS.md §7 (2026-06-06) is historical (24.3 QPS) and intentionally not used.

## Not done (deliberate)

- No push. No plan file update. `vanta-data.ts` only touched for the competitive table (WEB-18 will add pricing there separately).