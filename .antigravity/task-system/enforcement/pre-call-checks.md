# Pre-Call Check Rules

These rules are injected into the recitation block or agent prompt to guard every
tool invocation. Each check runs BEFORE the tool executes and can block or warn.

---

## 1. Edit Guard — `max_edit_lines`

**Purpose:** Prevent large single-shot edits that bypass review or exceed
per-state granularity limits.

**Trigger:** Tool name is `Edit`, `Write`, `edit_file`, `write_file`,
`create_or_update_file`, or `multiedit` with state parameter
`max_edit_lines` set.

**Action:** Estimate changed lines (difference between old/new strings, or
total content lines for Write). If estimate > `max_edit_lines`, **block** the
edit with a message showing the limit and suggesting smaller chunks.

**Example:**
```
State: "implementing" { max_edit_lines: 25 }
Agent sends Edit with 40-line new_string → BLOCKED:
  "Edit rejected: 40 lines changed exceeds limit of 25 for state
   'implementing'. Break the edit into smaller changes."
```

---

## 2. Bash Command Allow-Listing — `allowed_commands`

**Purpose:** Whitelist specific command prefixes for a state so the agent
can only run approved commands via the Bash tool.

**Trigger:** State has `allowed_commands` (array of prefix strings) and
tool name matches `allowed_commands_tool` (default: `"Bash"`).

**Action:** Extract the `command` argument, check if any prefix in
`allowed_commands` matches the command start. If none match, **block** with a
message listing allowed commands.

**Example:**
```
State: "verify" { allowed_commands: ["pytest", "cargo test", "npm test"] }
Agent sends Bash "rm -rf target/" → BLOCKED:
  "Command rejected: 'rm -rf target/' is not in the allowed commands for
   state 'verify'. Allowed: pytest, cargo test, npm test"
```

---

## 3. Bash Operation Classifier — tool-based blocking

**Purpose:** When `allowed_commands` is NOT set, classify Bash commands by
operation type and block operations whose required tool is absent from the
state's `allowed_tools`. This prevents bypassing Write/Edit/Read
restrictions via Bash redirects, `sed -i`, `cat >`, etc.

**Trigger:** State has no `allowed_commands` but has `allowed_tools`. Tool
name matches the Bash tool name. Command is classified via the operation
classifier.

**Action:** Classify each segment of the command (split on `&&`, `;`, `|`).
If any segment's required tool is missing from `allowed_tools`, **block**
with a message naming the operation class and required tool.

### Operation Classes and Required Tools

| Class | Required Tool | Examples |
|-------|--------------|----------|
| `Passthrough` | (none, always OK) | `echo`, `ls`, `pwd`, `cargo test` |
| `FileRead` | `Read` | `cat`, `head`, `tail`, `less`, `bat` |
| `ContentSearch` | `Grep` | `grep`, `rg`, `ag`, `ack`, `ripgrep` |
| `FileSearch` | `Glob` | `find`, `fd`, `locate` |
| `FileModify` | `Edit` | `sed -i`, `awk -i inplace`, `perl -pi`, `patch` |
| `FileWrite` | `Write` or `Edit` | `tee`, `cp`, `mv`, `dd`, any command with `>`/`>>` redirect |
| `Destructive` | `__BLOCKED__` (always blocked in restricted states) | `rm`, `rmdir`, `shred`, `truncate`, `git clean` |
| `Network` | `WebFetch` | `curl`, `wget`, `nc`, `ssh`, `scp`, `git clone/pull/fetch` |
| `GitRead` | `Read` | `git log`, `git status`, `git diff`, `git show`, `git branch` |
| `GitWrite` | `__GIT_WRITE__` (explicit gate) | `git commit`, `git push`, `git rebase`, `git merge` |

**Example:**
```
State: "planning" { allowed_tools: ["Read", "Grep", "Glob"] }
Agent sends Bash "cat > file.rs << 'EOF'" → BLOCKED:
  "Bash command blocked in state 'planning': Command does not have
   FileWrite operation which requires Write or Edit in allowed_tools."
```

---

## 4. File Scope — `max_files_per_state`

**Purpose:** Limit the number of distinct files an agent can edit within a
single state, forcing transitions between phases rather than editing
everything at once.

**Trigger:** State has `max_files_per_state` set and tool is an edit tool.

**Action:** Track `files_edited` set per state. If the file argument is
NOT already in `files_edited` AND `files_edited.len() >= max_files_per_state`,
**block** the edit. Re-edits of the same file are always allowed.

**Example:**
```
State: "implementing" { max_files_per_state: 3 }
files_edited: ["src/a.rs", "src/b.rs", "src/c.rs"]
Agent sends Edit for "src/d.rs" → BLOCKED:
  "Edit rejected: already edited 3 files in state 'implementing'
   (limit: 3). Transition to next state or reduce scope. Files edited:
   src/a.rs, src/b.rs, src/c.rs"

Agent sends Edit for "src/a.rs" → ALLOWED (re-edit of tracked file)
```

---

## 5. Read Dedup

**Purpose:** Warn the agent when it re-reads a file that is already in
context, reducing wasted tokens and encouraging context reuse.

**Trigger:** Tool is a read tool (`Read`, `read_file`, `cat`) and the
requested file has been read ≥ 2 times in the current state.

**Action:** Append a **warning** (not a block) to the tool response.

**Example:**
```
State: "planning"
session.files_read = { "src/main.rs": 2 }
Agent sends Read "src/main.rs" → ALLOWED + WARNING:
  "[STATEWRIGHT] You have read 'src/main.rs' 2 times in this state.
   The content is already in your context."
```

---

## 6. Context Budget — `context_budget_bytes`

**Purpose:** Warn when accumulated tool-result bytes approach the per-state
context limit, prompting the agent to transition before hitting limits.

**Trigger:** State has `context_budget_bytes > 0` and current accumulated
bytes ≥ 90% of budget.

**Action:** Append a **warning** to the tool response showing percentage
used, absolute bytes, and budget.

**Example:**
```
State: "planning" { context_budget_bytes: 5000 }
context_bytes: 4600 (92%)
Agent sends any tool → ALLOWED + WARNING:
  "[STATEWRIGHT] Context budget: 92% used (4600/5000 bytes) in state
   'planning'. Consider transitioning."
```

---

## 7. Environment Blocking — `blocked_env`

**Purpose:** Prevent the agent from reading or leaking sensitive environment
variables (secrets, keys, tokens) via Bash commands.

**Trigger:** State has `blocked_env` (array of env var name prefixes).
Tool is Bash and the command references any blocked variable.

**Action:** Scan command for `$VAR`, `${VAR}`, or `$VARIABLE` patterns
matching blocked prefixes. If any match, **block** with a message naming
the blocked variable.

**Example:**
```
State: "implementing" { blocked_env: ["API_KEY", "TOKEN", "SECRET", "PASSWORD"] }
Agent sends Bash "echo $API_KEY_PROD" → BLOCKED:
  "Bash command blocked in state 'implementing': references blocked
   environment variable matching 'API_KEY'."
```

---

## Check Execution Order

Checks run sequentially; the first `Block` short-circuits. Warnings from
non-blocking checks (read dedup, context budget) accumulate into one
message returned alongside `Allow`.

```
1. Environment blocking (blocked_env)
2. Edit guard (max_edit_lines)
3. Bash command allow-listing (allowed_commands) — if set, skip classifier
4. Bash operation classifier — if allowed_commands NOT set
5. File scope (max_files_per_state)
6. Read dedup (warn)
7. Context budget (warn)
```

---

## Post-Call Tracking

After each tool call completes, update session counters:

- **file_edited**: Append path to `files_edited` (edit tools only)
- **file_read**: Increment count in `files_read` map (read tools only)
- **result_bytes**: Add response content bytes to `context_bytes` accumulator

These counters persist across the current state and reset on transition.
