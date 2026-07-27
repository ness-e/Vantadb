# Site Map — VantaDB (Swiss + Neubrutalism)

> Versión: 2.0 | 2026-07-04 | Estilo: Swiss + Neubrutalism

---

## Route Structure (28 rutas, todas implementadas)

```
/                                    Home (landing page: Hero → Metrics → Core Engine → Quickstart → Architecture → Benchmarks → Use Cases → Ecosystem → Monolith)
├── /engine                          Core Engine — HNSW, BM25, WAL, PyO3, Zero-Copy
├── /architecture                    System Architecture — 5-layer stack, backends
├── /docs                            Documentation hub
├── /docs-api                        Redirect to /docs
├── /pricing                         Pricing — Free vs Enterprise
├── /use-cases                       Use Cases — AI Agents, Local RAG, IDE, Offline KB
├── /integrations                    Integration Ecosystem — Frameworks, LLMs, Memory, Deployment
├── /changelog                       Release history (v0.1.1–v0.1.5)
├── /config                          Configuration — environment, feature flags
├── /cost                            Cost analysis — VantaDB $0 vs competitors
├── /latency                         Latency benchmarks — p50/p99, Rust vs Python
├── /storage                         Storage — binary size, WAL, memory, backends
├── /security                        Security — no telemetry, encryption, memory safety
├── /maint                           Maintenance — backup, recovery, compaction
├── /about/                          About index
│   ├── /about/company               Company info
│   ├── /about/community             Community stats
│   └── /about/contact               Contact form
├── /blog/                           Blog index
│   └── /blog/$slug                  Dynamic blog post
├── /solutions/ai-agents             AI Agent Memory
├── /solutions/local-rag             Local RAG Pipeline
├── /solutions/ai-ide-tooling        AI IDE Tooling
└── /product/benchmarks              Benchmarks page
```

## Routes Status (28/28 — 100% redesigned)

| Route | Status | Design |
|-------|--------|--------|
| `/` | ✅ Swiss + Neubrutalism | Hero, Metrics, CoreEngine, Quickstart, Architecture, Benchmarks, UseCases, Ecosystem, Monolith |
| `/engine` | ✅ Swiss + Neubrutalism | nb-split-7-5, nb-frame, nb-pill-status, nb-block-amber |
| `/architecture` | ✅ Swiss + Neubrutalism | nb-asymmetric, nb-telemetry, nb-table, nb-block-amber |
| `/docs` | ✅ Swiss + Neubrutalism | nb-card grid, nb-frame, nb-bg-cross--faint |
| `/docs-api` | ✅ Redirect | — |
| `/pricing` | ✅ Swiss + Neubrutalism | nb-grid--cols-2, nb-card, nb-table, nb-pill-status |
| `/use-cases` | ✅ Swiss + Neubrutalism | nb-bento, nb-bento-cell, nb-frame |
| `/integrations` | ✅ Swiss + Neubrutalism | nb-grid--cols-2, nb-cell, nb-pill-status |
| `/changelog` | ✅ Swiss + Neubrutalism | nb-telemetry, nb-table, nb-pill-status |
| `/config` | ✅ Swiss + Neubrutalism | nb-split-7-5, nb-list, nb-block-amber |
| `/cost` | ✅ Swiss + Neubrutalism | nb-table, nb-telemetry, nb-bg-dot |
| `/latency` | ✅ Swiss + Neubrutalism | nb-asymmetric, nb-ticker, nb-bg-cross--faint |
| `/storage` | ✅ Swiss + Neubrutalism | nb-split-7-5, nb-list, nb-block-amber |
| `/security` | ✅ Swiss + Neubrutalism | nb-list, nb-telemetry, nb-block-amber |
| `/maint` | ✅ Swiss + Neubrutalism | nb-split-7-5, nb-list, nb-block-amber |
| `/about/` | ✅ Swiss + Neubrutalism | nb-grid--cols-4, nb-cell, nb-arrow |
| `/about/company` | ✅ Swiss + Neubrutalism | nb-split-7-5, nb-grid--cols-2, nb-list |
| `/about/community` | ✅ Swiss + Neubrutalism | nb-telemetry, nb-grid--cols-2/3, nb-arrow |
| `/about/contact` | ✅ Swiss + Neubrutalism | nb-telemetry, nb-grid--cols-3, nb-split-7-5 |
| `/blog/` | ✅ Swiss + Neubrutalism | nb-grid, nb-cell, nb-pill-status |
| `/blog/$slug` | ✅ Swiss + Neubrutalism | nb-telemetry, nb-pill-status, nb-frame |
| `/solutions/ai-agents` | ✅ Swiss + Neubrutalism | nb-telemetry, nb-list, nb-grid--cols-2 |
| `/solutions/local-rag` | ✅ Swiss + Neubrutalism | nb-telemetry, nb-list, nb-grid--cols-3 |
| `/solutions/ai-ide-tooling` | ✅ Swiss + Neubrutalism | nb-telemetry, nb-list, nb-grid--cols-3 |
| `/product/benchmarks` | ✅ Swiss + Neubrutalism | nb-list, nb-telemetry |

## Navigation

```
[ VANTADB ]     CORE ENGINE   ARCHITECTURE   AI AGENTS   LOCAL RAG   IDE TOOLING   USE CASES   PRICING
                                                                                                   [DOCS] [GITHUB]
```

## Section Mapping (Home Page)

| Order | Section | Component | CSS Pattern |
|-------|---------|-----------|-------------|
| 1 | Nav | `Nav.tsx` | `nb-frame` with `[ NAV ]` label, 2px borders |
| 2 | Hero | `SwissHero` | scanline overlay, glitch text, telemetry badges, pip install CTA |
| 3 | Metrics | `SwissMetricsBar` | `nb-frame` `[ METRICS ]`, 4 columns with vertical dividers |
| 4 | Core Engine | `SwissCoreEngine` | `nb-bento` with 1px gap grid, 6 features |
| 5 | Quickstart | `SwissQuickstart` | CRT terminal with amber glow, 4-step typewriter |
| 6 | Architecture | `SwissArchSection` | `nb-frame` `[ PIPELINE ]`, 6-stage card stack |
| 7 | Benchmarks | `SwissBenchmarkGrid` | `nb-frame` `[ BENCHMARKS ]`, comparison table |
| 8 | Use Cases | `SwissUseCases` | `nb-bento` 2×2 grid, `nb-frame` `[ USE CASES ]` |
| 9 | Ecosystem | `SwissEcosystem` | `nb-grid` with 1px gap, integration chips |
| 10 | Monolith CTA | `SwissMonolith` | `nb-block-amber`, pip install command, blinking cursor |
| 11 | Footer | `SwissFooter` | `nb-frame` `[ FOOTER ]`, 5-column metadata grid |

## Footer Links

```
Product:       Engine | Architecture | Pricing | Use Cases
Solutions:     AI Agents | Local RAG | IDE Tooling
Developers:    Docs | GitHub | Changelog | API
Resources:     Blog | About | Community | Contact
Legal:         MIT License | Apache 2.0
```
