# USAGE — Review Deep: Cómo ejecutar

## Comando único (un módulo)

```bash
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-sdk" "DEPTH=full"
```

Para revisión rápida (sin web research ni competitor):

```bash
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-sdk" "DEPTH=quick"
```

## Todos los módulos en orden (Wave por Wave)

Los módulos se ejecutan en 7 waves, de mayor a menor impacto:

```bash
# Wave 0 — Core crítico
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-sdk" "DEPTH=full"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-engine" "DEPTH=full"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-wal" "DEPTH=full"

# Wave 1 — Indexación
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-vector" "DEPTH=full"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-index" "DEPTH=full"

# Wave 2 — Gobernanza
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-governance" "DEPTH=full"

# Wave 3 — SDKs
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-python" "DEPTH=full"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-ts" "DEPTH=full"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-wasm" "DEPTH=full"

# Wave 4 — Infra
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-server" "DEPTH=full"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-mcp" "DEPTH=full"

# Wave 5 — Adaptadores (paralelizable con FAIL_MODE=parallel)
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-openai" "DEPTH=quick"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-ollama" "DEPTH=quick"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-litellm" "DEPTH=quick"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-mem0" "DEPTH=quick"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-letta" "DEPTH=quick"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-crewai" "DEPTH=quick"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-dspy" "DEPTH=quick"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-haystack" "DEPTH=quick"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-langchain" "DEPTH=quick"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-llamaindex" "DEPTH=quick"

# Wave 6 — Utils
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-crypto" "DEPTH=full"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-cli" "DEPTH=full"
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=vantadb-enterprise" "DEPTH=quick"
```

## Batch mode para una wave completa

```bash
/loop-goal --prompt-file .opencode/skills/task-executor/batch-prompt.md \
  "PLAN_FILE=docs/plans/review-deep-wave0.md" "FAIL_MODE=stop"
```

Primero creá un plan file con los módulos de la wave.

## Lo que hace internamente por módulo

```
1. Mapea estructura con codegraph (callers, callees, API surface)
2. Corre tools: cargo check, clippy, machete, outdated, audit, deny
3. Escanea patrones: expect, unwrap, unsafe, todo, clone, lock
4. Review manual asistido: errores, performance, concurrencia, seguridad, arquitectura, testing
5. (full) Investiga cada hallazgo en internet
6. (full) Compara con competidores por feature
7. Triage: fix ahora / backlog (DRV-NNN) / descartar
8. Actualiza Backlog.md
9. Reporta y cede el turno
```

## IDs de hallazgos

Todos los hallazgos que van al backlog usan el prefijo `DRV-NNN` (Deep ReView).
