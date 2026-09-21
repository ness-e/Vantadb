---
title: "ADR-013: Open Core Licensing (Apache-2.0 core + proprietary VantaDB Pro)"
type: adr
status: accepted
tags: [vantadb, architecture, adr, licensing]
created: 2026-08-06
last_reviewed: 2026-08-06
---

# ADR-013: Open Core Licensing — Apache-2.0 core + proprietary VantaDB Pro

## Context

VantaDB needs a sustainable business model without sacrificing adoption. The
engine is an embedded, local-first persistent-memory vector store targeting
local-first AI. Today the whole workspace is Apache-2.0 (`Cargo.toml`, `LICENSE`,
all crates). Commercial features (`encryption`, `wal-shipping`, `pitr`,
`prometheus`, `server`, `tls`) already exist as optional feature-flags inside
the core.

Research (2026-08-06) covered SurrealDB's BSL-1.1 model (whole-engine BSL with an
Additional Use Grant + change to Apache), STOA's Apache-vs-BSL analysis, Qdrant,
Supabase and dbt. Findings: for an embedded engine, relicensing the core (AGPL/BSL)
hurts exactly the enterprise-AI ICP and community, while the real moat is features
+ brand, not the license. The strategic manual (C3) already proposed this model.

## Decision

Adopt **Open Core**, decision D1/D2/D3/D4 (2026-08-06):

1. **Core `vantadb` stays Apache-2.0**, never relicensed, never gains Pro-only features.
2. **`vantadb-pro`** (commercial Pro/Enterprise) is **proprietary** ("all rights
   reserved" + per-customer license), lives in a **separate private repo**
   (`C:\Users\Eros\VantaDB Proyect\vantadb-pro`), **not** a workspace member.
3. **Delivery is compiled artifacts only** (private registry / signed on-prem
   `vantadb.license`), never source. Each Pro feature validates its license
   offline (expiry `yyyy-mm-dd` + max nodes), no server / call-home.
4. Commercial features beyond the gates are **new features** built in Pro, not
   moved out of the core (zero breakage, zero relicensing).

## Consequences

- Pros:
  - Maximum adoption of the Apache-2.0 core; no OSI-friction; enterprise-safe.
  - Moat is features + brand, recoverable regardless of the permissive license.
  - Zero risk to existing users; the core's gates/features are untouched.
  - Pro delivery via compiled artifacts + offline license avoids a licensing server.
- Cons:
  - The Apache-2.0 core can be legally forked by competitors (mitigation: brand
    + proprietary features + CLA for contributions).
  - First-party Pro features must be re-implemented in `vantadb-pro` (no borrow
    from the core) — slower to land.
  - Need discipline to keep Pro OUT of the workspace and deny.toml.
- Gatekeeping rules captured in `.opencode/rules/open-core-licensing.md`; plan:
  `docs/plans/2026-08-06-oc-vantadb-pro.md`; feature map: `docs/strategy/VANTADB-PRO-FEATURES.md`.