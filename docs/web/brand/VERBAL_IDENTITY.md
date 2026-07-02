# VantaDB Verbal Identity

---

## 1. Voice Dimensions

VantaDB speaks with a consistent voice across all channels. Six dimensions define it:

| Dimension | Position | What it means |
|---|---|---|
| **Precision vs. Warmth** | 60/40 | Technically accurate but never cold. We explain hard things clearly, not drily. |
| **Confidence vs. Humility** | 70/30 | We own our benchmarks (0.998 Recall@10, 1.2ms p50) but credit the community. |
| **Visionary vs. Practical** | 40/60 | We point toward the future of AI infrastructure, then show you how to install it today. |
| **Direct vs. Decorative** | 80/20 | Say what it does. No fluff, no jargon traps, no "revolutionizing the paradigm." |
| **Serious vs. Playful** | 75/25 | We take performance seriously, not ourselves. A well-placed metaphor lands harder than a pun. |
| **Expert vs. Peer** | 50/50 | We lead with authority on vector databases, but we sit next to developers — not above them. |

---

## 2. Tone Matrix

Voice stays constant. Tone shifts by context:

| Context | Precision/Warmth | Confidence/Humility | Notes |
|---|---|---|---|
| **Docs & SDK reference** | 80/20 | 60/40 | Clear, complete, neutral. Let the code speak. |
| **Landing page / Hero** | 40/60 | 80/20 | Bold claim + immediate proof. "One install. Zero servers." |
| **Blog / Engineering** | 50/50 | 70/30 | Show the work. Benchmarks, trade-offs, why we chose HNSW over IVF. |
| **Social / Community** | 30/70 | 50/50 | Short, human, generous. Retweet contributors, celebrate wins. |
| **Enterprise / Sales** | 70/30 | 90/10 | Compliance, SLAs, audit trails. Confident, not pushy. |
| **Error messages** | 60/40 | 60/40 | "Connection failed: check `VANTA_HOST`. Here's the fix." — no stack trace dump, no "An error occurred." |
| **Changelog** | 50/50 | 80/20 | "v0.4.2: 22% faster hybrid queries. No config changes needed. Just upgrade." |

---

## 3. Writing Principles

### Principle 1: Show the number
```
Bad:  "VantaDB is extremely fast."
Good: "p50 latency: 1.2ms. Recall@10: 0.998. One install."
```

### Principle 2: Lead with the verb
```
Bad:  "A converged query engine that supports SQL, vector, and full-text search is what VantaDB provides."
Good: "Query with SQL, vectors, or plain text — in a single call."
```

### Principle 3: One binary, zero jargon traps
```
Bad:  "Leverage our unified OLAP-optimized multi-model converged data architecture."
Good: "One binary. Three query engines. Zero configuration."
```

### Principle 4: Infrastructure is invisible
```
Bad:  "Deploy and manage your VantaDB cluster with our enterprise orchestration layer."
Good: "`pip install vantadb-py`. You're running. No cluster, no config, no ops."
```

### Principle 5: Reframe the comparison
```
Don't name competitors in product copy. Instead, name the old way:
- "No server process to manage" (vs. "better than Weaviate")
- "No per-vector pricing"       (vs. "cheaper than Pinecone")
- "No external vector model"    (vs. "unlike Chroma")
```

---

## 4. Editorial Glossary

### Always Use

| Term | Why |
|---|---|
| **Embedded** | "Embedded database" is precise. Avoid "serverless" — that means something else. |
| **Converged** | Unifies SQL + vector + FTS in one engine. Use sparingly, define early. |
| **Hybrid query** | Search across semantic (vector) and lexical (FTS) dimensions. |
| **Zero-infrastructure** | No server, no cluster, no ops team required. |
| **Agent memory** | The dominant use case for AI agent state management. |
| **Canonical data model** | Our typed, nullable columnar layout. Use in docs, not marketing. |

### Use Carefully

| Term | Caution |
|---|---|
| **Serverless** | Ambiguous. VantaDB has no server at all — different category. |
| **Vector database** | Accurate but narrowing. VantaDB is SQL + vector + FTS. |
| **ACID** | True for single-node. Not distributed ACID yet. Qualify. |
| **Real-time** | Define what "real-time" means for us (single-digit ms). |
| **AI-native** | Powerful if backed by specifics. Empty if not. |

### Never Use

| Term | Replacement |
|---|---|
| **Revolutionize** | "Redefine" or skip entirely. |
| **Leverage** | "Use." |
| **Game-changing** | Show the benchmark instead. |
| **Best-in-class** | Show the recall number instead. |
| **Disrupt** | "Change how you build." |
| **Paradigm shift** | Just stop. |
| **Synergy** | Never. |

---

## 5. Voice Examples by Channel

### Landing Page
> **Headline:** The database that thinks with you.
> **Subhead:** One `pip install`. Vector search, SQL, and full-text search in a single binary. Zero servers. Zero ops. Sub-millisecond.

### Docs Introduction
> VantaDB is an embedded, converged database engine written in Rust. It combines vector search (HNSW), full-text search (BM25), and SQL analytics in a single binary with no external dependencies.

### Social / Community Post
> Shipped v0.4.2: HNSW indexing is 34% faster on ARM. No config changes needed — just `pip install --upgrade vantadb`. Thanks @contributor for the PR.

### Error Message
> `VantaDBError: Connection refused — is the database path correct?`\
> Tried `vantadb.connect("./my_db.vdb")`? Check that the parent directory exists and is writable.

### Enterprise Pitch
> VantaDB Enterprise ships as a single air-gapped binary. Deploy to FedRAMP environments, on-premises data centers, or air-gapped VPCs. SOC 2 Type II report available under NDA. SSO/SAML included.

---

## 6. Brand Personality (If VantaDB were a person)

VantaDB is a **senior infrastructure engineer who also teaches at a code bootcamp**.

They are:
- Quietly brilliant — their code compiles on the first try, but they don't mention it
- Generous with knowledge — they write the docs they wish they'd found
- Intolerant of unnecessary complexity — "Why does this need three config files?"
- Excited about AI, skeptical of hype — "Show me the latency distribution, not the slide deck"
- Approachable — they answer questions on Discord at 11pm on a Saturday

They would never:
- Say "let's circle back on that ask"
- Ship a breaking change without a migration guide
- Gatekeep knowledge behind enterprise sales calls
- Benchmark on a curated dataset and call it production-ready
