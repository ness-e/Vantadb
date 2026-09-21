# Floor Guard — VantaDB Reference Implementation
# Diff-scoped 5 checks for CONSTRAINTS.md floor (exit 0/1/2, patterns adaptable)
# Ver: .opencode/references/floor-guard.md y CONSTRAINTS.md
# Checked by: constraint-driven-development skill, Step 6

param(
    [string]$BaseBranch = "origin/develop",
    [switch]$DiffOnly = $true,
    [switch]$Strict = $false
)

$ErrorActionPreference = "Continue"
$exitCode = 0
$findings = @()
$warnings = @()

Write-Host "=== Floor Guard (CONSTRAINTS.md) ===" -ForegroundColor Cyan
Write-Host "Base: $BaseBranch | DiffOnly: $DiffOnly | Strict: $Strict" -ForegroundColor Gray

# Helper: get diff files
function Get-DiffFiles {
    param([string]$Pattern)
    try {
        $files = git diff $BaseBranch --name-only -- $Pattern 2>$null
        if ($LASTEXITCODE -ne 0) {
            # Fallback: origin/main or HEAD~1
            $files = git diff HEAD~1 --name-only -- $Pattern 2>$null
        }
        return @($files | Where-Object { $_ -and $_.Trim() -ne "" })
    } catch {
        return @()
    }
}

function Test-DiffContent {
    param([string[]]$Files, [string]$Pattern, [string]$Description)
    $hits = @()
    foreach ($f in $Files) {
        try {
            $diff = git diff $BaseBranch -- $f 2>$null
            if (-not $diff) { $diff = git diff HEAD~1 -- $f 2>$null }
            $matches = $diff | Select-String -Pattern $Pattern -AllMatches
            foreach ($m in $matches) {
                # Only count additions (lines starting with + but not +++)
                if ($m.Line -match "^\+\+\+|^---") { continue }
                if ($m.Line -match "^\+" -and $m.Line -notmatch "^\+\+") {
                    $hits += "${f}:$($m.LineNumber): $($m.Line.Substring(0, [Math]::Min(80, $m.Line.Length)))"
                }
            }
        } catch {}
    }
    return $hits
}

# 1. No new suppression comments (Rust: #[allow], TS: @ts-ignore, Python: # noqa)
Write-Host "`n[1/5] Checking for new suppression comments..." -ForegroundColor Yellow
$suppressionFiles = Get-DiffFiles -Pattern "*.rs *.ts *.tsx *.js *.jsx *.py"
$suppressionHits = @()
$suppressionHits += Test-DiffContent -Files $suppressionFiles -Pattern '#\[allow\(|# noqa|# type: ignore|@ts-ignore|eslint-disable|istanbul ignore|Stryker disable|nosemgrep|gitleaks:allow' -Description "suppression"
if ($suppressionHits.Count -gt 0) {
    $findings += "NEW_SUPPRESSIONS: $($suppressionHits.Count) new suppression(s) — review required"
    $suppressionHits | Select-Object -First 10 | ForEach-Object { $findings += "  - $_" }
    if ($suppressionHits.Count -gt 10) { $findings += "  ... and $($suppressionHits.Count - 10) more" }
    $exitCode = 2
    Write-Host "  FAIL: $($suppressionHits.Count) suppressions" -ForegroundColor Red
} else {
    Write-Host "  PASS: No new suppressions" -ForegroundColor Green
}

# 2. No unimplemented stubs
Write-Host "[2/5] Checking for unimplemented stubs..." -ForegroundColor Yellow
$stubFiles = Get-DiffFiles -Pattern "*.rs *.ts *.js *.py"
$stubHits = @()
$stubHits += Test-DiffContent -Files $stubFiles -Pattern 'unimplemented!|todo!\(|panic!\("not implemented|throw new Error\("Not implemented|raise NotImplementedError|empty catch|catch\s*\{\s*\}' -Description "stub"
# Filter: only check added lines that are not comments
$stubHits = $stubHits | Where-Object { $_ -notmatch "^\s*//" -and $_ -notmatch "^\s*#" }
if ($stubHits.Count -gt 0) {
    $findings += "UNIMPLEMENTED_STUBS: $($stubHits.Count) stub(s) — must be implemented or tracked"
    $stubHits | Select-Object -First 10 | ForEach-Object { $findings += "  - $_" }
    $exitCode = 2
    Write-Host "  FAIL: $($stubHits.Count) stubs" -ForegroundColor Red
} else {
    Write-Host "  PASS: No stubs" -ForegroundColor Green
}

# 3. No skipped/deleted tests without reason
Write-Host "[3/5] Checking for skipped/deleted tests..." -ForegroundColor Yellow
$testFiles = Get-DiffFiles -Pattern "*test* *spec*"
$skippedHits = Test-DiffContent -Files $testFiles -Pattern '#\[ignore\]|\.skip\(|xit\(|test\.skip|@pytest\.mark\.skip' -Description "skipped test"
if ($skippedHits.Count -gt 0) {
    $warnings += "SKIPPED_TESTS: $($skippedHits.Count) skipped test(s) — ensure commit message explains why"
    $skippedHits | Select-Object -First 5 | ForEach-Object { $warnings += "  - $_" }
    if (-not $Strict) { $exitCode = [Math]::Max($exitCode, 1) } else { $exitCode = 2 }
    Write-Host "  WARN: $($skippedHits.Count) skipped" -ForegroundColor Yellow
} else {
    Write-Host "  PASS: No skipped tests" -ForegroundColor Green
}
# Deleted tests: check for deleted files that were tests
try {
    $deleted = git diff $BaseBranch --name-only --diff-filter=D 2>$null | Where-Object { $_ -match "test|spec" }
    if ($deleted) {
        $warnings += "DELETED_TESTS: $($deleted.Count) test file(s) deleted — ensure commit explains"
        $deleted | Select-Object -First 5 | ForEach-Object { $warnings += "  - $_" }
        if (-not $Strict) { $exitCode = [Math]::Max($exitCode, 1) } else { $exitCode = 2 }
        Write-Host "  WARN: $($deleted.Count) deleted test files" -ForegroundColor Yellow
    }
} catch {}

# 4. Secrets in source (gitleaks if available, else regex)
Write-Host "[4/5] Checking for secrets..." -ForegroundColor Yellow
$secretFound = $false
if (Get-Command gitleaks -ErrorAction SilentlyContinue) {
    try {
        $leaks = gitleaks detect --redact --no-banner --source . --config-path=dev-tools/gitleaks.toml 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0 -and $leaks -match "leak|secret|found") {
            $findings += "SECRETS_DETECTED: gitleaks found leaks (redacted)"
            $findings += "  $leaks"
            $secretFound = $true
            $exitCode = 2
            Write-Host "  FAIL: gitleaks detected leaks" -ForegroundColor Red
        } else {
            Write-Host "  PASS: gitleaks clean" -ForegroundColor Green
        }
    } catch {
        Write-Host "  SKIP: gitleaks error, falling back to regex" -ForegroundColor Gray
        $secretFound = $false
    }
}
if (-not $secretFound -and -not (Get-Command gitleaks -ErrorAction SilentlyContinue)) {
    # Fallback regex for common secrets (only in diff, not full scan)
    $secretPatterns = @(
        '(?i)(api[_-]?key|apikey)\s*[:=]\s*[''"][^''"]{16,}[''"]',
        '(?i)(secret|token)\s*[:=]\s*[''"][^''"]{16,}[''"]',
        'AKIA[0-9A-Z]{16}',
        'ghp_[a-zA-Z0-9]{36}',
        'sk-[a-zA-Z0-9]{20,}'
    )
    $secretHits = @()
    $allChanged = git diff $BaseBranch --name-only 2>$null
    if (-not $allChanged) { $allChanged = git diff HEAD~1 --name-only 2>$null }
    foreach ($f in $allChanged) {
        if ($f -match "\.(md|txt|example|sample)$") { continue }
        if (-not (Test-Path $f)) { continue }
        try {
            $content = git diff $BaseBranch -- $f 2>$null | Out-String
            foreach ($pat in $secretPatterns) {
                if ($content -match $pat) {
                    $secretHits += "${f}: potential secret pattern"
                    break
                }
            }
        } catch {}
    }
    if ($secretHits.Count -gt 0) {
        $findings += "SECRETS_REGEX: $($secretHits.Count) potential secret(s) via regex"
        $secretHits | Select-Object -First 5 | ForEach-Object { $findings += "  - $_" }
        $exitCode = 2
        Write-Host "  FAIL: $($secretHits.Count) potential secrets (regex)" -ForegroundColor Red
    } else {
        Write-Host "  PASS: No secrets (regex)" -ForegroundColor Green
    }
}

# 5. CONSTRAINTS.md not weakened
Write-Host "[5/5] Checking CONSTRAINTS.md not weakened..." -ForegroundColor Yellow
try {
    $constraintsDiff = git diff $BaseBranch -- CONSTRAINTS.md 2>$null | Out-String
    if (-not $constraintsDiff) { $constraintsDiff = git diff HEAD~1 -- CONSTRAINTS.md 2>$null | Out-String }
    if ($constraintsDiff) {
        # Check if thresholds lowered (look for - lines with numbers)
        $removedThresholds = $constraintsDiff | Select-String -Pattern "^-.*\|.*\d+.*\|" -AllMatches
        $addedThresholds = $constraintsDiff | Select-String -Pattern "^\+\s*\|.*\d+.*\|" -AllMatches
        if ($removedThresholds -and $constraintsDiff -match "-\s*\|.*\d+%|-\s*\|.*\d+\s*ms|-\s*\|.*coverage") {
            $findings += "CONSTRAINTS_WEAKENED: CONSTRAINTS.md thresholds may have been lowered"
            $findings += "  Diff: $($constraintsDiff.Substring(0, [Math]::Min(200, $constraintsDiff.Length)))"
            $exitCode = 2
            Write-Host "  FAIL: CONSTRAINTS.md weakened" -ForegroundColor Red
        } else {
            Write-Host "  PASS: CONSTRAINTS.md not weakened (or strengthened)" -ForegroundColor Green
        }
    } else {
        Write-Host "  PASS: No changes to CONSTRAINTS.md" -ForegroundColor Green
    }
} catch {
    Write-Host "  PASS: No CONSTRAINTS.md diff to check" -ForegroundColor Green
}

# Summary
Write-Host "`n=== Floor Guard Summary ===" -ForegroundColor Cyan
if ($findings.Count -gt 0) {
    Write-Host "FAILURES:" -ForegroundColor Red
    $findings | ForEach-Object { Write-Host $_ -ForegroundColor Red }
}
if ($warnings.Count -gt 0) {
    Write-Host "WARNINGS:" -ForegroundColor Yellow
    $warnings | ForEach-Object { Write-Host $_ -ForegroundColor Yellow }
}
if ($findings.Count -eq 0 -and $warnings.Count -eq 0) {
    Write-Host "FLOOR GUARD PASSED (all 5 checks)" -ForegroundColor Green
} elseif ($findings.Count -eq 0) {
    Write-Host "FLOOR GUARD PASSED WITH WARNINGS" -ForegroundColor Yellow
} else {
    Write-Host "FLOOR GUARD FAILED" -ForegroundColor Red
}

Write-Host "Exit code: $exitCode (0=pass, 1=warn, 2=fail)" -ForegroundColor Gray
exit $exitCode
