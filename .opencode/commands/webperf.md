---
description: Run a web performance audit via the vanta-tuner persona (web performance sub-specialty)
---

`/webperf` targets web applications specifically. Do not use it for utility libraries, CLIs, or server-only code with no browser-facing output.

## Determine the mode

**Deep mode** — activate when any of these is available:
- A Lighthouse JSON report file (e.g. `npx lighthouse <url> --output json --output-path ./report.json`, or `npx -p chrome-devtools-mcp chrome-devtools lighthouse_audit --output-format=json` from the Chrome DevTools MCP CLI)
- A PageSpeed Insights JSON response (includes Lighthouse + CrUX)
- A CrUX API response (requires `CRUX_API_KEY` or `GOOGLE_API_KEY`)
- A DevTools performance trace
- A live URL plus the `chrome-devtools` MCP server configured in the harness (the agent can capture metrics directly via `lighthouse_audit` and `performance_*` tools)
- The Chrome DevTools MCP CLI invoked locally (via `npx -p chrome-devtools-mcp chrome-devtools <tool>` or after `npm i -g chrome-devtools-mcp`) — the user runs commands like `chrome-devtools lighthouse_audit --output-format=json` and passes the JSON output to the agent

**Quick mode** — default when none of the above are available. The agent scans source code for structural anti-patterns and labels every finding as `potential impact`.

### Quick mode patterns to check
- Large JS bundles (no code splitting, no lazy loading)
- Unoptimized images (no width/height, no loading=lazy)
- Render-blocking resources (no defer/async on scripts)
- Missing cache headers (no `Cache-Control` or `ETag`)
- Layout shifts (no width/height on embeds, late-loading fonts)
- Long tasks in main thread (>50ms)

## Run the audit

Spawn `vanta-tuner` (web performance specialization). Load `performance-optimization` skill and `observability-and-instrumentation` skill. Pass them explicitly:

> **Alternativa más liviana:** `/audit quick` (CLI checks, ~2min) si solo necesitás verificar timing/Lighthouse sin el perfilado completo de `vanta-tuner`.

- The files, components, or diff under review
- Any artifact paths (Lighthouse JSON, PSI JSON, CrUX response, trace) or pasted JSON content
- The target URL or page name when known
- A note on which mode you expect (Quick or Deep), so the agent surfaces missing inputs if Deep was intended

The subagent returns a scorecard (only populated with sourced values), a ranked list of findings, positive observations, and proactive recommendations.

## Output

Return the full audit report to the user. No synthesis or merge step is needed — this is a single-persona command.
