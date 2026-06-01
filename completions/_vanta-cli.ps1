
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'vanta-cli' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'vanta-cli'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'vanta-cli' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('put', 'put', [CompletionResultType]::ParameterValue, 'Save a key-value pair to persistent memory')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Retrieve a value from persistent memory')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List keys and values in a namespace')
            [CompletionResult]::new('rebuild-index', 'rebuild-index', [CompletionResultType]::ParameterValue, 'Rebuild all database indexes (HNSW, text index, derived indexes)')
            [CompletionResult]::new('audit-index', 'audit-index', [CompletionResultType]::ParameterValue, 'Validate text index integrity without repairing')
            [CompletionResult]::new('repair-text-index', 'repair-text-index', [CompletionResultType]::ParameterValue, 'Repair text index if inconsistencies are detected')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export records to a JSON file')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import records from a JSON file')
            [CompletionResult]::new('query', 'query', [CompletionResultType]::ParameterValue, 'Execute a structured query (IQL/hybrid)')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Display database health diagnostics and system status')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'vanta-cli;put' {
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Namespace for the key')
            [CompletionResult]::new('--key', '--key', [CompletionResultType]::ParameterName, 'Key to store the value under')
            [CompletionResult]::new('--payload', '--payload', [CompletionResultType]::ParameterName, 'Value to store (payload text)')
            [CompletionResult]::new('--vector', '--vector', [CompletionResultType]::ParameterName, 'Optional vector embedding (comma-separated f32 values)')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'vanta-cli;get' {
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Namespace for the key')
            [CompletionResult]::new('--key', '--key', [CompletionResultType]::ParameterName, 'Key to retrieve the value for')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'vanta-cli;list' {
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Namespace to list')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'Maximum number of records to return')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'vanta-cli;rebuild-index' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'vanta-cli;audit-index' {
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Optional namespace to audit (audits all if not specified)')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output results as JSON')
            [CompletionResult]::new('--deep', '--deep', [CompletionResultType]::ParameterName, 'Perform deep structural validation')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'vanta-cli;repair-text-index' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'vanta-cli;export' {
            [CompletionResult]::new('--namespace', '--namespace', [CompletionResultType]::ParameterName, 'Optional namespace to export (exports all if not specified)')
            [CompletionResult]::new('--out', '--out', [CompletionResultType]::ParameterName, 'Output file path')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'vanta-cli;import' {
            [CompletionResult]::new('--input', '--input', [CompletionResultType]::ParameterName, 'Input file path')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'vanta-cli;query' {
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'Maximum results to return')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'vanta-cli;status' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'vanta-cli;completions' {
            [CompletionResult]::new('--shell', '--shell', [CompletionResultType]::ParameterName, 'Shell type for the completion script')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('--db', '--db', [CompletionResultType]::ParameterName, 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or ''./db'' if neither is set')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'vanta-cli;help' {
            [CompletionResult]::new('put', 'put', [CompletionResultType]::ParameterValue, 'Save a key-value pair to persistent memory')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Retrieve a value from persistent memory')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List keys and values in a namespace')
            [CompletionResult]::new('rebuild-index', 'rebuild-index', [CompletionResultType]::ParameterValue, 'Rebuild all database indexes (HNSW, text index, derived indexes)')
            [CompletionResult]::new('audit-index', 'audit-index', [CompletionResultType]::ParameterValue, 'Validate text index integrity without repairing')
            [CompletionResult]::new('repair-text-index', 'repair-text-index', [CompletionResultType]::ParameterValue, 'Repair text index if inconsistencies are detected')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export records to a JSON file')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import records from a JSON file')
            [CompletionResult]::new('query', 'query', [CompletionResultType]::ParameterValue, 'Execute a structured query (IQL/hybrid)')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Display database health diagnostics and system status')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'vanta-cli;help;put' {
            break
        }
        'vanta-cli;help;get' {
            break
        }
        'vanta-cli;help;list' {
            break
        }
        'vanta-cli;help;rebuild-index' {
            break
        }
        'vanta-cli;help;audit-index' {
            break
        }
        'vanta-cli;help;repair-text-index' {
            break
        }
        'vanta-cli;help;export' {
            break
        }
        'vanta-cli;help;import' {
            break
        }
        'vanta-cli;help;query' {
            break
        }
        'vanta-cli;help;status' {
            break
        }
        'vanta-cli;help;completions' {
            break
        }
        'vanta-cli;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
