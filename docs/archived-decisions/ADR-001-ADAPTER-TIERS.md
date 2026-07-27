# ADR-001: Adapter Tier Classification & Surface-Area Governance

## Status
**Accepted** — 2026-07-22

## Context

VantaDB ships 17+ crates, adapters, and SDKs across Rust, Python, TypeScript, and WebAssembly. As the ecosystem grows, we need a governance model that:

1. **Sets clear expectations** for users about which adapters are production-ready
2. **Guides maintenance effort** — not all adapters deserve equal attention
3. **Prevents surface-area bloat** — every adapter is a contract we maintain
4. **Provides a deprecation path** for adapters that don't gain traction

Without tiering, every adapter appears equally supported — which is misleading and creates support burden on under-tested adapters.

## Decision

Adopt a four-tier classification for all adapters and SDKs:

### Tier 1 — Core 🟢

Production-ready. Shipped to package registries. Full test coverage (≥9/10). CI-gated. Maintained by core team.

| Adapter | Type | Score | Package |
|---|---|---|---|
| `vantadb-openai` | Rust provider (PyO3) | 10/10 | PyPI |
| `vantadb-ollama` | Rust provider (PyO3) | 10/10 | PyPI |
| `vantadb-litellm` | Rust provider (PyO3) | 10/10 | PyPI |
| `vantadb-haystack` | Python adapter | 10/10 | PyPI |
| `vantadb-llamaindex` | Python adapter | 10/10 | PyPI |
| `vantadb-langchain` | Python adapter | 10/10 | PyPI |
| `vantadb-openai` | Python adapter | 10/10 | PyPI |
| `vantadb-ollama` | Python adapter | 10/10 | PyPI |
| `vantadb-python` | PyO3 SDK | — | PyPI |
| `vantadb` | Rust core crate | — | crates.io |

### Tier 2 — Community 🟡

Shipped and functional (≥9/10). Smaller ecosystem or known gaps. Community contributions welcome. Not part of core release checklist.

| Adapter | Type | Score | Package | Gap |
|---|---|---|---|---|
| `vantadb-letta` | Python adapter | 10/10 | PyPI | Smaller user base |
| `vantadb-mem0` | Python adapter | 10/10 | PyPI | Smaller user base |
| `vantadb-crewai` | Python adapter | 9/10 | PyPI | Pydantic v2 PrivateAttr pattern |
| `vantadb-dspy` | Python adapter | 9/10 | PyPI | Edge cases in forward() |

### Tier 3 — Experimental 🟠

Proof-of-concept or early-stage. Not production-ready. Not shipped to registries. May be removed without a major version bump.

| Adapter | Type | CI gate | Notes |
|---|---|---|---|
| `vantadb-wasm` | WASM bindings | check only | No browser test runner in CI |
| `vantadb-mcp` | MCP Server | full test | Functional but narrow use case |
| `vantadb-server` | HTTP Server | full test | Under active development |
| `vantadb-ts` | TypeScript SDK | none | Minimal test coverage |

### Tier 4 — Platform 🏗️

Infrastructure that is not an adapter per se but part of the product surface.

| Component | Type | CI gate |
|---|---|---|
| `web/` | Frontend | ci-web-11 (Next.js — build + lint + typecheck) |

### Surface-Area Governance Rules

1. **No adapter enters Tier 1 without CI gate**, published package, and ≥9/10 evaluation score
2. **Adapters in Tier 3 for >6 months** without progress → deprecation notice
3. **Adapters in Tier 3 for >12 months** → removed (can be resurrected from git history)
4. **Tier 2 → Tier 1 promotion** requires: 10/10 score, documented CI integration, and 3+ months of stable maintenance
5. **Tier 3 → Tier 2 promotion** requires: ≥9/10 score and CI gate
6. **New adapters start at Tier 3** by default
7. **Deprecation follows** `deprecation-and-migration` skill: announce, 3-month window, remove

### Surface-Area Decisions

#### Keep all current adapters
- All 12 Python/Rust adapters serve distinct frameworks or use cases
- No significant overlap between adapters (each targets a different framework)
- Maintenance cost is low (thin wrapper layers over core VantaDB API)
- Community presence: Haystack, LangChain, LlamaIndex, DSPy, CrewAI are established frameworks

#### Do NOT add adapters for:
- **Java/JVM** — No community demand; would require JNI bindings
- **Go** — No community demand; would require CGo bindings
- **.NET/C#** — No community demand
- **Ruby** — No community demand

New adapters require an ADR and community signal (GitHub issues, discussions) before implementation.

## Alternatives Considered

### Two-tier (Core / Experimental)
- Rejected: Too coarse — Community adapters (Letta, Mem0) are better than Experimental but not Core.
- Community tier gives users clear signal: "works well, smaller ecosystem."

### No tiering (status quo)
- Rejected: All adapters look equally supported. Users expect the same quality from all.
- Creates support burden when Experimental adapters have issues.

### By-language tiering (Rust vs Python)
- Rejected: Quality varies by adapter, not by language. Some Python adapters are 10/10.

## Consequences

### Positive
- Clear expectations for users browsing the ecosystem
- Prioritized maintenance: Core gets immediate attention, Community gets best-effort, Experimental may be removed
- Deprecation path for underperforming adapters
- ADR serves as onboarding doc for new contributors

### Negative
- Tier classification needs periodic review (every 6 months)
- Risk of Tier 2 becoming a "dead zone" — maintained but not improved
- Users may perceive Tier 2/3 as "bad" rather than "less mature"

### Mitigations
- Tier labels are descriptive, not judgmental
- Each tier has a clear promotion path
- Review every 6 months as part of release planning

## Related
- Supersedes informal scoring system used in 2026-07 campaign
- Reviewed against: test coverage, CI gates, package publication, evaluation scores
