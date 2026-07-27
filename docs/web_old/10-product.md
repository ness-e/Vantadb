---
title: Product & Brand Identity
type: design-doc
status: active
last_reviewed: 2026-07-05
tags: [product, brand, identity, vantadb]
---

# Product & Brand Identity

```
[ VANTADB ] >> product.identity
────────────────────────────────────────────────────
  PRODUCT:  VantaDB — The Local Memory Engine
  TAGLINE:  Performance you can benchmark. Control you can keep.
────────────────────────────────────────────────────
```

## What VantaDB Is

VantaDB is an embedded vector database for AI agents, tools, and
applications. Written in Rust. Built for local-first, privacy-preserving
semantic search at the edge.

It competes with ChromaDB, LanceDB, and Qdrant by being:
- **Faster** — benchmarks are a core product feature, not marketing
- **Smaller** — minimal footprint for edge deployment
- **Easier** — single binary, zero dependencies
- **Safer** — local-first, no data leaves your process

## Brand Voice

VantaDB's voice is **technical, confident, and direct**. It does not
sell. It presents facts, benchmarks, and architecture. It speaks to
engineers as equals.

| Trait | Manifestation |
|-------|---------------|
| Technical | Uses correct terminology, links to benchmarks |
| Confident | States facts without hedging or hyperbole |
| Direct | Short sentences. No marketing fluff. |
| Honest | Acknowledges tradeoffs, doesn't fake precision |
| Terse | Prefers fewer words. Not unfriendly — efficient. |

## Visual Identity

### The Mark

The VantaDB logo uses Space Grotesk at 800 weight. The wordmark is
two-tone: "Vanta" in foreground white, "DB" in amber accent. This
reflects the product's dual nature — general vector engine (Vanta)
+ database semantics (DB).

### Color Identity

Amber (#ff5500) is more than an accent color. It represents:
- **Signal** — in the noise of data, amber marks what matters
- **Heat** — performance under load, engine running hot
- **Caution** — precision tool with real consequences
- **The terminal** — amber cursor on black was the programmer's
  original interface

### Terminal Motif

VantaDB's design language draws heavily from the terminal:
- Dark background (#0a0a0a) as the CRT substrate
- Mono labels as command output
- Cursor blink as an active indicator
- Log lines as data presentation
- Code blocks as native content (not embedded)

The terminal is not decoration. It's a reference to the product's
primary interface — every interaction with VantaDB happens from
a command line or code.

## Design System Name: Nb

The design system is called **Nb** (Neubrutalism). This is:
- Short, memorable, two letters
- A reference to the aesthetic movement
- Pronounced "N-B" or "neubrutal"
- Consistently applied as `nb-` prefix everywhere

## Marketing vs. Product Pages

| Aspect | Marketing Pages (Home, Solutions) | Product Pages (Architecture, Benchmarks) |
|--------|-----------------------------------|------------------------------------------|
| Hero | Bold value prop + CTA | Terminal-style intro + spec |
| Content | Benefits, use cases, ecosystem | Architecture diagrams, data tables |
| Tone | Confident and direct | Technical and precise |
| CTAs | Install / Get Started | View docs / See benchmarks |
| Visual | Product imagery, metrics | Code blocks, architecture diagrams |

## Competitor Positioning

VantaDB does not claim to be "the best" at everything. Honest
positioning is part of the brand identity.

| Vs. | VantaDB Advantage | Tradeoff |
|-----|-------------------|----------|
| ChromaDB | 10-100x faster, smaller binary | Fewer integrations |
| LanceDB | True vector + text search | Newer ecosystem |
| Qdrant | Embedded, zero deps | Less mature networking |
| pgvector | Purpose-built for AI agents | Postgres integration missing |

## Benchmark Ethos

Benchmarks are not marketing material. They are product documentation.
Every benchmark on the site should be reproducible, dated, and
linked to the exact configuration used.

- Benchmark data uses tabular-nums for alignment
- Metric values use --text-metric scale
- Winner highlights use amber
- Comparison tables use the VsTable component

## Content Structure

### Hero Sections

```
Max 4 text elements:
1. Mono label (optional — max 1 per 3 sections)
2. Headline (max 2 lines)
3. Subtext (max 20 words, 4 lines)
4. CTAs (1 primary + max 1 secondary)
```

### Trust / Social Proof

Logo walls belong below the hero, never inside it.
- Logos only — no industry labels
- Real SVG marks for real companies
- Generated monograms for invented names

### Feature Presentation

- No zigzag alternating beyond 2 sections
- Bento grids must have exact cell count for content items
- Metrics use NbMetric component
- Code examples use NbCodeBlock

---

```
[ END PRODUCT ]
>> next: README.md
```
