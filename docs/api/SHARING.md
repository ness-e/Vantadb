---
title: "Shared memory: scopes, grants, and revocation"
kind: reference
status: active
description: "Multi-agent sharing over the existing permission checker: scope mapping, grants, revocation semantics, checked reads and writes, and declared boundaries"
tags: [vantadb, api, sharing, permissions]
---

# Shared memory: scopes, grants, and revocation

> **Source:** `src/sdk/api/sharing.rs` (MEMG-16). Built entirely on the
> allow-only permission chain in `src/entity/checker.rs` and the entity store
> in `src/entity/mod.rs` — no logic is duplicated and no new storage
> mechanism is introduced.

Sharing lets a record written by one principal be read/written by another
under explicit grants, and lets those grants be revoked with effect on the
**next access**. The model is: one `asset` entity per shared record, one
`team_member` entity per membership, one `acl` entity per grant.

## Scope model (declared mapping)

| Scope | Maps to | Enforced by |
|-------|---------|-------------|
| **org** | namespace (tenant root) | namespace isolation (MEMG-04 RBAC on the server; the SDK treats namespaces as the top boundary) |
| **project** | namespace sub-path `{org}/{project}` (e.g. `acme/atlas`) | same namespace boundary — projects are namespaces by convention |
| **team** | `team_member` entities (`{team_id}.{user_id}`, role + status) | membership step + role defaults of the checker |
| **asset** | `asset` entity (id = record key) + `acl` grants | visibility step + ACL step of the checker |

All sharing entities live in the standard entity partition
(`entity:{namespace}:{collection}::{id}`, `InternalMetadata`), scoped to the
namespace of the asset.

## Sharing entities

| Collection | Id | Fields | Meaning |
|------------|----|--------|---------|
| `asset` | record key | `team_id`, `owner_user_id`, `visibility` (`private`/`team`/`restricted`/`agent`/`task`), `status` | the shareable record; the owner always short-circuits the chain |
| `team_member` | `{team_id}.{user_id}` | `role` (`admin`/`member`/`reviewer`), `status` (`active`/`removed`) | team membership (role defaults: admin → read/write/assign/share; member/reviewer → read) |
| `acl` | `{asset_id}.{subject_type}.{subject_id}.{action}` | `effect` (`allow`), `permission`, `subject_type` (`user`/`team_role`/`agent`), `subject_id` | explicit grant |

## Grants

```rust
use vantadb::sdk::{Action, Subject, TeamRole, Visibility};
use vantadb::sdk::{GrantInput, ShareAssetInput, TeamMemberInput};

// 1. Make a record shareable (asset metadata; the record itself is untouched).
db.share_asset(ShareAssetInput {
    namespace: "acme/atlas",
    asset_id: "doc-1",              // = memory record key
    owner_user_id: "usr-alice",
    team_id: "core",
    visibility: Visibility::Team,
})?;

// 2. Membership (team-level access via role defaults).
db.add_team_member(TeamMemberInput {
    namespace: "acme/atlas",
    team_id: "core",
    user_id: "usr-bob",
    role: TeamRole::Member,
})?;

// 3. Explicit grant (required for actions not covered by the role default,
//    and for any non-admin access to `restricted` assets).
db.grant_access(GrantInput {
    namespace: "acme/atlas",
    asset_id: "doc-1",
    subject: Subject::user("usr-bob"),   // or Subject::team_role(..) / Subject::agent("agt-9")
    action: Action::Write,
})?;
```

**Subjects:** `user` (resolved principal), `team_role` (all members of a
role), `agent` (matches only when the caller supplies the matching
`agent_id` at check time). **Actions:** `read`, `write`, `assign`, `share`,
`use`.

## Decision chain (reused, unchanged)

`check_access` evaluates the existing allow-only chain, in order: asset
present and not archived → owner → active membership → visibility
(`private` denies non-owners; `restricted` requires ACL for non-admins;
`task` is read-only for non-admins) → role default → ACL allow → deny. A
missing asset, an inactive membership, or the absence of any allow rule all
**deny** (fail-closed). Grants never bypass membership.

## Checked operations

| Operation | Action checked | Result |
|-----------|----------------|--------|
| `check_access` | caller-chosen | `PermDecision { allowed, reason }` — decision as data; surfaces map it (e.g. HTTP 403) |
| `get_shared` | `Read` | `SharedOutcome { decision, record }` — `record: None` on deny |
| `put_shared` | `Write` | `SharedOutcome { decision, record }` — nothing is written on deny |

A deny never returns the record and never writes, so it cannot be used as an
existence oracle. `use`/`recall` consumers evaluate `check_access` with
`Action::Use` / `Action::Read` at their own boundary (see Boundaries).

> `get_shared` can also return `allowed = true` with `record: None` when the
> asset is valid but the record itself does not exist (e.g. deleted after
> sharing) — `None` alone does not distinguish "denied" from "absent"; read
> `decision.allowed` for the access outcome.

## Revocation semantics

**Revocation is future-access only — it is not a purge.**

- `revoke_access` deletes the ACL entity; the checker reads live entities on
  every call (no cache), so the **next access** is denied.
- `revoke_team_member` flips the membership `status` to `removed` (the entity
  is kept for audit); the next access is denied `not_team_member`.
- Re-granting (same `grant_access` / `add_team_member`) re-enables access.

Data already delivered or copied is **not** un-shared by revocation. Erasing
shared content is the erasure path (MEMG-17: DEK destruction + tombstone +
verifiable receipt), a different operation with its own contract — a
revocation only stops *future* access through this database.

## Propagation and containment

- **What propagates:** access (a grant lets another principal reach the same
  record in place). Sharing never copies records between namespaces or teams,
  so there is no divergent copy to reconcile.
- **Containment:** revoking membership and grants contains a misbehaving
  agent immediately (next-access semantics). This is the minimum containment
  this task declares; the full multi-agent error cascade
  (`EXE-07`, quarantine/dream isolation) is tracked in `FIND-305`.

## Boundaries (declared)

- **Erasure (MEMG-17):** purge of already-shared data is out of scope here —
  use the erasure surface; revocation alone does not erase.
- **Quarantine (SCH-05 / MEMG-10):** quarantined records remain excluded from
  default retrieval regardless of grants; sharing does not promote or
  un-quarantine content.
- **Surface exposure:** this is the SDK surface. The HTTP server already
  resolves a principal (`AuthIdentity`) and declares its intent to authorize
  with `PermissionChecker`; wiring `check_access` into the record/search
  handlers (and gating the management methods behind its RBAC) is
  `FIND-305`. The management methods carry the same trust level as the rest
  of the embedded SDK — the embedding application decides who may call them.

## Limits (declared)

- Ids composed into sharing keys (`asset_id`, `team_id`, `user_id`,
  `owner_user_id`, `subject_id`, `agent_id`) must be non-empty and must not
  contain `.`, `{`, `}` or `:` — the checker composes
  `{team_id}.{user_id}` and `{asset_id}.{subject_type}.{subject_id}.{action}`,
  so a `.` inside a component would let distinct grants/memberships collide.
  Record keys outside this charset cannot be shared in v1 (an encoding scheme
  is the upgrade path if needed).
- One asset per record key; project-level sharing is expressed by the
  namespace convention (no `project` ACL subject).
- `share_asset` is an upsert that always writes `status = active`: re-sharing
  an archived asset re-activates it (explicit call, never implicit).
- Checked operations cover `get`/`put`; other operations (`search`, `list`,
  `delete`) evaluate `check_access` at the caller boundary.

## Related

- [Embedded SDK Reference](./EMBEDDED_SDK.md) — the `Embedded` surface these methods extend.
- [HTTP API](./HTTP_API.md) — the server surface (multi-tenant RBAC).
- [VANTA_MEMORY](./VANTA_MEMORY.md) — the memory layer built on the SDK.
- [Certified delete](./CERTIFIED_DELETE.md) — the purge/attestation path (contrast with revocation).
