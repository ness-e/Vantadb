# Public Issue Drafts

These are draft public issues for post-`v0.1.1` external validation. They are
not yet published to GitHub.

## 1. Verify TestPyPI Install Flow Across Supported OSes

Labels: `packaging`, `python`, `triage`

### Problem

The Python wheel workflow builds and smoke-installs wheels, but the TestPyPI
install path still needs a clean, documented verification pass across Linux,
macOS, and Windows.

### Acceptance Criteria

- TestPyPI upload is triggered manually from the `Python Wheels` workflow.
- A clean virtual environment installs `vantadb-py` from TestPyPI on each OS.
- `python -m pytest vantadb-python/tests/test_sdk.py -v` passes against the
  installed wheel.
- Any platform-specific dependency or compiler requirement is documented.

## 2. Validate the 5-Minute Quickstart From a Clean Clone

Labels: `docs`, `good first issue`, `triage`

### Problem

The quickstart should be proven by someone starting from a clean checkout, not
from an already configured development machine.

### Acceptance Criteria

- Follow `docs/QUICKSTART.md` from a clean clone.
- Confirm CLI `put/get/list/export/audit-index` commands work as written.
- Confirm Python vector, text, and hybrid search examples run as written.
- Report any missing prerequisite, confusing wording, or platform-specific
  adjustment.

## 3. Define Search Quality v2 Scope

Labels: `search`, `roadmap`, `triage`

### Problem

Hybrid Retrieval v1 is intentionally conservative. Snippets, highlighting,
stable ranking explanations, tokenizer evolution, and ranking improvements
need a scoped design before implementation.

### Acceptance Criteria

- Define which outputs are public SDK/CLI API and which remain debug-only.
- Decide whether snippets/highlighting ship before tokenizer changes.
- Document non-goals, including competitive hybrid-search parity claims.
- Propose a small validation corpus for regression tests.

## 4. Define External Benchmark Validation Matrix

Labels: `benchmarks`, `validation`, `triage`

### Problem

Internal certification is useful, but external users need reproducible
benchmark instructions and honest interpretation.

### Acceptance Criteria

- Define benchmark datasets and hardware profile assumptions.
- Separate HNSW recall, BM25 text search, and hybrid retrieval measurements.
- Document what metrics are release evidence and what metrics are exploratory.
- Keep competitive claims out of scope until reproducible evidence exists.

## 5. Harden Backup and Restore Policy

Labels: `storage`, `reliability`, `docs`

### Problem

JSONL export/import is logical data movement, not a transactional physical
backup. Restore expectations need to be clearer for external operators.

### Acceptance Criteria

- Document supported restore paths for Fjall and RocksDB separately.
- Clarify when the database must be closed for file-level copies.
- Validate restore with canonical records, text search, phrase filters, and
  hybrid retrieval.
- Keep snapshot/checksum policy deferred until explicitly designed.

## 6. Define Python Distribution Policy

Labels: `python`, `packaging`, `release`

### Problem

The Python package is prepared for wheel validation, but production PyPI,
signing, and installer support are still intentionally deferred.

### Acceptance Criteria

- Define supported Python versions and OS wheel targets.
- Decide what must pass before production PyPI publication.
- Document signing and provenance expectations.
- Keep source install as the documented default until policy is met.

## 7. Improve Namespace-Scoped Memory Examples

Labels: `good first issue`, `docs`, `examples`

### Problem

Namespace-scoped memory is the core MVP concept, but examples can be expanded
without adding new features.

### Acceptance Criteria

- Add or improve examples for `put/get/list/search` with namespaces.
- Include metadata filters and text-only search.
- Avoid IQL, MCP, LLM, graph database, and enterprise claims.
- Link examples from the README or quickstart where appropriate.
