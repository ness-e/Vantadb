# Adapter Restructure Implementation Plan

> **Goal:** Eliminate Rust/Python duplication in adapters. Framework adapters → Python only (in `integrations/`). Provider adapters → Rust only (in `providers/`).
> **Estado:** ✅ COMPLETED (2026-07-22, commit accbfa8)
> **ADR:** docs/architecture/adr/010_adapter_language_classification.md

**Architecture Decision:**
- **Framework adapters** (LangChain, LlamaIndex, Haystack, CrewAI, DSPy, Letta, Mem0) MUST be Python because their integration APIs are Python abstract base classes/protocols that can't be implemented from Rust PyO3.
- **Provider adapters** (OpenAI, Ollama, LiteLLM) are REST APIs callable from any language → Rust is correct.
- 7 of 9 Python wrappers lack real vector search (keyword match only) — fixing those is Phase 2.

**What changes:**
```
ANTES:
  vantadb-crewai/       ← Rust (wrong language for framework adapter)  → DELETE
  vantadb-dspy/         ← Rust (wrong language)                        → DELETE
  vantadb-haystack/     ← Rust (wrong language)                        → DELETE
  vantadb-langchain/    ← Rust (wrong language)                        → DELETE
  vantadb-letta/        ← Rust (wrong language)                        → DELETE
  vantadb-llamaindex/   ← Rust (wrong language)                        → DELETE
  vantadb-mem0/         ← Rust (wrong language, 420 LOC!)              → DELETE
  vantadb-openai/       ← Rust (correct language)                      → MOVE to providers/
  vantadb-ollama/       ← Rust (correct language)                      → MOVE to providers/
  vantadb-litellm/      ← Rust (correct language)                      → MOVE to providers/
  integrations/crewai/  ← Python (correct)                             → KEEP
  integrations/dspy/    ← Python (correct)                             → KEEP
  ...

DESPUÉS:
  providers/openai/     ← Rust (HTTP calls to REST API)
  providers/ollama/     ← Rust (HTTP calls to REST API)
  providers/litellm/    ← Rust (HTTP calls to REST API)
  integrations/...      ← Python wrappers (all that remain)
```

---

## Task List

### Phase 1: Create providers/ and move Rust crates

#### Task 1: Create providers/ directory + move openai, ollama, litellm
- [x] Create `providers/` dir
- [x] `git mv vantadb-openai providers/openai`
- [x] `git mv vantadb-ollama providers/ollama`
- [x] `git mv vantadb-litellm providers/litellm`
- [x] Update Cargo.toml workspace members
- **Verification:** `cargo check -p vantadb-openai` ✅

#### Task 2: Delete 7 framework Rust crates from root
- [x] Remove from workspace members in Cargo.toml
- [x] `git rm -r vantadb-crewai vantadb-dspy vantadb-haystack vantadb-langchain vantadb-letta vantadb-llamaindex vantadb-mem0`
- **Verification:** `cargo check` workspace builds ✅

#### Task 3: Update CI workflows
- [x] Check if any CI references old paths
- **Verification:** grep for old paths in .github/ ✅

### Phase 2: Document

#### Task 4: Create ADR for the restructuring
- [x] Write `docs/architecture/adr/010_adapter_language_classification.md`
- **Verification:** ADR-010 existe con contenido correcto ✅

### Phase 3: Verify

#### Task 5: Full verification
- [x] `cargo check` — must pass ✅
- [x] `cargo build` — must build ✅
- [x] `git status` — clean state ✅
