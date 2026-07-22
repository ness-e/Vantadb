# Adapter Restructure Implementation Plan

> **Goal:** Eliminate Rust/Python duplication in adapters. Framework adapters → Python only (in `integrations/`). Provider adapters → Rust only (in `providers/`).

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
- [ ] Create `providers/` dir
- [ ] `git mv vantadb-openai providers/openai`
- [ ] `git mv vantadb-ollama providers/ollama`
- [ ] `git mv vantadb-litellm providers/litellm`
- [ ] Update Cargo.toml workspace members
- **Verification:** `cargo check -p vantadb-openai`

#### Task 2: Delete 7 framework Rust crates from root
- [ ] Remove from workspace members in Cargo.toml
- [ ] `git rm -r vantadb-crewai vantadb-dspy vantadb-haystack vantadb-langchain vantadb-letta vantadb-llamaindex vantadb-mem0`
- **Verification:** `cargo check` workspace builds

#### Task 3: Update CI workflows
- [ ] Check if any CI references old paths
- **Verification:** grep for old paths in .github/

### Phase 2: Document

#### Task 4: Create ADR for the restructuring
- [ ] Write `docs/architecture/adr/adr-009-adapter-language-classification.md`
- **Verification:** File exists with correct content

### Phase 3: Verify

#### Task 5: Full verification
- [ ] `cargo check` — must pass
- [ ] `cargo build` — must build
- [ ] `git status` — clean state
