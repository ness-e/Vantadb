---
description: Simplify code for clarity and maintainability — reduce complexity without changing behavior
---

Invoke the agent-skills:code-simplification skill.

Simplify recently changed code (or the specified scope) while preserving exact behavior:

1. Read CLAUDE.md and study project conventions
2. Identify the target code — recent changes unless a broader scope is specified
3. Understand the code's purpose, callers, edge cases, and test coverage before touching it
4. Scan for simplification opportunities:
   - Deep nesting → guard clauses or extracted helpers
   - Long functions → split by responsibility
   - Nested ternaries → if/else or switch
   - Generic names → descriptive names
   - Duplicated logic → shared functions
   - Dead code → remove after confirming
5. Apply each simplification incrementally — run tests after each change
6. Verify all tests pass, the build succeeds, and the diff is clean

**What NOT to simplify:**
- Performance-critical hot paths (let the compiler/benchmark drive decisions)
- Security-sensitive validation (redundancy is defense-in-depth)
- Public API signatures (breaking changes need deprecation cycles)
- FFI bindings or `unsafe` blocks (change one, audit them all)
- Generated code (re-generate instead)

If tests fail after a simplification, revert that change and reconsider. Use `code-review-and-quality` to review the result.

**Upstream:** `/build` (genera el código a simplificar). Se recomienda correr `/build prove` si hay bugs post-simplificación.
