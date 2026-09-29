---
title: Deprecations Registry
kind: reference
status: active
description: Concrete deprecations of the public API surfaces and their removal windows
tags: [vantadb, api, deprecation]
---

# Deprecations Registry

Concrete deprecations of the public API surfaces and their removal windows.
The policy itself lives in [`VERSIONING.md` § Deprecation policy](VERSIONING.md#deprecation-policy) —
this page registers instances, it does not restate the policy:

- A deprecated API stays functional for **at least one MINOR release**.
- Removal ships in a later MINOR with a `feat!:` / `BREAKING CHANGE:` marker.

## Active deprecations

**None.** As of v0.7.0 (updated 2026-09-27) no public API item is deprecated on
any of the [11 surfaces](VERSIONING.md#the-11-public-api-surfaces).

## Entry shape (fill for every new deprecation)

### Template — `old_name` → `new_name`

- **Surface:** <number + name from the 11-surface table>
- **Deprecated in:** v0.X.0 (YYYY-MM-DD)
- **Removal target:** v0.Y.0 — ≥ 1 MINOR after deprecation (earliest possible)
- **Replacement / migration:** <what consumers should do instead>
- **Changelog:** [`docs/CHANGELOG.md`](../CHANGELOG.md) § [0.X.0]
- **Code/doc marker:** <deprecation attribute / doc note / release-note marker>

## Process

1. **Deprecate** — in the same PR that marks the item (attribute, doc note or
   reference-doc note): add the changelog entry, update the affected surface
   doc, and fill an entry above.
2. **Remove** — at or after the removal target, in a MINOR release with a
   `feat!:` / `BREAKING CHANGE:` marker; delete the entry above.
3. **Mechanical line** — the rails in [`COMPATIBILITY.md`](COMPATIBILITY.md)
   (semver-checks + public API snapshot) fail any removal that skips the
   marker/registers, so this registry cannot silently rot.
