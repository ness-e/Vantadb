# READY.md — State Machine Enforcement for VantaDB Campaign Executor

> **Source:** [Statewright Engine](https://github.com/statewright/statewright) (crates/engine)
> **Schema:** `.opencode/task-system/enforcement/state-machine-schema.json`
> **Reference:** `.opencode/task-system/prompts/iter.md` (C0 State Machine)

## 1. Concept: State Machine → Campaign Executor

A state machine is a **guarantee** over agent behavior. Instead of telling the model "be careful", you encode the exact boundaries per phase:

```
  ┌──────────┐   DONE    ┌──────────┐   FAIL    ┌────────┐
  │  PLAN    │ ────────→ │   ACT    │ ────────→ │VERIFY  │
  │          │           │          │           │        │
  │ Read only│           │ Edit tools│           │Bash cmds│
  └──────────┘           └──────────┘           └────────┘
                                                     │
                                          ┌──────────┼──────────┐
                                          ▼          ▼          ▼
                                      ┌────────┐┌────────┐┌──────────┐
                                      │ STALL  ││COLLAT. ││  PLAN    │
                                      │(3 fails)││(ok)    ││(retry)  │
                                      └────────┘└────────┘└──────────┘
```

**Key insight:** "Agents are suggestions, states are laws." The model *suggests* what to do; the state machine *enforces* what it's allowed to do.

### C0 Reference

The Campaign Executor uses a **12-state machine** defined in `.opencode/task-system/prompts/iter.md:105-125`. States:

| State | Tools | Purpose |
|-------|-------|---------|
| `PLAN` | Read, Grep, Glob, codegraph_explore | Understand code, plan change |
| `ACT` | Read, Edit, Write | Implement change |
| `VERIFY` | Bash, cargo-mcp | Run checks |
| `STALL` | Read, campaign MCP | 3 failures → human escalation |
| `COLLATERAL` | Read, Bash | Verify collateral damage |
| `RESEARCH` | Web search, Read | Investigate ambiguity |
| `EVALUATE` | Read | Self-critique |
| `REVIEW` | Read | Senior engineer review |
| `ACCEPT` | campaign MCP | Approve for close |
| `CLOSE` | Bash (git) | Commit and close |

## 2. Per-State Tool Enforcement

Every state defines **what tools the agent can see**. Tools outside `allowed_tools` are never sent in the inference request — the model cannot call what it cannot see.

### Mechanisms

| Mechanism | Layer | Effect |
|-----------|-------|--------|
| `allowed_tools` | Tool schema filtering | Model never sees the tool definition |
| `disallowed_tools` | Inverse filtering | All tools except these are visible |
| `allowed_commands` | Bash prefix whitelist | Only matching commands execute |
| `blocked_env` | Environment stripping | Env vars removed from Bash context |
| `max_iterations` | Iteration budget | Force-transition after N rounds |
| `context_budget_bytes` | Context window cap | Prevents OOM in long sessions |

### Design Rules

1. **Prefer `allowed_tools` over `disallowed_tools`** — whitelists are safer than blacklists
2. **`disallowed_tools` takes precedence** if both are set (anti-footgun)
3. **Default-deny**: if neither is set, all tools are allowed (advisory only)
4. **`allowed_commands` applies to the tool named in `allowed_commands_tool`** (default: `"Bash"`)
5. **Model routing** (`model`, `thinking_level`) is advisory unless the client supports programmatic enforcement

### Example: C0 PLAN state

```json
{
  "planning": {
    "instructions": "Analyze the codebase, understand the bug, and plan the fix. Do not edit any files.",
    "allowed_tools": ["Read", "Grep", "Glob", "codegraph_explore"],
    "disallowed_tools": ["Write", "Edit", "Bash"],
    "thinking_level": "high",
    "model": "anthropic/claude-sonnet-4-6",
    "max_iterations": 5,
    "on": {
      "PLAN_READY": "implementing",
      "FAIL": "failed"
    }
  }
}
```

## 3. Guard Conditions

Guards are **declarative predicates** evaluated against the current `context` object. They gate transitions — a transition only proceeds when all its guards pass.

### Operator Reference

| Op | Field Type | Value Type | Behavior |
|----|-----------|------------|----------|
| `eq` | any | any | Field value equals the comparison value (`==`) |
| `neq` | any | any | Field value does not equal comparison value (`!=`) |
| `gt` | number | number | Field value > comparison value |
| `gte` | number | number | Field value >= comparison value |
| `lt` | number | number | Field value < comparison value |
| `lte` | number | number | Field value <= comparison value |
| `exists` | any | ignored | Field exists in context AND is not null |
| `not_exists` | any | ignored | Field is missing OR is null |
| `in` | any | array | Field value is contained in the array |
| `contains` | string | string | Field string contains the value substring |

### Context Patching

On every transition, `event_data` is shallow-merged into `context`:

```json
// Context before: { "item_count": 0 }
// Event data:    { "item_count": 2, "payment_method": "card_4242" }
// Context after: { "item_count": 2, "payment_method": "card_4242" }
```

This is how the agent's work results propagate between states.

### Example: Guarded transition with context

```json
{
  "guards": {
    "has_items": { "field": "item_count", "op": "gt", "value": 0 },
    "has_payment": { "field": "payment_method", "op": "exists" }
  },
  "states": {
    "draft": {
      "on": {
        "SUBMIT": { "target": "pending_payment", "guards": ["has_items", "has_payment"] }
      }
    }
  }
}
```

## 4. Transition Types

### Simple

```json
"on": { "DONE": "completed" }
```
Shortest form. Just a target state name.

### Full

```json
"on": {
  "SUBMIT": {
    "target": "pending_payment",
    "guard": "has_items",
    "guards": ["has_payment", "is_verified"],
    "requires_approval": true,
    "approval_message": "Agent wants to submit order"
  }
}
```
Full control with guards and approval gate.

### Guarded (Conditional Array)

```json
"on": {
  "POSTED": [
    { "guard": "under_limit", "target": "evaluating" },
    { "guard": "at_limit", "target": "reporting" },
    { "target": "overflow" }
  ]
}
```
First matching branch wins. Un-guarded branch acts as fallback.

### Fork (Parallel Branches)

```json
"on": {
  "BUILD_DONE": {
    "fork": {
      "branches": {
        "lint": { "initial": "lint_run", "terminal": "lint_done" },
        "types": { "initial": "types_run", "terminal": "types_done" }
      },
      "join": "all",
      "on_complete": "deploying",
      "on_fail": "failed"
    }
  }
}
```
Each branch runs its own sub-machine from `initial` to `terminal`. The runtime orchestrates concurrent execution. Join strategies: `all` (wait for all) or `any` (first completion wins).

### Invoke (Sub-machine)

```json
"on": {
  "TEST_FAIL": {
    "invoke": "debug_machine",
    "on_complete": "testing",
    "on_fail": "failed",
    "input": { "scope": "last_change" }
  }
}
```
Delegates to a sub-machine definition. The parent machine pauses until the sub-machine completes or fails.

## 5. Approval Gates

When `requires_approval: true`, the orchestrator pauses and **waits for human confirmation** before executing the transition. The `approval_message` is displayed to the human.

```json
"on": {
  "TESTS_PASS": {
    "target": "deploy",
    "requires_approval": true,
    "approval_message": "All tests pass. Deploy to production?"
  }
}
```

Approval gates are **hard enforcement** — the transition cannot proceed without human consent. The orchestrator uses `$statewright_approve` / `$statewright_reject` hooks.

Typical use cases: deploying to production, deleting data, destructive operations, external API calls with side effects.

## 6. Implicit Transitions (Fuzzy Matching + safe_next)

The engine provides **three layers** of fallback when the model emits an unrecognized event:

### Layer 1: Target Name Matching
If the model sends a **state name** instead of an event, and exactly one event leads to that state, the engine resolves it:
```
Model says → "completed"
Events:    TESTS_PASS → "completed", FAIL → "failed"
Resolves:  TESTS_PASS (only one event targets "completed")
```

### Layer 2: Substring Matching
If no exact event or target match, the engine tries **case-insensitive substring matching**:
```
Model says → "PASS"
Events:    TESTS_PASS → "completed", FAIL → "failed"
Resolves:  TESTS_PASS ("PASS" is a substring of "TESTS_PASS")
```

### Layer 3: safe_next
If both layers fail and `safe_next` is defined on the state, the engine routes to the fallback target:
```json
"working": {
  "safe_next": "done",
  "on": { "FINISH": "done", "FAIL": "failed" }
}
```
The model emits `"COMPLETE"` → no match → safely routes to `"done"`.

**Note:** `safe_next` does NOT override valid events. If `FAIL` is defined, it still works normally.

### History State Pattern ($return)

Interrupts use the `$return` special target. When triggered, the engine:
1. Looks up `context._interrupt_return` for the saved return state
2. Transitions to that state
3. Removes `_interrupt_return` from context

```json
"pb_validating": {
  "on": { "VALIDATED": "$return", "FAIL": "failed" }
}
```

## 7. Fork/Join

Fork transitions enable **parallel execution** of independent sub-tasks.

### Branch Definition

```json
"fork": {
  "branches": {
    "lint":  { "initial": "lint_run",  "terminal": "lint_done" },
    "types": { "initial": "types_run",  "terminal": "types_done" }
  },
  "join": "all",
  "on_complete": "deploying",
  "on_fail": "failed"
}
```

Each branch:
- Starts at `initial` state
- Runs independently until it reaches `terminal` state
- Reports completion when it enters a `type: final` state

### Join Strategies

| Strategy | Behavior |
|----------|----------|
| `all` (default) | Wait for ALL branches to complete. Transitions to `on_complete`. If any branch fails → `on_fail`. |
| `any` | Proceed when the FIRST branch completes. Other branches are cancelled. |

### Validation Rules for Forks

- `initial` and `terminal` states of every branch must exist in `states`
- `on_complete` and `on_fail` must reference existing states
- Branch states must be reachable from fork (the validator treats them as reachable from initial)

## 8. Interrupts (File-Driven Detours)

Interrupts are **reactive auto-transitions** triggered by file edit events matching a glob pattern. They use the History State pattern to save and resume.

### Definition

```json
"interrupts": {
  "pb_check": {
    "trigger": { "file_pattern": "site/pb/**/*.js" },
    "target": "pb_validating"
  },
  "env_audit": {
    "trigger": { "file_pattern": "**/*.env*" },
    "target": "env_checking"
  }
}
```

### Flow

1. Agent edits a file matching `file_pattern`
2. Runtime saves current state to `context._interrupt_return`
3. Transitions to `target` state
4. Target state handles the interrupt (e.g. validates, fixes, reports)
5. Target state emits event that triggers `$return` transition
6. Runtime restores the saved state and cleans up `_interrupt_return`

### Glob Patterns

| Pattern | Matches |
|---------|---------|
| `**/*.js` | Any `.js` file at any depth |
| `site/pb/**/*.js` | Any `.js` under `site/pb/` |
| `**/*.env*` | `.env`, `.env.local`, `config/.env.production` |
| `src/*.rs` | `.rs` files directly in `src/` (not subdirectories) |
| `src/?.rs` | Single-char `.rs` files like `a.rs`, `b.rs` |

## 9. How to Write State Machine Definitions

### Step 1: Identify the states

Map your workflow phases. For each state, ask:
- What tools does the agent need?
- What constraints apply (iterations, edit lines, files)?
- What events can end this state?
- What state comes next for each event?

### Step 2: Write the definition

```json
{
  "id": "bug-fix",
  "initial": "planning",
  "context": { "fix_count": 0 },
  "meta": {
    "task_type": "bug_fix",
    "danger_level": "moderate",
    "default_model": "anthropic/claude-sonnet-4-6"
  },
  "states": {
    "planning": {
      "allowed_tools": ["Read", "Grep", "codegraph_explore"],
      "max_iterations": 5,
      "instructions": "Analyze the bug report and understand the root cause.",
      "on": { "PLAN_READY": "implementing", "FAIL": "failed" }
    },
    "implementing": {
      "allowed_tools": ["Read", "Edit", "Write", "Bash"],
      "allowed_commands": ["cargo check", "cargo test"],
      "max_edit_lines": 200,
      "instructions": "Implement the fix following the plan.",
      "on": { "DONE": "testing", "FAIL": "failed" }
    },
    "testing": {
      "allowed_tools": ["Bash", "Read"],
      "allowed_commands": ["cargo nextest", "cargo check"],
      "on": {
        "TESTS_PASS": { "target": "review", "requires_approval": true },
        "TESTS_FAIL": "implementing",
        "FAIL": "failed"
      }
    },
    "review": {
      "allowed_tools": ["Read", "codegraph_explore"],
      "thinking_level": "high",
      "on": { "APPROVED": "completed", "REJECTED": "implementing" }
    },
    "completed": { "type": "final" },
    "failed": { "type": "final" }
  },
  "guards": {
    "has_fix_count": { "field": "fix_count", "op": "gte", "value": 1 }
  }
}
```

### Step 3: Validate

Use the validation engine (from `validate.rs`):

| Check | Enforced |
|-------|----------|
| Initial state exists | ✅ |
| At least one final state | ✅ |
| All transition targets exist | ✅ (except `$return`) |
| All guard references resolve | ✅ |
| All interrupt targets exist | ✅ |
| All fork branch states exist | ✅ |
| All states reachable from initial | ✅ (interrupt & fork targets exempt) |

## 10. Enforcement Strength

| Mechanism | Strength | Description |
|-----------|----------|-------------|
| `allowed_tools` | **Hard** | Tool schemas never sent to model in inference |
| `disallowed_tools` | **Hard** | Same mechanism (inverse) |
| `allowed_commands` | **Hard** | Bash tool intercepts and validates prefix |
| `blocked_env` | **Hard** | Environment stripped before Bash execution |
| `requires_approval` | **Hard** | Orchestrator pauses until human confirms |
| `max_iterations` | **Hard** | Runtime force-transitions after N rounds |
| `instructions` | Advisory | Prompt injection — model may ignore |
| `model` | Advisory* | Enforced if client supports model switching |
| `thinking_level` | Advisory* | Enforced if client supports effort control |
| `env_overrides` | Advisory | Injected as context hint, not enforced |

*\* Strengthened by the Campaign Executor's agent harness which asserts model identity before execution.*

## 11. C0 State Machine Integration

The C0 state machine (`.opencode/task-system/prompts/iter.md:105-125`) is the **default enforcement schema** for all Campaign Executor tasks. It maps directly to the Statewright pattern:

```
iter.md C0 State  →  Statewright Concept      →  Enforcement
──────────────────────────────────────────────────────────────
PLAN              →  StateDef(allowed_tools)   →  Read-only tools
ACT               →  StateDef(allowed_tools)   →  Edit + Write tools
VERIFY            →  StateDef(allowed_commands) →  Bash prefix filter
STALL             →  safe_next + guard         →  3-failure escalation
COLLATERAL        →  GuardedTransition         →  Error branching
EVALUATE          →  TransitionDef(simple)     →  Self-critique gate
REVIEW            →  TransitionDef(guarded)    →  Code review gate
ACCEPT            →  TransitionDef(requires_approval) →  Human gate
CLOSE             →  StateDef(type: final)     →  Terminal state
```

Invalid transitions (blocked by engine):

| Transition | Why blocked |
|------------|-------------|
| PLAN → EVALUATE | ❌ Not implemented |
| ACT → ACCEPT | ❌ Not verified |
| ACT → CLOSE | ❌ Not reviewed |
| ACT → REVIEW | ❌ Not evaluated |
| VERIFY → ACCEPT | ❌ Not evaluated/reviewed |

## 12. Writing New Campaign Machines

### Template

```json
{
  "id": "{{MACHINE_NAME}}",
  "initial": "{{FIRST_STATE}}",
  "context": {},
  "states": {
    "{{STATE_1}}": {
      "allowed_tools": ["Read", "codegraph_explore"],
      "max_iterations": 5,
      "instructions": "{{INSTRUCTION}}",
      "on": { "DONE": "{{NEXT_STATE}}", "FAIL": "failed" }
    },
    "completed": { "type": "final" },
    "failed": { "type": "final" }
  },
  "guards": {}
}
```

### Checklist for New Machines

- [ ] `initial` state exists in `states`
- [ ] At least one `type: final` state
- [ ] All transition targets reference existing states (or `$return`)
- [ ] All guard names reference entries in `guards`
- [ ] Machine `id` is unique (no collisions with other machines)
- [ ] Every non-final state has a `FAIL` transition (graceful failure path)
- [ ] Context fields used in guards are initialized in `context`
- [ ] Fork `join` matches the branch semantics (all vs any)
- [ ] Interrupt target states have `$return` or explicit target
- [ ] `safe_next` only defined on states where unknown events are recoverable

---

**Reference:** `.opencode/task-system/enforcement/state-machine-schema.json` — full JSON Schema (draft-07)
**Source:** `.agents/references/statewright/crates/engine/src/`
**C0 Machine:** `.opencode/task-system/prompts/iter.md:105-125`
