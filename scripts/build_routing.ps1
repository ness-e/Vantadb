#!/usr/bin/env pwsh
# Build src/server/routing.rs by taking cli_server.rs and removing the
# ranges that now live in src/server/state.rs.

param(
    [string]$Source = "src/cli_server.rs",
    [string]$Target = "src/server/routing.rs"
)

$ErrorActionPreference = "Stop"

$src = Get-Content $Source

# State-owned ranges (1-indexed, inclusive)
$removeRanges = @(
    @{ Start = 60;  End = 161 },
    @{ Start = 392; End = 505 },
    @{ Start = 506; End = 642 }
)

$keep = New-Object System.Collections.Generic.List[string]
for ($i = 1; $i -le $src.Count; $i++) {
    $inRemove = $false
    foreach ($r in $removeRanges) {
        if ($i -ge $r.Start -and $i -le $r.End) {
            $inRemove = $true
            break
        }
    }
    if (-not $inRemove) {
        $keep.Add($src[$i - 1])
    }
}

Write-Host "Kept lines: $($keep.Count) of $($src.Count)"

# Find the original doc-comment block (leading //! lines)
$firstItemIdx = 0
for ($i = 0; $i -lt $keep.Count; $i++) {
    $line = $keep[$i]
    $trimmed = $line.Trim()
    if ($trimmed -eq "" -or $trimmed.StartsWith("//!")) {
        continue
    }
    $firstItemIdx = $i
    break
}

# Convert the original //! doc-block to regular // so it can sit between the
# super::state use and the rest of the imports without breaking the
# outer-doc-comment rule.
$docBlock = @()
for ($i = 0; $i -lt $firstItemIdx; $i++) {
    $line = $keep[$i]
    if ($line.StartsWith("//!")) {
        # Strip the second `!` so `//!` becomes `// `
        $docBlock += ($line -replace '^//!', '//')
    } else {
        $docBlock += $line
    }
}

$body = @()
for ($i = $firstItemIdx; $i -lt $keep.Count; $i++) {
    $body += $keep[$i]
}

$newHeader = @(
    '//! HTTP server routing, RBAC middleware, telemetry, TLS, bootstrap and tests.',
    '//!',
    '//! REVIEW-10: extracted from `src/cli_server.rs` (5327 lines) by splitting shared',
    '//! types into [`super::state`]. This module owns everything that *runs*: routing,',
    '//! auth middleware, OTEL/tracing setup, TLS handshake, the bootstrap loop, and the',
    '//! integration tests that exercise the wired router.',
    '',
    '// Re-export items moved to `super::state` so existing call sites inside this file',
    '// (and its inline tests) keep their unqualified paths without an edit.',
    'use super::state::{',
    '    AuthIdentity, AuthRateLimiter, AuthState, ConversationTrigger, NodeDTO, QueryRequest,',
    '    QueryResponse, RequestId, ServerState, AUTH_ENTITY_NS, REQUEST_ID_HEADERS,',
    '    REQUEST_ID_MAX_LEN, REQUEST_TIMEOUT, LONG_REQUEST_TIMEOUT, SERVICE_ID_HEADER,',
    '    USER_KEY_HEADER, audit_auth, extract_namespace, extract_request_id, resolve_user_key,',
    '    simple_url_decode,',
    '};',
    ''
)

$full = ($newHeader + $docBlock + $body) -join "`n"

Set-Content -Path $Target -Value $full -Encoding utf8
Write-Host "Wrote $Target ($(((Get-Content $Target) | Measure-Object -Line).Lines) lines)"