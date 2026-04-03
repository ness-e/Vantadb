# ConnectomeDB — Investor Pitch Deck (15 Slides)

> **Para uso en:** Y Combinator AI Batch, a16z OSS Fund, reuniones ángeles,
> GitHub Accelerator, Antler, pre-seed rounds.

---

## Slide 1: Title
```
ConnectomeDB
The Unified Database for AI Agents

"3 databases in 1 Rust binary"

[Logo]                              [Founder Name]
connectomedb.dev                          CEO / Creator
```

---

## Slide 2: The Problem
```
AI teams today need 3 separate databases:

  ┌──────────┐   ┌──────────┐   ┌──────────┐
  │ Pinecone │   │  Neo4j   │   │ Postgres │
  │ Vectors  │   │  Graphs  │   │   SQL    │
  │ $70/mo   │   │  $65/mo  │   │  $25/mo  │
  └──────────┘   └──────────┘   └──────────┘
       │              │              │
       └──────────────┼──────────────┘
                      │
              3 bills. 3 teams.
          3 failure points. $160/mo.

And they STILL can't do a single query that combines
vector similarity + graph traversal + relational filter.
```

---

## Slide 3: The Solution
```
ConnectomeDB: One binary. Three engines. Zero overhead.

  ┌─────────────────────────────────┐
  │           ConnectomeDB                │
  │  ┌─────────┬────────┬────────┐ │
  │  │ Vector  │ Graph  │  SQL   │ │
  │  │  HNSW   │  BFS   │ KV+B  │ │
  │  └─────────┴────────┴────────┘ │
  │     Single UnifiedNode struct  │
  │     15MB cold start            │
  │     Written in Rust            │
  └─────────────────────────────────┘

  One query. All three paradigms. <5ms.
```

---

## Slide 4: Demo (Live or Video)
```
# 1. Start (2 seconds)
$ docker run -p 3000:3000 connectomedb/connectomedb

# 2. Insert with auto-embedding (Ollama does the vectors)
> INSERT NODE#1 TYPE Persona { nombre: "Eros", bio: "Rust developer" }
✓ Node 1 inserted. Vector auto-generated (384d, 1.2ms).

# 3. Hybrid query (vector + graph + filter)
> FROM Persona SIGUE 1..3 "amigo" Amigo
  WHERE bio ~ "systems programming", min=0.85
  FETCH nombre
✓ 3 results in 4.1ms

# 4. Chat primitive (native conversational memory)
> INSERT MESSAGE USER "What is HNSW?" TO THREAD#1
✓ Message linked to Thread 1. Embedding stored.
```

---

## Slide 5: Market Size
```
TAM: Database market = $100B by 2028 (Gartner)

SAM: AI-native databases = $8.2B by 2027
     (Vector DBs alone: $3.1B — MarketsAndMarkets)

SOM: Local-first AI databases = $400M
     (Edge AI + privacy-first + on-prem demand)

Our entry: Developers building AI agents who need
a unified data layer without cloud lock-in.

Target personas:
  🧑‍💻 Solo AI devs building agents (250k+)
  🏢 Startups with <50 engineers (50k+)
  🏦 Enterprise on-prem mandates (10k+)
```

---

## Slide 6: Traction
```
Pre-launch metrics (update with real numbers):

  ⭐ GitHub Stars:        [XXX] (target: 500 first month)
  🐳 Docker Pulls:        [XXX]
  📦 Cargo Downloads:     [XXX]
  👥 Discord Members:     [XXX]
  📝 Blog Views:          [XXX]
  🔀 Forks:               [XXX]
  🐛 External PRs:        [XXX]

Milestones:
  ✅ 15 development phases completed
  ✅ Full IQL query language (parser → executor)
  ✅ Native Ollama integration (auto-embedding)
  ✅ Python SDK via PyO3
  ✅ Production-ready server daemon
  ✅ CI/CD pipeline on GitHub Actions
```

---

## Slide 7: Business Model
```
Open-Core (Apache 2.0 core + BSL enterprise)

Revenue streams:

1. ConnectomeDB Cloud (SaaS)          → 70% of revenue
   $29-$299/mo per tenant
   92% gross margin

2. Enterprise License (Self-hosted) → 20% of revenue
   $299/mo per node
   Sharding, SSO, compliance

3. Support & Consulting          → 10% of revenue
   $150/h, onboarding packages

Projected ARR:
  Year 1:  $96k   (80 Cloud + 5 Enterprise)
  Year 2:  $480k  (300 Cloud + 20 Enterprise)
  Year 3:  $1.8M  (800 Cloud + 50 Enterprise + marketplace)
```

---

## Slide 8: Competitive Landscape
```
                    Multimodel?     Local-first?    Rust?     Price
                    ───────────     ────────────    ─────     ─────
Pinecone            Vector only     ❌ Cloud        ❌         $70/mo
Qdrant              Vector only     ✅              ✅         $25/mo
Neo4j               Graph only      ❌ JVM heavy    ❌         $65/mo
pgvector            Vector+SQL      ❌ PG overhead  ❌         $25/mo
Weaviate            Vector+some     ❌ Go+heavy     ❌         $25/mo
SurrealDB           Multi (SQL)     ✅              ✅         Free
────────────────────────────────────────────────────────────────────
ConnectomeDB              Vec+Graph+SQL   ✅ 15MB start   ✅         $29/mo

Only ConnectomeDB does all three paradigms natively
in a single unified data structure.
```

---

## Slide 9: Technology
```
Architecture: UnifiedNode (single struct per entity)

  UnifiedNode {
    id:      u64,
    fields:  BTreeMap<String, FieldValue>,   // Relational
    edges:   Vec<Edge>,                       // Graph
    vector:  VectorData::F32(Vec<f32>),       // Vector
    flags:   u128 bitset,                     // CP-Index filter
  }

Key innovations:
  🔬 CP-Index: Co-located Pre-filter → O(1) bitset check before HNSW
  ⚡ Zero-copy: Bincode serialization over RocksDB pinned slices
  🧠 Auto-embedding: INSERT text → Ollama → vector, transparent
  🔒 Agent RBAC: Sub-graph isolation per agent role
  💬 Conversational nodes: INSERT MESSAGE natively

Written in 100% Rust. No JVM. No Python. No GC pauses.
```

---

## Slide 10: Why Now?
```
3 converging trends make this the right time:

1. 🤖 AI Agent Explosion (2025-2026)
   LangChain, CrewAI, AutoGen → all need persistent memory
   Current solution: glue code between 3 databases

2. 🏠 Local-First / Privacy Movement
   GDPR, data sovereignty, on-prem mandates
   "Your data never leaves your hardware"

3. 🦀 Rust Infrastructure Renaissance
   Turso, SurrealDB, Neon, Redb → Rust is eating databases
   Developer trust in Rust for critical infrastructure

ConnectomeDB sits at the intersection of all three.
```

---

## Slide 11: Go-To-Market
```
Phase 1 (Month 1-3): Developer Adoption
  → HackerNews launch, Rust forums, /r/MachineLearning
  → "Show HN" + blog posts + demo videos
  → Target: 500 stars, 50 Docker daily pulls

Phase 2 (Month 3-6): Community Building
  → Discord community, contributors program
  → Documentation site, online playground
  → Target: 2k stars, 50 forks, 10 external PRs

Phase 3 (Month 6-12): Revenue Launch
  → Cloud SaaS beta, Enterprise pilot customers
  → Ollama/LangChain official partnerships
  → Target: $8k MRR, 3 Enterprise clients

Phase 4 (Month 12-24): Scale
  → Distributed mode (v2.0), WASM playground
  → Series A positioning
  → Target: $40k MRR, 10k stars
```

---

## Slide 12: Team
```
[Founder Name]
  Solo founder (Phase 1-3 self-funded)
  Built entire engine: 15 phases, 20+ Rust modules
  Background: [Your background]

Hiring plan (post-funding):
  Month 1-3:   +1 Rust Systems Engineer (core engine)
  Month 3-6:   +1 DevRel / Community Manager
  Month 6-12:  +1 Cloud Infrastructure Engineer
  Month 12+:   +1 Sales Engineer (enterprise)
```

---

## Slide 13: The Ask
```
Pre-Seed Round: $250,000

Allocation:
  40% → Engineering (hire Rust engineer)        $100k
  25% → Cloud Infrastructure (AWS/Fly.io)        $62k
  20% → Marketing & DevRel                       $50k
  15% → Legal & Operations                       $38k

Runway: 12-14 months to reach $8k MRR

Key milestones this round funds:
  ✅ ConnectomeDB Cloud launch (SaaS)
  ✅ 5,000 GitHub stars
  ✅ 3 Enterprise pilot customers
  ✅ v2.0 with distributed mode
  ✅ Path to Series A ($2M at $20M valuation)
```

---

## Slide 14: Vision (5-Year)
```
2026: Best local AI database for agents (MVP ✅)
2027: Default database for AI agent frameworks
2028: Cloud platform for AI-native applications
2029: Enterprise standard for hybrid AI workloads
2030: The "Snowflake of AI databases"

"Every AI agent needs memory.
 We make that memory fast, unified, and private."
```

---

## Slide 15: Contact
```
[Founder Name]
[email]
[GitHub: github.com/ness-e/ConnectomeDB]
[Website: connectomedb.dev]
[Demo: connectomedb.dev/playground]

"Star us: github.com/ness-e/ConnectomeDB ⭐"
```

---

## Métricas Que Importan a Inversionistas

| Métrica | Por qué importa | Target Mes 6 |
|---|---|---|
| **Star velocity** | Señal de tracción developer | 200 stars/mes |
| **Docker pulls** | Uso real (no vanity) | 500/semana |
| **Time to first query** | DX quality | <60 segundos |
| **GitHub issues/PRs** | Community health | 20 open, 5 external PRs |
| **MRR** | Revenue validation | $1,500 |
| **WAU (weekly active users)** | Retention | 50 |
| **NPS** | Satisfaction | >50 |

---

## VC Targets (Priorizado)

### Tier 1 — OSS/Infra Specialists
| Fund | Why | Stage | Check Size |
|---|---|---|---|
| **Y Combinator** (AI Batch) | Rust DB fits their thesis | Pre-seed | $500k |
| **a16z OSS Fund** (ROSS) | Dedicated OSS infrastructure fund | Seed | $1-5M |
| **Amplify Partners** | Backed Render, Railway | Pre-seed | $250k-1M |
| **GitHub Accelerator** | Free program, no dilution | Pre-seed | $20k grant |

### Tier 2 — AI Infrastructure
| Fund | Why | Stage | Check Size |
|---|---|---|---|
| **Sequoia Arc** | AI infrastructure thesis | Seed | $1M+ |
| **Greylock** | Backed Databricks, Confluent | Seed | $2M+ |
| **Index Ventures** | Backed Elastic, Confluent | Seed | $1M+ |

### Tier 3 — LATAM / Emerging Markets
| Fund | Why | Stage | Check Size |
|---|---|---|---|
| **Antler** | Global pre-seed, LATAM presence | Pre-seed | $100-250k |
| **500 Global** | LATAM focus, technical founders | Pre-seed | $150k |
| **NXTP Ventures** | Argentina-based, infra interest | Pre-seed | $200k |
