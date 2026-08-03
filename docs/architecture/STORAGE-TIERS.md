# Storage Tiers (hot / warm / cold / archive)

## Purpose

Vector segments in VantaDB are stored as level-mapped `VantaFile` (mmap) blocks,
one per LSM level. This document defines the **tier policy**: what drives a node
(and the segment it lives in) from hot → warm → cold → archive, and how that
maps onto the existing `SegmentLevel` enum (`L0`–`L3`) and the maintenance
pipeline.

> Tiering applies to the **vector segments** (`vstore_L*.vanta`) only.
> KV backends (Fjall, RocksDB) implement their own LSM at the KV level and are
> out of scope.

## Level mapping

| SegmentLevel | Tier     | Placement      | `active` (mmap resident) | Config knob                |
|--------------|----------|----------------|--------------------------|----------------------------|
| `L0`         | hot      | in-memory/mmap | yes                      | `l0_max_size` (64 MiB)    |
| `L1`         | warm     | in-memory/mmap | yes                      | `l1_max_size` (512 MiB)   |
| `L2`         | cold     | on disk        | no                       | `l2_max_size` (4 GiB)     |
| `L3`         | archive  | on disk        | no (optional)            | `l3_max_size` (32 GiB)    |

File names are `vstore_L0.vanta` … `vstore_L3.vanta`. `SegmentRegistry::open_or_create`
pre-allocates all four levels up front, so a segment crossing a tier never needs
dynamic `vector_store` growth (no `unsafe`). The packed offset uses a 6-bit
`segment_id` (`0x3F` mask) — four levels fit with 60 ids to spare; its semantics
must never change.

## Promotion criterion

A level is selected for compaction by `should_compact_level`, which evaluates a
`TierPolicy`. The promotion happens by **segment threshold + tombstone ratio**.

| TierPolicy        | Rule                                                                            | Status            |
|-------------------|---------------------------------------------------------------------------------|-------------------|
| `SizeBased`       | promote when `write_cursor >= lX_max_size` or tombstone ratio >= threshold      | implemented (default) |
| `FrequencyBased`  | promote when resident nodes fall below `cold_min_frequency` accesses/window     | config only       |
| `AgeBased`        | promote when resident nodes idle longer than `cold_age_days`                    | config only       |

`SizeBased` is the default and is what `compact_level`/`run_pipeline` execute.
`FrequencyBased` and `AgeBased` are exposed in `TierPolicyConfig` as a nominated
heuristic for a future per-node access tracker; they do not change behavior
until such a tracker exists. `TierPolicyConfig.archive` toggles whether the `L3`
archive level participates (default `true`).

## Chained promotion

`compact_level(level)` reads live (non-tombstone) nodes from `level`, rewrites
them to `level + 1` via `write_node_to_vstore`, updates each node's HNSW
`storage_offset` to the new packed `(segment_id, local_offset)`, then truncates
the source. `run_pipeline` chains the levels in order:

```
L0 (hot) --size/tombstone--> L1 (warm) --size/tombstone--> L2 (cold) --size/tombstone--> L3 (archive)
```

1. Insert writes the node into `L0`.
2. When `L0` crosses `l0_max_size`, `compact_level(0)` promotes live nodes to `L1`.
3. When `L1` crosses `l1_max_size`, `compact_level(1)` promotes to `L2`.
4. When `L2` crosses `l2_max_size`, `compact_level(2)` promotes to `L3`
   (only if `TierPolicyConfig.archive` is enabled; if not, `L2` is the deepest
   tier and `should_compact_level` never selects `L3`).

Promoted nodes keep their id and stay **queryable**: reads derive the segment
from the packed offset, and the HNSW node map is updated before the source level
is truncated.

## Defaults

`LsmConfig::default()` (in `src/lsm.rs`):

| Knob                   | Default |
|------------------------|---------|
| `l0_max_size`          | 64 MiB  |
| `l1_max_size`          | 512 MiB |
| `l2_max_size`          | 4 GiB   |
| `l3_max_size`          | 32 GiB  |
| `l0_tombstone_threshold` | 0.20  |
| `l1_tombstone_threshold` | 0.15  |
| `l2_tombstone_threshold` | 0.10  |
| `l3_tombstone_threshold` | 0.05  |
| `min_segment_size`     | 64 KiB  |
| `tier.policy`          | `SizeBased` |
| `tier.archive`         | `true`   |

Existing deployments keep `vstore_L0/L1/L2.vanta`; only the new `L3` archive
level gains a threshold. No public API, on-disk file, or packed-offset semantics
change.