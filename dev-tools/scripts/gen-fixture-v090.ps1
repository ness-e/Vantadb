# gen-fixture-v090.ps1 — ADR-0060 D8(a): reproducible v0.9.0 database fixture.
#
# Builds the vanta-cli binary from the `v0.9.0` git tag in a throwaway
# worktree under %TEMP% (never inside the repo; CARGO_TARGET_DIR also in
# %TEMP% so the repo target/ is untouched), creates a small database with
# only v0.9.0 CLI subcommands, and packs it into tests/fixtures/v0.9.0/
# together with MANIFEST.json (tag, tag commit, commands, expected contents,
# per-file sha256).
#
# Usage:  pwsh dev-tools/scripts/gen-fixture-v090.ps1
# Stop conditions: fixture must stay <= 5 MB.

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false

$repo = (git rev-parse --show-toplevel).Trim()
Set-Location $repo

$tag       = 'v0.9.0'
$tagCommit = (git rev-list -n1 $tag).Trim()
$fixture   = Join-Path $repo 'tests/fixtures/v0.9.0'
$maxBytes  = 5MB

$tmp = Join-Path $env:TEMP ("vanta-fixture-v090-" + [guid]::NewGuid().ToString('N'))
$wt  = Join-Path $tmp 'wt'
# Persistent cargo target cache in %TEMP% (never the repo target/): a rerun
# only rebuilds what changed, the first run pays the full cold build.
$tgt = Join-Path $env:TEMP 'vanta-v090-target'
$db  = Join-Path $tmp 'db'

$commands = [System.Collections.Generic.List[string]]::new()
function Invoke-VantaCli {
    param([string[]]$CliArgs)
    $commands.Add("vanta-cli " + ($CliArgs -join ' '))
    & $script:vanta @CliArgs | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "vanta-cli failed ($LASTEXITCODE): $($CliArgs -join ' ')" }
}

try {
    Write-Host "== worktree add $tag -> $wt"
    git worktree add --detach $wt $tag | Out-Null

    Write-Host "== building vanta-cli @ $tag (debug, --jobs 4)"
    $env:CARGO_TARGET_DIR = $tgt
    Push-Location $wt
    try { cargo build --bin vanta-cli --jobs 4 } finally { Pop-Location }
    $script:vanta = Join-Path $tgt 'debug/vanta-cli.exe'
    if (-not (Test-Path $script:vanta)) { throw "vanta-cli binary not found at $script:vanta" }

    # --- populate the database with v0.9.0 subcommands only ---------------
    # Namespace `docs`: 6 records, all with 4-d vectors + metadata.
    $docs = @(
        @('doc1','quantum lighthouse photon harbour','0.10,0.20,0.30,0.40'),
        @('doc2','harbour lighthouse beacon signal','0.11,0.19,0.31,0.39'),
        @('doc3','quantum entanglement ledger cipher','0.90,0.05,0.03,0.02'),
        @('doc4','meadow orchard harvest breeze','0.02,0.90,0.05,0.03'),
        @('doc5','cipher ledger quantum vault','0.88,0.06,0.04,0.02'),
        @('doc6','orchard meadow dusk harvest','0.03,0.91,0.04,0.02')
    )
    foreach ($d in $docs) {
        Invoke-VantaCli @('--db', $db, 'put', '--namespace', 'docs',
            '--key', $d[0], '--payload', $d[1], '--vector', $d[2],
            '--metadata', ('{\"topic\":\"' + $d[0] + '-topic\",\"origin\":\"fixture\"}'))
    }
    # Namespace `notes`: 2 text-only records with metadata (pre-rotation).
    # NOTE: the WAL is sharded (4 shards, round-robin). 8 puts pre-rotation
    # keep every shard's durable record count EQUAL, which the ERR-011
    # round-robin coherence check (`verify_shard_counts`, src/wal_sharded.rs)
    # requires of the archive+live layout produced below.
    foreach ($n in @('note1','note2')) {
        Invoke-VantaCli @('--db', $db, 'put', '--namespace', 'notes',
            '--key', $n, '--payload', "scratch note $n quantum appendix",
            '--metadata', ('{\"kind\":\"note\",\"seq\":\"' + $n + '\"}'))
    }
    # WAL rotation: archive the live segment(s) exactly the way WalWriter::rotate
    # does (rename to `vanta*.wal.<epoch_ms>`), then keep writing so a fresh
    # live segment is created. v0.9.0 hardcodes max_segment_size = 256 MiB, so
    # automatic rotation is unreachable within the 5 MB fixture budget — this
    # rename IS the on-disk result of a rotation.
    $dataDir = Join-Path $db 'data'
    $rotTs   = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    Get-ChildItem $dataDir -Filter 'vanta*.wal' |
        Where-Object { $_.Name -notmatch '\.wal\.' } |
        ForEach-Object {
            $arch = "$($_.FullName).$rotTs"
            Move-Item $_.FullName $arch
            $commands.Add("# simulated WAL rotation: rename $($_.Name) -> $($_.Name).$rotTs")
        }
    # More writes + one delete (tombstone) into the fresh WAL segment.
    # Exactly 4 WAL ops post-rotation (put, delete, put, put) so every live
    # shard receives the same number of new records — round-robin coherence.
    Invoke-VantaCli @('--db', $db, 'put', '--namespace', 'docs',
        '--key', 'doc_gone', '--payload', 'ephemeral record to delete',
        '--vector', '0.50,0.50,0.00,0.00', '--metadata', '{\"topic\":\"deleted\"}')
    Invoke-VantaCli @('--db', $db, 'delete', '--namespace', 'docs', '--key', 'doc_gone')
    Invoke-VantaCli @('--db', $db, 'put', '--namespace', 'docs',
        '--key', 'doc7', '--payload', 'post rotation quantum lighthouse',
        '--vector', '0.12,0.18,0.33,0.37', '--metadata', '{\"topic\":\"post-rotation\"}')
    Invoke-VantaCli @('--db', $db, 'put', '--namespace', 'notes',
        '--key', 'note3', '--payload', 'scratch note note3 quantum appendix',
        '--metadata', '{\"kind\":\"note\",\"seq\":\"note3\"}')
    # Persist index state (vector_index.bin etc.) before packing.
    Invoke-VantaCli @('--db', $db, 'rebuild-index')

    # v0.9.0 pre-allocates every vstore_L*.vanta segment at 64 MiB
    # (`File::open(path, 64 * 1024 * 1024)` in lsm.rs); bytes beyond the
    # write cursor (u64 LE at header offset 16) are pure zero padding.
    # Truncate each segment at its write cursor so the fixture fits the 5 MB
    # budget — the on-disk format and every record byte are untouched, and
    # both v0.9.0 and v0.10.0 read by offset, not by file length (verified:
    # a truncated segment opens and reads correctly under this build).
    Get-ChildItem $dataDir -Filter 'vstore_L*.vanta' | ForEach-Object {
        $fs = [System.IO.File]::OpenRead($_.FullName)
        try {
            $buf = New-Object byte[] 8
            [void]$fs.Seek(16, 'Begin')
            [void]$fs.Read($buf, 0, 8)
            $cursor = [Math]::Max([BitConverter]::ToUInt64($buf, 0), 64)
        } finally { $fs.Close() }
        $w = [System.IO.File]::Open($_.FullName, 'Open', 'Write')
        try { $w.SetLength($cursor) } finally { $w.Close() }
        $commands.Add("# truncated " + $_.Name + " at write_cursor=" + $cursor +
            " (was 64MiB pre-allocation, tail was zero padding)")
        Write-Host "  truncated $($_.Name) -> $cursor bytes"
    }

    # --- pack the fixture --------------------------------------------------
    if (Test-Path $fixture) { Remove-Item $fixture -Recurse -Force }
    New-Item -ItemType Directory -Force (Join-Path $fixture 'db') | Out-Null
    Copy-Item (Join-Path $db '*') (Join-Path $fixture 'db') -Recurse -Force

    $files = @{}
    Get-ChildItem (Join-Path $fixture 'db') -Recurse -File | ForEach-Object {
        $rel = $_.FullName.Substring($fixture.Length + 1).Replace('\','/')
        $files[$rel] = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    $total = (Get-ChildItem $fixture -Recurse -File | Measure-Object Length -Sum).Sum
    if ($total -gt $maxBytes) { throw "fixture too large: $total bytes > $maxBytes" }

    $manifest = [ordered]@{
        generator   = 'dev-tools/scripts/gen-fixture-v090.ps1'
        tag         = $tag
        tag_commit  = $tagCommit
        created_utc = [DateTime]::UtcNow.ToString('o')
        commands    = $commands
        expected    = [ordered]@{
            namespaces    = [ordered]@{
                docs  = @('doc1','doc2','doc3','doc4','doc5','doc6','doc7')
                notes = @('note1','note2','note3')
            }
            payloads      = [ordered]@{
                'docs/doc1'  = 'quantum lighthouse photon harbour'
                'docs/doc2'  = 'harbour lighthouse beacon signal'
                'docs/doc3'  = 'quantum entanglement ledger cipher'
                'docs/doc4'  = 'meadow orchard harvest breeze'
                'docs/doc5'  = 'cipher ledger quantum vault'
                'docs/doc6'  = 'orchard meadow dusk harvest'
                'docs/doc7'  = 'post rotation quantum lighthouse'
                'notes/note1' = 'scratch note note1 quantum appendix'
                'notes/note2' = 'scratch note note2 quantum appendix'
                'notes/note3' = 'scratch note note3 quantum appendix'
            }
            deleted_keys  = [ordered]@{ docs = @('doc_gone') }
            vector_dim    = 4
            vector_query  = @(0.10, 0.20, 0.30, 0.40)
            text_query    = 'quantum'
            text_hits_docs = @('doc1','doc3','doc5','doc7')
        }
        files_sha256 = $files
    }
    # UTF-8 *without* BOM — serde_json rejects a BOM at byte 0.
    [System.IO.File]::WriteAllText(
        (Join-Path $fixture 'MANIFEST.json'),
        ($manifest | ConvertTo-Json -Depth 8))

    Write-Host ("== fixture written: {0} files, {1} bytes" -f $files.Count, $total)
}
finally {
    if (Test-Path $wt) { git worktree remove --force $wt }
    if (git worktree list | Select-String -SimpleMatch $wt) {
        throw "worktree $wt still registered"
    }
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "== worktree removed; temp cleaned"
}
