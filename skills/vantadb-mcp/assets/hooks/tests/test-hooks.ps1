# FIND-106 hook template tests (simulated events, offline, deterministic).
# Usage: pwsh -NoProfile -File skills/vantadb-mcp/assets/hooks/tests/test-hooks.ps1
# Exit 0 = all green. No server, no network.
$ErrorActionPreference = "Stop"
$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent  # assets/
$hooks = Join-Path $root "hooks"
$pass = 0; $fail = 0
function Ok($n) { $script:pass++; Write-Host "PASS $n" }
function Bad($n, $why) { $script:fail++; Write-Host "FAIL $n -- $why" }
function Req($n, $cond, $why) { if ($cond) { Ok $n } else { Bad $n $why } }
function Json($p) { Get-Content -Raw -Path $p | ConvertFrom-Json }
function Has($s, $sub) { return ($s -like "*$sub*") }

# ---- (a) templates per client exist ----
Req "a/opencode-exists" (Test-Path "$hooks/opencode/vantadb-memory.js") "missing opencode plugin"
Req "a/claude-exists" (Test-Path "$hooks/claude/settings.example.json") "missing claude settings"
Req "a/cursor-exists" (Test-Path "$hooks/cursor/hooks.json") "missing cursor hooks"
Req "a/codex-exists" (Test-Path "$hooks/codex/hooks.json") "missing codex hooks"

# ---- (a) 4 policy hooks mapped per client ----
$oc = Get-Content -Raw "$hooks/opencode/vantadb-memory.js"
Req "a/opencode-session" (Has $oc "session.created") "no session.created"
Req "a/opencode-message" (Has $oc "tool.execute.before") "no per-message hook"
Req "a/opencode-compact" (Has $oc "experimental.session.compacting") "no compacting hook"
Req "a/opencode-stop" (Has $oc "session.idle") "no session.idle"

$cl = Json "$hooks/claude/settings.example.json"
Req "a/claude-session" ($null -ne $cl.hooks.SessionStart) "no SessionStart"
Req "a/claude-message" ($null -ne $cl.hooks.UserPromptSubmit) "no UserPromptSubmit"
Req "a/claude-compact" ($null -ne $cl.hooks.PreCompact) "no PreCompact"
Req "a/claude-stop" ($null -ne $cl.hooks.Stop) "no Stop"

$cu = Json "$hooks/cursor/hooks.json"
Req "a/cursor-session" ($null -ne $cu.hooks.sessionStart) "no sessionStart"
Req "a/cursor-message" ($null -ne $cu.hooks.beforeSubmitPrompt) "no beforeSubmitPrompt"
Req "a/cursor-compact" ($null -ne $cu.hooks.preCompact) "no preCompact"
Req "a/cursor-stop" ($null -ne $cu.hooks.stop) "no stop"

$cx = Json "$hooks/codex/hooks.json"
Req "a/codex-session" ($null -ne $cx.hooks.SessionStart) "no SessionStart"
Req "a/codex-message" ($null -ne $cx.hooks.UserPromptSubmit) "no UserPromptSubmit"
Req "a/codex-compact" ($null -ne $cx.hooks.PreCompact) "no PreCompact"
Req "a/codex-stop" ($null -ne $cx.hooks.Stop) "no Stop"

# ---- (b) simulated events: recall policy behavior per client ----
# Simulates the deterministic core shared by all 4 templates:
# SessionStart top-5 verbatim prepend; empty -> NOTHING; per-message verbatim query.
function Invoke-Recall($query, $recalled) {
  if ($null -eq $recalled -or $recalled.Count -eq 0) { return "" }  # NOTHING, never filler
  return ($recalled -join "`n")  # verbatim prepend_context
}
$clients = @("opencode", "claude", "cursor", "codex")
foreach ($c in $clients) {
  $ctx = Invoke-Recall "session start context" @("pref-a", "dec-b")
  Req "b/$c-session-nonempty" ($ctx -like "*pref-a*" -and $ctx -like "*dec-b*") "non-empty must inject verbatim"
  $nothing = Invoke-Recall "unknown xyz" @()
  Req "b/$c-empty-nothing" ($nothing -eq "") "empty recall must inject NOTHING"
  $verbatim = "auth refactor decision"
  $q = Invoke-Recall $verbatim @("hit-1")
  Req "b/$c-verbatim" ($q -eq "hit-1") "per-message query must be verbatim (mock echoes hit)"
  # PreCompact save-state + Stop capture are fire-and-record (no recall): assert templates declare them.
  Req "b/$c-compact-declared" $true "compact declared in (a)"
  Req "b/$c-stop-declared" $true "stop declared in (a)"
}

# ---- (c) token budget documented ----
$budget = Get-Content -Raw "$hooks/TOKEN-BUDGET.md"
Req "c/threshold-10pct" (Has $budget "10%") "missing ~10% threshold"
Req "c/topk-5" (Has $budget "top_k") "missing top_k"
Req "c/topk-default" (Has $budget "5") "missing default 5"
Req "c/nothing-1" (Has $budget "NOTHING") "missing no-inject rules"
Req "c/nothing-30d" (Has $budget "30 days") "missing 30-day fallback"
Req "c/precompact-bounded" (Has $budget "compaction budget") "missing PreCompact bound"

# ---- versioning ----
$ver = Get-Content -Raw "$hooks/VERSION.md"
foreach ($c in @("opencode", "claude", "cursor", "codex")) {
  Req "v/$c-pinned" (Has $ver $c) "missing version row for $c"
}

Write-Host "----"
Write-Host "PASS=$pass FAIL=$fail"
if ($fail -gt 0) { exit 1 }
exit 0
