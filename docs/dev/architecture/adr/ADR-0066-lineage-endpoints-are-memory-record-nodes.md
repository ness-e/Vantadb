---
title: "ADR-0066: Lineage endpoints are the memory record nodes, addressed by memory_node_id"
kind: adr
status: accepted
description: "scene:/persona:/episode:/record: are a phantom id space nothing materializes; lineage endpoints become memory_node_id(namespace, key), the address put already assigns to every record — no node creation, no src/ change, no re-index"
tags: [vantadb, architecture, vanta-memory, graph]
---

# ADR-0066: Lineage endpoints are the memory record nodes, addressed by memory_node_id

## Status

Accepted by `vanta-arch` (2026-11-26). Spec only — the implementation contract
is [`FIND-756`](../../tasks/FIND-756.md); acceptance does not mean the edges
exist yet. The change touches no `unsafe`, no FFI and no I/O primitive — it is
five id computations — so the audit stage has nothing to review; the chaos stage
is the deleted-episode path (the guard, D7); there is no throughput claim to
measure (§Measurement and its limits). Supersedes nothing. Corrects the premise
recorded in the `FIND-756` backlog row (`docs/dev/Backlog.md:801`), which claims
the change "cruza a `src/`" and cites `ADR-0054`; both are wrong (D8).

## Context and problem statement

`FIND-747` established, with evidence, that the lineage edges this crate tries to
write are never created. The four facts, re-verified here against `HEAD`
(2026-11-26) because the whole decision rests on them:

1. `add_edge` requires **both** endpoints as live nodes — `engine.get` +
   `Error::NodeNotFound` on the source
   (`src/sdk/api/graph.rs:129-131`) and on the target (`:141-143`).
2. The endpoint that is missing is the **origin** — the scene, the persona, the
   record — not the parent (the parent is the episode).
3. Those ids come from `entity_node_id`
   (`vanta-memory/src/core/ontology/mod.rs:45-52`), which is FNV-1a over a
   lowercased string: **it hashes, it does not create**.
4. The only production `insert_node` calls in the crate materialize **triplet
   entity** nodes (`ontology/mod.rs:221,230`). `entity_node_id` has **0 hits
   outside** `vanta-memory`.

So `scene:<s>:<name>`, `persona:<s>:<key>`, `episode:<s>:<id>`,
`record:<s>:<id>` are a **phantom id space**: names for nodes that no code path
ever writes. This is not an ordering defect in the cascade — nothing is
"about to be created" later in the pass. It is a wrong *representation of an
identity*: the entities exist, and their graph address was invented instead of
looked up.

The entities do exist, and they already have an address. Every memory record
**is** a node:

- `put` → `put_one` (`src/sdk/api/memory.rs:971`) → `insert_one_internal`
  (`:980`), which computes `node_id = memory_node_id(namespace, key)`
  (`:1143`) and `engine.insert(&node)` from `memory_record_to_node_owned`
  (`:1234`).
- `memory_node_id` is `XxHash3_128(ns ++ "\0" ++ key)` —
  `src/sdk/serialization/mod.rs:113-124` — and is **already public** to sibling
  crates via `pub use serialization::{… memory_node_id …}`
  (`src/sdk/mod.rs:32-36`).
- That address is the documented memory↔graph bridge (MEMG-03 / DIST-19 S6,
  `docs/api/EMBEDDED_SDK.md` DX-04).

The consequence of the phantom space is not cosmetic: `FIND-747` added a guard
(`ontology/mod.rs:262-302`) that makes the failure a `debug!` skip instead of an
`ERROR` per pass. The guard is correct, and it is also why the defect is
invisible — the lineage is silently absent while every pass reports success.

`FIND-296` ("L2/L3 sin linkage", `docs/dev/Backlog.md:60`) is the same defect
stated as an outcome, and `docs/dev/avance/activo/vanta-memory.md:202` already
claimed it closed by MEMG-03 which it was not.

## Considered options

### O1 — Address the endpoints with `memory_node_id(namespace, key)` (chosen)

Point every lineage endpoint at the node `put` already created for the record
that *is* that entity.

- Pros: no node is created, so there is no lifecycle to own — the node appears
  and dies with the record it belongs to (`forget`, cascade erase, supersession
  all already maintain it). One address per fact. Zero `src/` change
  (`memory_node_id` is already exported). Node ids are unchanged, so **no
  re-index and no HNSW/text rebuild**.
- Cons: inherits the two-owner situation described in D3 (declared, bounded);
  makes the cross-namespace lineage depend on a convention (`l1/<session>` is
  where episodes live) instead of a type.

### O2 — Materialize the phantom ids with `insert_node`

Do for scenes/personas/records what the triplet path does for entities: create
the node, then the edge.

- Pros: the `entity_node_id` space becomes real; no convention needed about
  which namespace an entity lives in.
- Cons: **two addresses for one fact** — the record's node at
  `memory_node_id(ns,key)`, plus an edge-only node at an FNV hash. It splits
  every traversal in two (a `derived_from` walk lands on a node with no payload,
  no `key`, no namespace) and it invents a garbage-collection problem that does
  not exist today: `carry_graph_state` (`memory.rs:169-179`) only preserves
  edges on the **record's** node, and `cascade_forget` deletes the record node —
  the entity node would be orphaned forever, invisibly. Rejected.

### O3 — Fix only the endpoint that is missing

Keep the parent (episode) as `entity_node_id` and fix only the origin.

- Pros: smallest diff.
- Cons: it does not create a single edge — `add_edge` still hits
  `NodeNotFound(target)`. This is the trap the finding's wording invites ("the
  missing one is the source"): one fixed end is not an edge. Rejected.

### O4 — Do not write edges; answer lineage from the metadata

`source_episode_ids` / `derived_from` are already canonical on the record.
Delete `ensure_lineage_edge` and let callers read the metadata.

- Pros: zero graph cost, zero new failure modes, and it makes the current
  behaviour (no edges) an intentional decision instead of a bug.
- Cons: `graph_bfs` (`src/sdk/graph.rs:50`) cannot traverse a metadata list — a
  multi-hop question ("which decisions descend from this episode?") becomes a
  namespace scan per hop, which is the thing MEMG-03 exists to avoid. It also
  deletes a shipped capability in the same breath as fixing it, forcing every
  consumer onto a different API. Rejected; the honest version of this option is
  "declare the feature dead", which is a different ADR.

### O5 — Teach `derived_from` to accept qualified parents, and let `put` own the edges

`MemoryRecord::derived_from` is persisted as the node field
`__vanta_derived_from` (`src/sdk/serialization/mod.rs:43`, read back at `:583`)
and reconciled by `put` for parents **in the record's own namespace**
(`memory_node_id(record.namespace, parent_key)`, `memory.rs:196-199,1261-1267`).
Extend the element syntax to `<ns>\0<key>` so metadata can express a parent in
another namespace, and drop `ensure_lineage_edge` entirely.

- Pros: **one** writer of `derived_from`, so no drop-and-recreate churn (D3
  disappears), and a re-put is idempotent by construction.
- Cons: it changes a persisted node field's semantics and the export/import line
  (`MemoryExportLine.derived_from`, `src/sdk/serialization/mod.rs:884,934`) —
  i.e. an on-disk format change needing a version bump and a migration path — and
  it collides with the `Asserted`/`Derived` validation, which currently requires
  `derived_from` parents to resolve **in the same namespace**
  (`memory.rs:437-444`) and forbids parents entirely on `Asserted` records
  (`:401-407`), which is what these records are (`l1_writer.rs:725-726`). That
  is a contract change in `src/` plus a data migration, for a bug whose fix is
  four lines. **Rejected for this decision; this is the declared upgrade path**
  if the churn in D3 ever costs more than it looks.

### O6 — A distinct label for cross-namespace provenance

Keep the projection right but label these edges `provenance` instead of
`derived_from`, so `put`'s reconciler (which only touches `derived_from` and
`superseded_by`) never deletes them.

- Pros: removes the churn of D3 immediately, with no `src/` change, and with it
  the duplicate-edge exposure that D7 has to patch.
- Cons: it forks the vocabulary — two labels for `MemoryRecord::derived_from`'s
  relation, where the only difference is that the parent lives in another
  namespace. No **in-tree** consumer filters on `derived_from` (`graph_bfs` takes
  no label, `src/sdk/graph.rs:50`; the filter lives in `graph_bfs_filtered`,
  `:89`, and the only in-tree caller passes user labels,
  `vantadb-mcp/src/handlers/tools.rs:3705-3708`), so the cost is not a broken
  consumer today — it is a contract leak: the label would encode a technical
  detail (which namespace the parent is in) instead of the relation, in the one
  place the SDK documents as the memory↔graph bridge. `LABEL_DERIVED_FROM` is
  the canonical name for child→parent (`src/sdk/api/graph.rs:20-27`). Rejected:
  with D7 removing the duplicate half of its advantage, O6 buys only the churn,
  and O5 buys that correctly. What is **not** measured: whether any consumer
  outside this repo filters on the label.

### O7 — A core API that addresses nodes by memory key

Add `add_memory_edge(from_ns, from_key, to_ns, to_key, label)` in `src/`, so the
ns+key → `u128` projection lives in the core exactly once.

- Pros: the projection has a single owner, in the crate that owns
  `memory_node_id`; `vanta-memory` cannot hash the wrong string because it never
  hashes at all.
- Cons: a new **public** API in `src/` for a bug that needs no `src/` change
  (Hyrum: every public behaviour becomes a contract), and it does not actually
  remove the risk — the defect was never in the hashing, it was in **which
  `(ns, key)` pair** was passed, and the caller still supplies that pair. It
  would also have to be a second spelling of `add_edge`/`ensure_edge`
  (`graph.rs:113,186`), which is a One-Version-Rule problem for no gain.
  Rejected.

## Decision outcome

**O1** — the endpoints are memory record nodes, addressed by
`memory_node_id(namespace, key)`, with the projection fixed per endpoint. The
label (`derived_from`), the direction (child → parent) and the guard all stay.

### D1 — The projection, per endpoint

| Endpoint | namespace | key |
|---|---|---|
| scene (L2) | `scene_namespace(session_key)` | `sanitize_key(&block.scene_name)` — the local `key` in `write_scene_block` |
| persona (L3) | `persona_namespace(session_key)` | the local `key` (= `sanitize_key(PERSONA_KEY)`) in `write_persona` |
| scoped persona | `profile_namespace(scope)` | the local `key` in `sync_persona_to_scope` |
| L1 record / L3 axiom | the write namespace `ns` | `sanitize_key(&record.id)` |
| episode (parent) | `l1_namespace(session_key)` | `sanitize_key(id)` |

The key column is not free: it is **the key `put` used**, which is what makes
the address the same one. `record_input` sanitizes (`l1_writer.rs:733`,
`key: sanitize_key(&record.id)`), so the parent ids — which are record ids —
must be sanitized too. A projection that uses the raw `record.id` is wrong for
any id containing a character outside the safe set, and it fails **silently**
(the guard skips it).

`session_key` is **not** available at the L1 site: `put_record_with_options`
takes `(db, ns, record, vector, confidence_override, dedup_policy_override)`
(`l1_writer.rs:767-774`). The session comes from `record.session_key`
(`MemoryRecord.session_key`, `core/abstractions/types.rs:108`). Passing
`l1_namespace(ns)` instead would double the prefix — `l1_namespace` is
`format!("l1/{}", …)` (`l1_reader.rs:25-27`) — and the guard would skip every
edge silently.

### D1b — The batch path is a sixth site, and it is the one production uses

`apply_dedup_batch` (`l1_writer.rs:457`, `pub`) is the group-commit entry point
(MEMG-11): it plans writes and calls **`db.put_batch`** (`:520`), never
`put_record_with_options`. `put_batch_inner` reconciles only the parents
declared in the node **field** (`memory.rs:1638-1705`), so a batch flush creates
**no** lineage edge. Callers: `services/pipeline_worker.rs:614`,
`record/l1_dedup.rs:236`, `approval.rs:137`.

A contract that covers only `put_record_with_options` therefore leaves every L1
record written by the pipeline unlinked while the tests pass — the exact drift
this decision exists to end. The fix is a post-batch loop over the planned
writes, in `vanta-memory` only: `apply_dedup_batch` already has `session_key`
(`:459`), `ns` (`:466`) and the planned records.

### D2 — Nothing creates nodes; `put` already did

No `insert_node` is added by this decision. The node exists because the record
exists, and the record is written **before** the edge at every one of the five
sites. That ordering is now a contract, not an accident — see D3.

### D3 — The edge must be created **after** the `put`, in the same function

`insert_one_internal` runs `reconcile_lineage_edges` (`memory.rs:1245-1251`),
which drops every forward `derived_from` edge the record's **node field**
(`__vanta_derived_from`) does not declare (`:202-231`), and cleans the
counterpart half (`:1258-1260`). On the primary path these records declare
**nothing** in that field — `record_input` builds `MemoryInput` with
`confidence_class: Asserted` and `derived_from: None` (`l1_writer.rs:719-730`),
and `Asserted` forbids parents (`memory.rs:401-407`) — because their provenance
lives in the **metadata** key `source_episode_ids` / `derived_from`, a different
key in a different structure (`scene_index.rs:208-217`,
`persona_generator.rs:452-461`).

The qualifier matters twice:

- With a `confidence_override: Some(Derived)` — the controlled re-persist paths
  (contradiction marking, dream promotion) — the field **is** populated, and a
  declared parent that no longer exists makes **`put` fail**, not skip: the
  ensure is `ensure_edge(...)?` (`memory.rs:1266`). O5 would inherit that
  landmine byte for byte, so it must make the ensure non-fatal before it can be
  used as the migration route.
- Those same records can then carry the **same** `derived_from` edge from two
  producers: `put` (from the field) and `ensure_lineage_edge` (from metadata).
  `add_edge` does not deduplicate (`graph.rs:132-138` pushes unconditionally),
  so a second creation is a duplicate edge — see D7.

So on every rewrite a scene/persona/L1 record **drops and recreates** its own
lineage edges. Creating the edge before the `put` would mean the `put` deletes
it. This is a declared, bounded cost (two engine writes per edge per rewrite,
writes only) and the reason D1's sites must keep the current order. O5 is the
way out if it ever stops being cheap.

### D4 — Both provenance fields are projected by the same rule

`source_episode_ids` and `source_message_ids` are merged into one list at
`l1_writer.rs:796-812` and must stay merged. `put_l3_axiom` sets
`source_message_ids = source_episode_ids` (`cascade_forget.rs:143`), so the two
fields are **not** disjoint by construction, and the implementation must not
branch on which field an id came from. Ids that are not records — raw
conversation turn ids — hit the FIND-747 guard and are skipped at `debug!`.
That is exactly what the guard is for; probing is cheaper than guessing.

### D5 — The fifth site is included, and D1b is the sixth

`sync_persona_to_scope` (`profile_sync.rs:148-173`) writes a record in
`profile/<scope>` and declares `source_episode_ids` + `derived_from` metadata
with **no edge at all**. Same defect, one site over. Including it now is what
stops the drift that produced this bug: a class of sites where four had a wrong
id and one had no edge. A fix that touches four and not the fifth re-creates the
condition — and D1b adds the sixth (`apply_dedup_batch`), which is the one the
pipeline actually calls. A fix covering 1–4 and not 5–6 passes every test that
looks at `put_record` and leaves production unlinked.

### D6 — `entity_node_id` survives, for the triplets only

The triplet path (`ontology/mod.rs:216-238`) materializes its nodes before
linking them, so its FNV space is self-consistent. `entity_node_id` stays
`pub`, unchanged. It is not deprecated: deleting a used `pub fn` is a breaking
change to `vanta-memory`'s surface for no gain.

### D7 — The guard keeps its precondition, logging and return value, and gains idempotency

`ensure_lineage_edge` keeps the endpoint precondition, the `debug!` with
`missing_node`, and the `false` return. After this change it stops firing for
record-backed endpoints and keeps firing for genuinely absent ones — an episode
removed by a **Hard** forget (D9), a raw turn id.

It is also allowed — and required — to stop duplicating edges. It already holds
both `NodeRecord`s (`ontology/mod.rs:269-295`), so the check is free: skip (or
`false`) when a forward `derived_from` edge to that target is already present.
Without it, the two producers named in D3 (node field + metadata) create the same
edge twice, and a `put` that returns early on dedup — no reconcile, no drop —
leaves `add_edge` appending a fresh copy on every pass. `add_edge` itself does
not deduplicate (`graph.rs:132-138,144-150`), and D6 forbids changing it.

### D9 — `Soft` forget does not remove a node; the chaos case is `Hard`

`ForgetMode::Soft` re-reads the record, stamps `invalid_at_ms` and **reinserts
the same node** (`memory.rs:2081-2106`), so `get_node` still returns `Some`,
the guard passes and the edge **is** created. Only `Hard`
(`:2108-2114` → `delete` → KV entry removed) produces `None`. The
"episode absent" case is therefore a **Hard** forget, and it must be tested as
one; `cascade_forget` uses `Soft` for L1 (`cascade_forget.rs:369`), so the
absence case is rarer than the FIND-747 comment implies.

### D8 — No `src/` change, no re-index, no migration of edges

- `memory_node_id` is already exported (`src/sdk/mod.rs:32-36`), so the fix is
  entirely inside `vanta-memory`. The backlog row's "cruza a `src/`" is wrong.
- Record node ids do not change, so no HNSW rebuild and no text-index rebuild.
- There is nothing to migrate: no edge was ever created under the phantom ids
  (`entity_node_id` has no producer outside the triplet path), so the old edges
  are **absent**, not wrong.
- What is **not** free: records written before the fix get their lineage edges
  only when they are rewritten. There is no backfill pass, and its cost is
  **not measured** here (see §Measurement and its limits). This is the one
  conscious debt the decision leaves, and it is the row this ADR opens.

## Positive consequences

- The lineage in the description of MEMG-03 becomes true: a scene or persona
  pass creates traversable `derived_from` edges, and `graph_bfs` answers "what
  descends from this episode?" without a namespace scan.
- `FIND-296`'s defect (L2/L3 without linkage) is closed for every record-backed
  entity. See §Relation to FIND-296 for what it does **not** close.
- One address per entity, forever: the same `memory_node_id(ns,key)` the SDK
  documents, the export/import uses, and `carry_graph_state` preserves.
- The five wrong id constructions disappear, so the next agent reading
  `scene_index.rs` sees a namespace and a key — not a string label with colons
  in it that looks like an id.

## Negative consequences

- **Drop-and-recreate churn on every rewrite** (D3): a rewritten scene deletes
  its forward edges and their counterparts, then recreates them. Correct end
  state, more writes than a reconciler-based design would need.
- **`derived_from` has two writers** with two projections: `put` (same
  namespace, from the node field) and `vanta-memory` (cross-namespace, from
  metadata). It works because of the ordering in D3, and it is the debt O5
  removes. `.opencode/rules/api-contract.md` R-8 ("one owner per fact") is
  bent here, deliberately and in writing.
- **The graph changes size.** A database that has been through many L2/L3
  passes gains edges it never had. Anything asserting "no `derived_from` edges
  from scenes" breaks. The two guard tests in `ontology/mod.rs:340-354` and
  `:358-382` are **not** among them — they use ids that are still not nodes and
  keep passing unchanged; what must be rewritten are the "Guarded: … declared
  skip" comments at `scene_index.rs:229-232`,
  `persona_generator.rs:473-475`, `l1_writer.rs:789-791`, which stop describing
  the normal path.
- **`Soft` forget keeps the node** (D9), so the absence path is narrower than
  FIND-747's comment suggests: an invalidated episode is still a valid edge
  target. The edge will point at a record that recall no longer returns — the
  metadata and the graph disagree only in *visibility*, not in existence.
- **`add_edge` is a read-modify-write** (`graph.rs:129-151`: `get`, mutate,
  `insert`, twice) with no lock. Until now no lineage edge existed, so the lost
  update this permits was dormant; this decision makes it live. Whether two
  passes can run concurrently in one process is **not measured** (one
  `tokio::spawn` in `services/scheduler.rs:314` suggests a single loop, and the
  `.vanta.lock` file lock keeps one writer per database, but ADR-0065's
  on-demand path can call a pass outside the loop). It is opened as the second
  proposed row in [`FIND-756`](../../tasks/FIND-756.md), pending an ID from the
  lead, because this task may not edit the Backlog (`docs/dev/Backlog.md` is
  machine-appended by the task system).
- **`add_edge` is a read-modify-write** (`graph.rs:129-151`: `get`, mutate,
  `insert`, twice) with no lock. Until now no lineage edge existed, so the lost
  update this permits was dormant; this decision makes it live. Whether two
  passes can run concurrently in one process is **not measured** here (one
  `tokio::spawn` in `services/scheduler.rs:314` suggests a single loop, and the
  `.vanta.lock` file lock keeps one writer per database, but the on-demand path
  of ADR-0065 can call a pass outside the loop). It is a candidate for its own
  finding; opening it is out of this task's scope.
- **Historical records are not retroactively linked** (D8), with no backfill
  and no measured cost for one.

## Measurement and its limits

Measured by reading the tree at `HEAD` (2026-11-26), not by running:

| Claim | Source |
|---|---|
| `add_edge` requires both endpoints and does not deduplicate | `src/sdk/api/graph.rs:129-131,132-138,141-143,144-150` |
| `memory_node_id = XxHash3_128(ns ++ \0 ++ key)` | `src/sdk/serialization/mod.rs:113-124` |
| `memory_node_id` is exported to sibling crates | `src/sdk/mod.rs:32-36` |
| Every record is a node at that address | `src/sdk/api/memory.rs:1143` (id), `:1234` (projection), `:1254` (insert) |
| `put_batch` does the same | `memory.rs:1560` |
| `put` reconciles `derived_from` against the node field only, and **errors** on a declared-but-absent parent | `memory.rs:196-199,1245-1267` (`ensure_edge(...)?` at `:1266`); field `__vanta_derived_from` at `serialization/mod.rs:43,583` |
| The writers declare provenance in metadata, not the field | `scene_index.rs:208-217`; `persona_generator.rs:452-461`; `profile_sync.rs:154-163` |
| Primary-path L1 records are `Asserted` (field empty) | `l1_writer.rs:719-730`; `memory.rs:401-407` |
| L1 record key is sanitized | `l1_writer.rs:733` |
| An episode **is** an L1 record | `l1_writer.rs:733` (key); `cascade_forget.rs:180,193` (`ForgetTarget::Key(episode_id)` on `l1_namespace`) |
| `source_message_ids` and `source_episode_ids` are conflated | `cascade_forget.rs:143` |
| The L1 **batch** path bypasses the per-record writer | `apply_dedup_batch` `l1_writer.rs:457,520`; `put_batch_inner` reconciles fields only `memory.rs:1638-1705` |
| `Soft` forget reinserts the node; only `Hard` removes it | `memory.rs:2081-2106` vs `:2108-2114` |
| An L0 turn **is** a record | `l0_recorder.rs:363`; `l0_namespace` private at `:556` |
| `graph_bfs` takes no label; filtering lives in `graph_bfs_filtered` | `src/sdk/graph.rs:50,89` |
| The write sites, each `put` then edge | `scene_index.rs:219,239`; `persona_generator.rs:463,482,493`; `l1_writer.rs:781,816`; `profile_sync.rs:164` (no edge) |
| `entity_node_id` has 0 hits outside `vanta-memory` | grep over `*.rs`, 25 matches, all under `vanta-memory/` |
| The only production `insert_node` in the crate are triplet entities | `ontology/mod.rs:221,230` |

**Not measured:**

- **The cost of a backfill.** No pass exists and none was written, timed, or
  estimated. If one is wanted, it needs its own finding.
- **Whether two passes can race on `add_edge` in one process.** Stated as
  unknown above rather than guessed.
- **The throughput effect of the edges themselves.** Not benchmarked. Structurally
  the write path already attempted one `get_node` per endpoint (the guard) and
  now additionally performs the two inserts `add_edge` always did when it
  succeeded; on the paths where edges are created this is *more* work than
  today, not less. No percentage is claimed.
- **Whether any consumer outside this repo filters on the `derived_from` label**
  (O6). No in-tree one does; an external one cannot be checked from here.

## Verification

The mechanical contract — exact assertions, the file to create, the command —
lives in [`FIND-756`](../../tasks/FIND-756.md). At the ADR level:

- After an L2 scene pass over two L1 records, the scene's node carries a forward
  `derived_from` edge whose target is the **L1 record's** node id, and that
  record's node carries the reverse half. Read from the graph, not the log.
- The same for a persona (to episodes and to scenes), for an L1 record (to its
  source episodes), and for the scoped persona.
- The phantom ids return `None` from `get_node` — they were never materialized
  and the fix does not materialize them.
- A parent that is not a record (raw turn id, or an episode removed by
  `cascade_forget`) is skipped by the guard and produces no edge and no error.
- `cargo check -p vanta-memory` shows **no** diff outside `vanta-memory/`.

## Relation to FIND-296

`FIND-296` is "L2/L3 sin linkage" (`docs/dev/Backlog.md:801`). This decision
closes **that defect**: scenes and personas are L2/L3, and after it they are
linked. It does not close `MEMG-03` (the P1 row that owns the theme), because two
gaps remain outside its statement:

- **Batch-flushed L1 records stay unlinked** unless D1b is implemented — and D1b
  is the site the pipeline actually uses. This is the largest gap and it is
  inside this decision's own scope precisely because of that.
- **Message-level provenance** is deferred, not impossible: an L0 turn **is** a
  record (`l0_recorder.rs:363`, namespace `l0/<session>`), so the same rule
  addresses it — `memory_node_id(l0_namespace(session), sanitize_key(id))`.
  `l0_namespace` is private (`:556`), so wiring it means exporting it first.
  Deferred for scope and volume, not for lack of an address.
- **`sync_persona_to_scope`'s scoped persona** is linked by D5, but the
  *scoped* layer has no lineage query of its own yet.

Whoever closes the row must cite the graph assertions above, not the absence of
`Node not found` in a log: the log was already silent, by design, since
`FIND-747`.

## References

- [FIND-756](../../tasks/FIND-756.md) — the implementation contract.
- [`FIND-740-747`](../../tasks/FIND-740-747.md) — where `FIND-747` (the guard, the
  measured four facts, and the `FIND-296` note) lives; it has no task file of its
  own.
- `docs/dev/Backlog.md:801` — the `FIND-756` row (its `cruza a src/` and
  `ADR-0054` claims are corrected here, D8).
- `docs/dev/Backlog.md:60` — `MEMG-03`, which owns the theme and cites
  `FIND-296`.
- [vanta-memory.md § MEMG-03](../../avance/activo/vanta-memory.md) — the bridge
  this ADR makes traversable; `:202` is where `FIND-296` was prematurely
  declared folded.
- `src/sdk/api/graph.rs:20-27` — `LABEL_DERIVED_FROM` and the canonical
  direction (child → parent), which this ADR does not change.
- `src/sdk/serialization/mod.rs:113-124` — `memory_node_id`, the address chosen.
- [ADR-0020](./ADR-0020-storage-backend-default.md) — the storage default this
  change leaves untouched (no re-index, D8).
- `.opencode/rules/api-contract.md` R-8 (one owner per fact) — the rule D3
  knowingly bends, under O5 as the way back.
