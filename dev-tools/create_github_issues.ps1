<#
.SYNOPSIS
    Automates the creation of community launch issues on GitHub using the official GitHub CLI (gh).
.DESCRIPTION
    This script verifies 'gh' installation and login state, defines the 7 launch issues from 
    PUBLIC_ISSUE_DRAFTS.md, and creates them dynamically. Supports -DryRun mode.
.PARAMETER DryRun
    If specified, shows the titles, labels, and issue bodies without executing the creation API.
.EXAMPLE
    .\dev-tools\create_github_issues.ps1 -DryRun
    .\dev-tools\create_github_issues.ps1
#>

param(
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

# 1. Dependency Checks
Write-Host "=== VantaDB GitHub Issue Automator (T4.4) ===" -ForegroundColor Cyan

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Error "GitHub CLI ('gh') is not installed or not present on PATH. Please install it from https://cli.github.com/ and try again."
    exit 1
}

if (-not $DryRun) {
    Write-Host "Verifying GitHub CLI authentication status..." -ForegroundColor Gray
    & gh auth status
    if ($LASTEXITCODE -ne 0) {
        Write-Error "GitHub CLI is not authenticated. Please run 'gh auth login' first to log into your GitHub account."
        exit 1
    }
} else {
    Write-Host "[DRY-RUN] Verification of GitHub authentication bypassed." -ForegroundColor Yellow
}

# 2. Define the Issues
$Issues = @(
    @{
        Title = "Verify TestPyPI Install Flow Across Supported OSes"
        Labels = "packaging,python,triage"
        Body = @"
### Problem

The Python wheel workflow builds and smoke-installs wheels, but the TestPyPI install path still needs a clean, documented verification pass across Linux, macOS, and Windows.

### Acceptance Criteria

- TestPyPI upload is triggered manually from the `Python Wheels` workflow.
- A clean virtual environment installs `vantadb-py` from TestPyPI on each OS.
- `python -m pytest vantadb-python/tests/test_sdk.py -v` passes against the installed wheel.
- Any platform-specific dependency or compiler requirement is documented.
"@
    },
    @{
        Title = "Validate the 5-Minute Quickstart From a Clean Clone"
        Labels = "docs,good first issue,triage"
        Body = @"
### Problem

The quickstart should be proven by someone starting from a clean checkout, not from an already configured development machine.

### Acceptance Criteria

- Follow `docs/QUICKSTART.md` from a clean clone.
- Confirm CLI `put/get/list/export/audit-index` commands work as written.
- Confirm Python vector, text, and hybrid search examples run as written.
- Report any missing prerequisite, confusing wording, or platform-specific adjustment.
"@
    },
    @{
        Title = "Define Search Quality v2 Scope"
        Labels = "search,roadmap,triage"
        Body = @"
### Problem

Hybrid Retrieval v1 is intentionally conservative. Snippets, highlighting, stable ranking explanations, tokenizer evolution, and ranking improvements need a scoped design before implementation.

### Acceptance Criteria

- Define which outputs are public SDK/CLI API and which remain debug-only.
- Decide whether snippets/highlighting ship before tokenizer changes.
- Document non-goals, including competitive hybrid-search parity claims.
- Propose a small validation corpus for regression tests.
"@
    },
    @{
        Title = "Define External Benchmark Validation Matrix"
        Labels = "benchmarks,validation,triage"
        Body = @"
### Problem

Internal certification is useful, but external users need reproducible benchmark instructions and honest interpretation.

### Acceptance Criteria

- Define benchmark datasets and hardware profile assumptions.
- Separate HNSW recall, BM25 text search, and hybrid retrieval measurements.
- Document what metrics are release evidence and what metrics are exploratory.
- Keep competitive claims out of scope until reproducible evidence exists.
"@
    },
    @{
        Title = "Harden Backup and Restore Policy"
        Labels = "storage,reliability,docs"
        Body = @"
### Problem

JSONL export/import is logical data movement, not a transactional physical backup. Restore expectations need to be clearer for external operators.

### Acceptance Criteria

- Document supported restore paths for Fjall and RocksDB separately.
- Clarify when the database must be closed for file-level copies.
- Validate restore with canonical records, text search, phrase filters, and hybrid retrieval.
- Keep snapshot/checksum policy deferred until explicitly designed.
"@
    },
    @{
        Title = "Define Python Distribution Policy"
        Labels = "python,packaging,release"
        Body = @"
### Problem

The Python package is prepared for wheel validation, but production PyPI, signing, and installer support are still intentionally deferred.

### Acceptance Criteria

- Define supported Python versions and OS wheel targets.
- Decide what must pass before production PyPI publication.
- Document signing and provenance expectations.
- Keep source install as the documented default until policy is met.
"@
    },
    @{
        Title = "Improve Namespace-Scoped Memory Examples"
        Labels = "good first issue,docs,examples"
        Body = @"
### Problem

Namespace-scoped memory is the core MVP concept, but examples can be expanded without adding new features.

### Acceptance Criteria

- Add or improve examples for `put/get/list/search` with namespaces.
- Include metadata filters and text-only search.
- Avoid IQL, MCP, LLM, graph database, and enterprise claims.
- Link examples from the README or quickstart where appropriate.
"@
    }
)

# 3. Create/Print Issues
Write-Host "`nReady to process $($Issues.Length) issues..." -ForegroundColor Cyan

foreach ($Issue in $Issues) {
    Write-Host "----------------------------------------" -ForegroundColor Gray
    Write-Host "Title  : $($Issue.Title)" -ForegroundColor White
    Write-Host "Labels : $($Issue.Labels)" -ForegroundColor Yellow
    
    if ($DryRun) {
        Write-Host "[DRY-RUN] Skip API call. Body preview:" -ForegroundColor DarkYellow
        Write-Host $Issue.Body -ForegroundColor Gray
    } else {
        Write-Host "Creating issue on GitHub..." -ForegroundColor Gray
        # Execute gh issue create command
        & gh issue create --title $Issue.Title --body $Issue.Body --label $Issue.Labels
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to create issue '$($Issue.Title)'"
            exit 1
        }
        Write-Host "Successfully created! ✅" -ForegroundColor Green
    }
}

Write-Host "`nAll processed successfully! Done." -ForegroundColor Green
