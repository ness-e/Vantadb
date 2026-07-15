<#
.SYNOPSIS
    VantaDB Campaign Executor Session Tracking Module.
.DESCRIPTION
    Manages state machine sessions for the Campaign Executor. Tracks current
    state, iteration counts, file edits/reads, context budget, and plan limits.
    Sessions are persisted as JSON files so they survive between harness invocations.

    Import via: Import-Module .agents\enforcement\session-tracking.ps1 -Force
    Default store: .agents\enforcement\sessions\
.NOTES
    Adapted from statewright GatewaySession + SessionManager (session.rs).
#>

# --- Module state ---
$script:SessionStorePath = Join-Path $PSScriptRoot 'sessions'
if (-not (Test-Path -LiteralPath $script:SessionStorePath)) {
    New-Item -ItemType Directory -Path $script:SessionStorePath -Force | Out-Null
}

# Cache for current session operations
$script:SessionCache = @{}

# ============================================================================
# Internal helpers
# ============================================================================

function Get-SessionFileName {
    param([string]$SessionId)
    $safe = $SessionId -replace '[<>:"/\\|?*]', '_'
    Join-Path $script:SessionStorePath "$safe.json"
}

function Save-SessionToDisk {
    param([hashtable]$Session)
    $path = Get-SessionFileName $Session.instance_id
    $json = $Session | ConvertTo-Json -Depth 10
    Set-Content -Path $path -Value $json -Encoding UTF8
    $script:SessionCache[$Session.instance_id] = $Session
}

function Load-SessionFromDisk {
    param([string]$SessionId)
    $path = Get-SessionFileName $SessionId
    if (Test-Path -LiteralPath $path) {
        $json = Get-Content -Path $path -Raw -Encoding UTF8 | ConvertFrom-Json
        $h = @{}
        $json.PSObject.Properties | ForEach-Object { $h[$_.Name] = $_.Value }
        # Deep-convert nested hashtables
        if ($h.files_read -is [PSCustomObject]) {
            $d = @{}
            $h.files_read.PSObject.Properties | ForEach-Object { $d[$_.Name] = [int]$_.Value }
            $h.files_read = $d
        }
        if ($h.files_edited -is [Array]) {
            $h.files_edited = @($h.files_edited)
        } else {
            $h.files_edited = @()
        }
        $h.context_bytes = [uint64]($h.context_bytes -as [uint64])
        $h.iteration_count = [uint32]($h.iteration_count -as [uint32])
        $h.transition_count = [uint64]($h.transition_count -as [uint64])
        $script:SessionCache[$SessionId] = $h
        return $h
    }
    return $null
}

# ============================================================================
# Public functions
# ============================================================================

<#
.SYNOPSIS
    Creates a new Vanta session.
.DESCRIPTION
    Initialises a session with an instance ID, state machine definition (as
    hashtable with id, initial, states, guards), and optional plan limit.
.PARAMETER InstanceId
    Unique identifier for this session.
.PARAMETER Definition
    Hashtable describing the state machine. Must contain at least:
      @{ id="..."; initial="state_name"; states=@{...}; guards=@{} }
.PARAMETER PlanLimit
    Optional max transition count (None = unlimited).
.PARAMETER Force
    Overwrite existing session with same ID.
.EXAMPLE
    $def = Get-Content 'machine.json' -Raw | ConvertFrom-Json
    $s = New-VantaSession -InstanceId "campaign-01" -Definition $def -PlanLimit 50
#>
function New-VantaSession {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId,

        [Parameter(Mandatory)]
        [hashtable]$Definition,

        [uint64]$PlanLimit = 0,

        [switch]$Force
    )

    $existing = Load-SessionFromDisk $InstanceId
    if ($existing -and -not $Force) {
        # Preserve _fork context (parallel fork guard)
        if ($existing.context._fork) {
            return $existing
        }
        Write-Warning "Session '$InstanceId' already exists. Use -Force to overwrite."
        return $existing
    }

    $context = @{}
    if ($Definition.ContainsKey('context') -and $Definition.context -is [hashtable]) {
        $context = $Definition.context
    }

    $session = @{
        instance_id        = $InstanceId
        definition         = $Definition
        current_state      = $Definition.initial
        previous_state     = $null
        context            = $context
        iteration_count    = [uint32]0
        transition_count   = [uint64]0
        plan_limit         = if ($PlanLimit -gt 0) { [uint64]$PlanLimit } else { $null }
        files_edited       = @()
        context_bytes      = [uint64]0
        files_read         = @{}
        pending_approval   = $null
    }

    Save-SessionToDisk $session
    return $session
}

<#
.SYNOPSIS
    Gets a session by ID.
.PARAMETER InstanceId
    Session identifier.
.EXAMPLE
    $s = Get-VantaSession -InstanceId "campaign-01"
#>
function Get-VantaSession {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId
    )

    if ($script:SessionCache.ContainsKey($InstanceId)) {
        return $script:SessionCache[$InstanceId]
    }
    return Load-SessionFromDisk $InstanceId
}

<#
.SYNOPSIS
    Transitions session to a new state, resetting per-state counters.
.DESCRIPTION
    Sets previous_state → current_state, moves to new_state, resets
    iteration_count to 0, files_edited to empty, context_bytes to 0,
    files_read to empty, increments transition_count.
.PARAMETER InstanceId
    Session identifier.
.PARAMETER NewState
    Target state name.
.PARAMETER Context
    New context hashtable (defaults to empty).
.EXAMPLE
    Update-VantaSessionState -InstanceId "campaign-01" -NewState "implementing"
#>
function Update-VantaSessionState {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId,

        [Parameter(Mandatory)]
        [string]$NewState,

        [hashtable]$Context = @{}
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) {
        Write-Error "Session '$InstanceId' not found."
        return $false
    }

    $session.previous_state   = $session.current_state
    $session.current_state    = $NewState
    $session.context          = $Context
    $session.iteration_count  = [uint32]0
    $session.transition_count = [uint64]($session.transition_count + 1)
    $session.files_edited     = @()
    $session.context_bytes    = [uint64]0
    $session.files_read       = @{}
    # pending_approval NOT cleared here — caller manages it

    Save-SessionToDisk $session
    return $true
}

<#
.SYNOPSIS
    Increments the iteration counter for a session.
.PARAMETER InstanceId
    Session identifier.
.EXAMPLE
    $count = Increment-VantaIteration -InstanceId "campaign-01"
#>
function Increment-VantaIteration {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) {
        Write-Error "Session '$InstanceId' not found."
        return $null
    }

    $session.iteration_count = [uint32]($session.iteration_count + 1)
    Save-SessionToDisk $session
    return $session.iteration_count
}

<#
.SYNOPSIS
    Returns usage info for plan metering.
.DESCRIPTION
    Returns (used, limit, percentage). Limit is $null if unlimited.
    Percentage is $null if limit is $null.
.EXAMPLE
    $used, $limit, $pct = Get-VantaUsage -InstanceId "campaign-01"
#>
function Get-VantaUsage {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) {
        Write-Error "Session '$InstanceId' not found."
        return (0, $null, $null)
    }

    $used = [uint64]$session.transition_count
    $limit = $session.plan_limit
    $pct = if ($limit) {
        if ($limit -eq 0) { [double]100.0 } else { [double](($used / $limit) * 100.0) }
    } else {
        $null
    }

    return ($used, $limit, $pct)
}

<#
.SYNOPSIS
    Returns a warning message if approaching plan limit.
.DESCRIPTION
    Returns null if under 80%. Warns at 80%, 90%, 95%, and 100%+.
.EXAMPLE
    $msg = Get-VantaUsageWarning -InstanceId "campaign-01"
    if ($msg) { Write-Host $msg }
#>
function Get-VantaUsageWarning {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId
    )

    $used, $limit, $pct = Get-VantaUsage $InstanceId
    if (-not $limit -or -not $pct) {
        return $null
    }

    if ($pct -ge 100.0) {
        return "Plan limit reached ($used/$limit). Upgrade to continue."
    }
    if ($pct -ge 95.0) {
        return "Warning: 95% of plan limit ($used/$limit). Consider upgrading."
    }
    if ($pct -ge 90.0) {
        return "Notice: 90% of plan limit ($used/$limit)."
    }
    if ($pct -ge 80.0) {
        return "Notice: 80% of plan limit ($used/$limit)."
    }
    return $null
}

<#
.SYNOPSIS
    Returns true if the current state's iteration count has reached or
    exceeded max_iterations for that state.
.DESCRIPTION
    Checks the state definition in the machine for max_iterations.
.EXAMPLE
    if (Test-VantaCheckpoint -InstanceId "campaign-01") { "checkpoint!" }
#>
function Test-VantaCheckpoint {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) { return $false }

    $stateDef = $session.definition.states[$session.current_state]
    if (-not $stateDef -or -not $stateDef.max_iterations) {
        return $false
    }

    $max = [uint32]$stateDef.max_iterations
    return $session.iteration_count -ge $max
}

<#
.SYNOPSIS
    Returns true if the current state is a final state.
.DESCRIPTION
    A state is final if its definition has "type": "final".
.EXAMPLE
    if (Test-VantaFinal -InstanceId "campaign-01") { "workflow complete" }
#>
function Test-VantaFinal {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) { return $false }

    $stateDef = $session.definition.states[$session.current_state]
    if (-not $stateDef) { return $false }

    return $stateDef.type -eq 'final'
}

<#
.SYNOPSIS
    Adds a file path to the edited-files list if not already present.
.DESCRIPTION
    Tracks distinct files edited per state (resets on transition).
.EXAMPLE
    Add-VantaFileEdit -InstanceId "campaign-01" -FilePath "src/main.rs"
#>
function Add-VantaFileEdit {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId,

        [Parameter(Mandatory)]
        [string]$FilePath
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) {
        Write-Error "Session '$InstanceId' not found."
        return
    }

    $normalized = $FilePath.Replace('/', '\')
    if ($session.files_edited -notcontains $normalized) {
        $session.files_edited += $normalized
        Save-SessionToDisk $session
    }
}

<#
.SYNOPSIS
    Records a file read and returns the read count.
.DESCRIPTION
    Increments the read counter for the given file path. Returns the
    new count (1 on first read).
.EXAMPLE
    $count = Add-VantaFileRead -InstanceId "campaign-01" -FilePath "src/main.rs"
    if ($count -ge 3) { "dedup warning" }
#>
function Add-VantaFileRead {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId,

        [Parameter(Mandatory)]
        [string]$FilePath
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) {
        Write-Error "Session '$InstanceId' not found."
        return 1
    }

    $normalized = $FilePath.Replace('/', '\')
    $count = [int]($session.files_read[$normalized] -or 0)
    $count++
    $session.files_read[$normalized] = $count
    Save-SessionToDisk $session
    return $count
}

<#
.SYNOPSIS
    Adds bytes to the context budget accumulator.
.DESCRIPTION
    Returns the new total context_bytes for the session.
.EXAMPLE
    $total = Add-VantaContextBytes -InstanceId "campaign-01" -Bytes 4500
#>
function Add-VantaContextBytes {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId,

        [Parameter(Mandatory)]
        [uint64]$Bytes
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) {
        Write-Error "Session '$InstanceId' not found."
        return 0
    }

    $session.context_bytes = [uint64]($session.context_bytes + $Bytes)
    Save-SessionToDisk $session
    return $session.context_bytes
}

<#
.SYNOPSIS
    Sets a pending approval gate on the session.
.DESCRIPTION
    Parks a transition until human approves or rejects.
.PARAMETER InstanceId
    Session identifier.
.PARAMETER ApprovalId
    Unique approval identifier.
.PARAMETER Event
    Transition event name.
.PARAMETER FromState
    Source state name.
.PARAMETER ToState
    Target state name.
.PARAMETER TransitContext
    New context for after the transition.
.PARAMETER Message
    Optional human-readable message for review.
.EXAMPLE
    Set-VantaPendingApproval -InstanceId "campaign-01" -ApprovalId "apr_001" `
        -Event "DEPLOY" -FromState "testing" -ToState "live" `
        -TransitContext @{ result = "pass" } -Message "Review before deploy"
#>
function Set-VantaPendingApproval {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId,

        [Parameter(Mandatory)]
        [string]$ApprovalId,

        [Parameter(Mandatory)]
        [string]$Event,

        [Parameter(Mandatory)]
        [string]$FromState,

        [Parameter(Mandatory)]
        [string]$ToState,

        [hashtable]$TransitContext = @{},

        [string]$Message = $null
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) {
        Write-Error "Session '$InstanceId' not found."
        return
    }

    $session.pending_approval = @{
        approval_id   = $ApprovalId
        event         = $Event
        from_state    = $FromState
        to_state      = $ToState
        new_context   = $TransitContext
        message       = $Message
    }
    Save-SessionToDisk $session
}

<#
.SYNOPSIS
    Clears and returns the pending approval for a session.
.EXAMPLE
    $pending = Clear-VantaPendingApproval -InstanceId "campaign-01"
    if ($pending) { "approved: $($pending.event)" }
#>
function Clear-VantaPendingApproval {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) { return $null }

    $pending = $session.pending_approval
    $session.pending_approval = $null
    Save-SessionToDisk $session
    return $pending
}

<#
.SYNOPSIS
    Returns true if the session has a pending approval.
.EXAMPLE
    if (Test-VantaPendingApproval -InstanceId "campaign-01") { "awaiting human" }
#>
function Test-VantaPendingApproval {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) { return $false }
    return $null -ne $session.pending_approval
}

<#
.SYNOPSIS
    Sets the plan limit (max transition count) for a session.
.EXAMPLE
    Set-VantaPlanLimit -InstanceId "campaign-01" -Limit 100
#>
function Set-VantaPlanLimit {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId,

        [Parameter(Mandatory)]
        [uint64]$Limit
    )

    $session = Get-VantaSession $InstanceId
    if (-not $session) {
        Write-Error "Session '$InstanceId' not found."
        return
    }

    $session.plan_limit = $Limit
    Save-SessionToDisk $session
}

<#
.SYNOPSIS
    Lists all active sessions.
.EXAMPLE
    Get-VantaSessionList
#>
function Get-VantaSessionList {
    [CmdletBinding()]
    param()

    $sessions = @()
    $pattern = Join-Path $script:SessionStorePath '*.json'
    Get-ChildItem -Path $pattern | ForEach-Object {
        $json = Get-Content $_.FullName -Raw -Encoding UTF8 | ConvertFrom-Json
        $sessions += [PSCustomObject]@{
            InstanceId     = $json.instance_id
            CurrentState   = $json.current_state
            Iteration      = [uint32]$json.iteration_count
            Transition     = [uint64]$json.transition_count
            FilesEdited    = @($json.files_edited).Count
            ContextBytes   = [uint64]$json.context_bytes
            IsFinal        = ($json.definition.states.$($json.current_state).type -eq 'final')
        }
    }
    return $sessions
}

<#
.SYNOPSIS
    Removes a session from disk and cache.
.EXAMPLE
    Remove-VantaSession -InstanceId "campaign-01"
#>
function Remove-VantaSession {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$InstanceId
    )

    $path = Get-SessionFileName $InstanceId
    if (Test-Path -LiteralPath $path) {
        Remove-Item -Path $path -Force
    }
    $script:SessionCache.Remove($InstanceId) | Out-Null
}

# ============================================================================
# Export module members
# ============================================================================

Export-ModuleMember -Function @(
    'New-VantaSession'
    'Get-VantaSession'
    'Update-VantaSessionState'
    'Increment-VantaIteration'
    'Get-VantaUsage'
    'Get-VantaUsageWarning'
    'Test-VantaCheckpoint'
    'Test-VantaFinal'
    'Add-VantaFileEdit'
    'Add-VantaFileRead'
    'Add-VantaContextBytes'
    'Set-VantaPendingApproval'
    'Clear-VantaPendingApproval'
    'Test-VantaPendingApproval'
    'Set-VantaPlanLimit'
    'Get-VantaSessionList'
    'Remove-VantaSession'
)
