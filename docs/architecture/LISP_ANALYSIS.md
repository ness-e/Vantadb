---
title: "LISP Experimental Analysis — Features to Recover"
type: architecture
status: draft
tags: [vantadb, architecture, query-language, lisp, experimental]
links: "[[EXPERIMENTAL_GOVERNANCE_DESIGN]], [[Backlog]]"
last_reviewed: 2026-07-04
aliases: [lisp-analysis]
---

# LISP Experimental Analysis

> **Source:** `archive/experimental-quarantine-2024-06/experimental-lisp/` (deleted Jul 2026)
> **Status:** Not salvageable — parser S-expressions + VM with 5 opcodes, only INSERT implemented.
> **Action:** Capabilities extracted as future features for SDK-level query composition.

---

## 1. Why It Was Deleted

The experimental LISP DSL had fundamental architectural problems:

| Problem | Details |
|---------|---------|
| **Only INSERT implemented** | MATCH, UPDATE, DELETE were stubs (returned `unimplemented!()`) |
| **Borrow checker conflicts** | VM state held references into the parser output; impossible to make the borrow checker happy without arena allocation |
| **GIL + async conflict** | The VM was sync Rust but VantaDB's runtime uses async; blocking inside an async context caused cancellation issues |
| **Fuel limit of 1000** | Hardcoded max operations — a real query would exhaust fuel before completing |
| **No query planning** | Pure AST walk — no optimization, no index selection, no cost estimation |
| **IQL already existed** | IQL (nom-based, 349 LOC) covers all the query functionality that LISP was trying to provide |

The correct approach is SDK-level query composition (`.then()`, `.pipe()`), not a DSL embedded in the engine.

---

## 2. Features We Lost (and How to Recover Them)

### 2.1 S-Expression Composition (Medium Value)

LISP's `(insert (into "memories") (value {"text": "hello"}))` was an alternative to JSON. The nesting allowed sub-expressions.

**Replace with:** IQL is flat by design. If composition is needed, use the SDK chain pattern:

```python
# Current IQL:
client.query('INSERT INTO memories {"text": "hello"}')

# Future composed (Phase 5):
client
  .query("INSERT INTO memories")
  .value({"text": "hello"})
  .execute()
```

### 2.2 Pipelining (High Value)

LISP's `(|> )` threading operator would pass the output of one expression as input to another.

**Replace with:** SDK-level pipe/chaining:

```python
# Conceptual Phase 5 API:
results = (
    await client
    .search("quantum computing")
    .filter({"year": {"$gt": 2020}})
    .limit(10)
    .pipe(memory.consolidate)
)
```

### 2.3 Recursive Sub-Queries (High Value)

LISP allowed nesting queries. IQL explicitly rejects `(` at the start of input (`if input.starts_with('(') -> error`).

**Replace with:** Parameterized sub-queries in SDK, not in IQL grammar:

```python
# Conceptual Phase 5 API:
sub = await client.search("neural networks").limit(5).collect()
results = await client.search("deep learning").combine(sub, strategy="intersect")
```

### 2.4 VM with Fuel (Low Value)

The VM approach (5 opcodes: Push, Pop, Call, Return, Halt) with fuel limit was interesting for sandboxing but impractical.

**Replace with:** IQL already has `LIMIT` and `TIMEOUT`. No need for a VM.

### 2.5 S-Expression Parsing (Low Value)

LISP used a custom `parse_sexpr()` tokenizer. Nom-based IQL parsing is more maintainable and performant.

**No replacement needed:** IQL parser is superior.

---

## 3. Phase 5 IQL Enhancement Recommendations

The ROADMAP.md defers Query Language decisions to Phase 5. Based on this analysis:

### 3.1 SDK Query Builder (TSK-XXX)
```python
# Builder pattern instead of string concatenation
q = client.query_builder()
q.insert().into("memories").value({"text": "hello"}).tags(["ai", "demo"])
q.execute()
```

### 3.2 Pipeline Operators (TSK-XXX)
Support for chaining multiple operations:

```
search("term") → filter("year > 2020") → sort("relevance") → limit(10)
```

### 3.3 DO NOT Reintroduce LISP
Implementing recursive LISP in the Rust engine was the wrong architecture. Push complexity to the SDK layer, keep IQL simple and flat.

---

## See Also

- [[EXPERIMENTAL_GOVERNANCE_DESIGN]] — Governance was the companion system to LISP
- [[Backlog]] — GOV-01 governance redesign, Phase 5 IQL enhancements
- [[docs/strategy/ROADMAP.md]] — Query Language decision deferred to Phase 5
