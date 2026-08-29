# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_vanta_cli_global_optspecs
    string join \n d/db= v/verbose memory-limit= h/help V/version
end

function __fish_vanta_cli_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_vanta_cli_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_vanta_cli_using_subcommand
    set -l cmd (__fish_vanta_cli_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -s V -l version -d 'Print version'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "put" -d 'Save a key-value pair to persistent memory'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "get" -d 'Retrieve a value from persistent memory'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "list" -d 'List keys and values in a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "rebuild-index" -d 'Rebuild all database indexes (HNSW, text index, derived indexes)'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "audit-index" -d 'Validate text index integrity without repairing'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "repair-text-index" -d 'Repair text index if inconsistencies are detected'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "export" -d 'Export records to a JSON file'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "import" -d 'Import records from a JSON file'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "query" -d 'Execute a structured query (IQL/hybrid)'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "status" -d 'Display database health diagnostics and system status'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "backup" -d 'Create a filesystem-level backup of the database directory'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "restore" -d 'Restore the database from a previously created backup directory'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "doctor" -d 'Run comprehensive health diagnostics on the database'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "inspect" -d 'Inspect a single record showing all fields, vectors, and metadata'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "stats" -d 'Display detailed database statistics in human-readable or JSON format'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "completions" -d 'Generate shell completion scripts'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "search" -d 'Search records semantically across a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "delete" -d 'Delete a record by namespace and key'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "delete-by-filter" -d 'Delete all records in a namespace matching a JSON metadata filter'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "count" -d 'Count records in a namespace, optionally filtered by metadata'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "similar-to-key" -d 'Find records similar to a given key using vector similarity search'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "migrate" -d 'Migrate a database to the latest storage schema version'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "namespace" -d 'Manage namespaces'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "snapshot" -d 'Manage instant filesystem snapshots'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "wal" -d 'Manage the Write-Ahead Log (compact, vacuum)'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "search-multi" -d 'Search across multiple namespaces and merge results by score'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "search-all" -d 'Search across ALL known namespaces and merge results by score'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "server" -d 'Start the HTTP or MCP server wrapper'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -l namespace -d 'Namespace for the key' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -l key -d 'Key to store the value under' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -l payload -d 'Value to store (payload text)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -l vector -d 'Optional vector embedding (comma-separated f32 values)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -l metadata -d 'Optional metadata as a JSON object, e.g. \'{"k":"v","n":1}\'' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand get" -l namespace -d 'Namespace for the key' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand get" -l key -d 'Key to retrieve the value for' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand get" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand get" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand get" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand get" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand list" -l namespace -d 'Namespace to list' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand list" -l limit -d 'Maximum number of records to return' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand list" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand list" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand list" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand list" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand rebuild-index" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand rebuild-index" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand rebuild-index" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand rebuild-index" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -l namespace -d 'Optional namespace to audit (audits all if not specified)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -l json -d 'Output results as JSON'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -l deep -d 'Perform deep structural validation'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand repair-text-index" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand repair-text-index" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand repair-text-index" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand repair-text-index" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand export" -l namespace -d 'Optional namespace to export (exports all if not specified)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand export" -l out -d 'Output file path' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand export" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand export" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand export" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand export" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand import" -l input -d 'Input file path' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand import" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand import" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand import" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand import" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand query" -l limit -d 'Maximum results to return' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand query" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand query" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand query" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand query" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand status" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand status" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand status" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand status" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand backup" -l out -d 'Output directory for the backup (default: vantadb_backups/backup_<timestamp>)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand backup" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand backup" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand backup" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand backup" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand restore" -l input -d 'Path to the backup directory' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand restore" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand restore" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand restore" -l force -d 'Overwrite existing database directory if it exists'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand restore" -l rebuild -d 'Rebuild indexes after restore'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand restore" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand restore" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand doctor" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand doctor" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand doctor" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand doctor" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand inspect" -l namespace -d 'Namespace of the record' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand inspect" -l key -d 'Key of the record to inspect' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand inspect" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand inspect" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand inspect" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand inspect" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand stats" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand stats" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand stats" -l json -d 'Output statistics as JSON'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand stats" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand stats" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand completions" -l shell -d 'Shell type for the completion script' -r -f -a "bash\t'Bash shell completions'
zsh\t'Zsh shell completions'
fish\t'Fish shell completions'
powershell\t'PowerShell shell completions'"
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand completions" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand completions" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand completions" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -l namespace -d 'Namespace to search within' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -l query -d 'Text query for semantic/hybrid search' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -l query-vector -d 'Optional explicit vector query (comma-separated f32 values)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -l limit -d 'Maximum number of results' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -l json -d 'Output in JSON format'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete" -l namespace -d 'Namespace of the record' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete" -l key -d 'Key of the record to delete' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete-by-filter" -l namespace -d 'Namespace to operate on' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete-by-filter" -l filter -d 'JSON filter in MongoDB-like format, e.g. \'{"field": {"$op": value}}\' Operators: $eq, $neq, $gt, $gte, $lt, $lte Example: \'{"status": {"$eq": "inactive"}}\'' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete-by-filter" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete-by-filter" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete-by-filter" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete-by-filter" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand count" -l namespace -d 'Namespace to count records in' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand count" -l filter -d 'Optional JSON filter (same format as delete-by-filter)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand count" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand count" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand count" -l json -d 'Output as raw number only'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand count" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand count" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand similar-to-key" -l namespace -d 'Namespace of the reference record' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand similar-to-key" -l key -d 'Key of the reference record' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand similar-to-key" -l top-k -d 'Number of similar records to return' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand similar-to-key" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand similar-to-key" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand similar-to-key" -l json -d 'Output in JSON format'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand similar-to-key" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand similar-to-key" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and not __fish_seen_subcommand_from plan run check help" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and not __fish_seen_subcommand_from plan run check help" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and not __fish_seen_subcommand_from plan run check help" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and not __fish_seen_subcommand_from plan run check help" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and not __fish_seen_subcommand_from plan run check help" -f -a "plan" -d 'Plan migrations that would be performed'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and not __fish_seen_subcommand_from plan run check help" -f -a "run" -d 'Run migrations to bring formats up to date'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and not __fish_seen_subcommand_from plan run check help" -f -a "check" -d 'Check storage integrity for all formats'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and not __fish_seen_subcommand_from plan run check help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from plan" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from plan" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from plan" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from plan" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from run" -l format -d 'Specific format to migrate (vfile, index, wal, schema, all)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from run" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from run" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from run" -l dry-run -d 'Preview changes without modifying files'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from run" -l force -d 'Skip confirmation prompts'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from run" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from run" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from check" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from check" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from check" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from check" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from help" -f -a "plan" -d 'Plan migrations that would be performed'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from help" -f -a "run" -d 'Run migrations to bring formats up to date'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from help" -f -a "check" -d 'Check storage integrity for all formats'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -f -a "list" -d 'List all namespaces'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -f -a "info" -d 'Show record count and details for a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from list" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from list" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from info" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from info" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "list" -d 'List all namespaces'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "info" -d 'Show record count and details for a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and not __fish_seen_subcommand_from create list help" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and not __fish_seen_subcommand_from create list help" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and not __fish_seen_subcommand_from create list help" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and not __fish_seen_subcommand_from create list help" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and not __fish_seen_subcommand_from create list help" -f -a "create" -d 'Create an instant filesystem snapshot by hard-linking all data files'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and not __fish_seen_subcommand_from create list help" -f -a "list" -d 'List all existing snapshots'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and not __fish_seen_subcommand_from create list help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and __fish_seen_subcommand_from create" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and __fish_seen_subcommand_from create" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and __fish_seen_subcommand_from create" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and __fish_seen_subcommand_from list" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and __fish_seen_subcommand_from list" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create an instant filesystem snapshot by hard-linking all data files'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and __fish_seen_subcommand_from help" -f -a "list" -d 'List all existing snapshots'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand snapshot; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and not __fish_seen_subcommand_from compact vacuum help" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and not __fish_seen_subcommand_from compact vacuum help" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and not __fish_seen_subcommand_from compact vacuum help" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and not __fish_seen_subcommand_from compact vacuum help" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and not __fish_seen_subcommand_from compact vacuum help" -f -a "compact" -d 'Compact the WAL: flush all data, archive the current WAL file, and start a fresh one'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and not __fish_seen_subcommand_from compact vacuum help" -f -a "vacuum" -d 'Remove tombstoned nodes from HNSW and reclaim space'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and not __fish_seen_subcommand_from compact vacuum help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and __fish_seen_subcommand_from compact" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and __fish_seen_subcommand_from compact" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and __fish_seen_subcommand_from compact" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and __fish_seen_subcommand_from compact" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and __fish_seen_subcommand_from vacuum" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and __fish_seen_subcommand_from vacuum" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and __fish_seen_subcommand_from vacuum" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and __fish_seen_subcommand_from vacuum" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and __fish_seen_subcommand_from help" -f -a "compact" -d 'Compact the WAL: flush all data, archive the current WAL file, and start a fresh one'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and __fish_seen_subcommand_from help" -f -a "vacuum" -d 'Remove tombstoned nodes from HNSW and reclaim space'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand wal; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-multi" -l namespaces -d 'Comma-separated list of namespaces to search (e.g. "ns1,ns2,ns3")' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-multi" -l query -d 'Text query for hybrid/lexical search' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-multi" -l query-vector -d 'Optional explicit vector query (comma-separated f32 values)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-multi" -l top-k -d 'Maximum number of results across all namespaces' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-multi" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-multi" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-multi" -l json -d 'Output in JSON format'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-multi" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-multi" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-all" -l query -d 'Text query for hybrid/lexical search' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-all" -l query-vector -d 'Optional explicit vector query (comma-separated f32 values)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-all" -l top-k -d 'Maximum number of results across all namespaces' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-all" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-all" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-all" -l json -d 'Output in JSON format'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-all" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search-all" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -s p -l port -d 'Port for the HTTP server' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -l host -d 'Host for the HTTP server' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -l dashboard-dir -d 'Directory of static files to serve at /dashboard (Vanta Studio web console). When unset, /dashboard responds 404 with a hint' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -l memory-limit -d 'Optional memory limit for the database engine, in bytes. Accepts suffixes: KB, MB, GB (also KiB, MiB, GiB), e.g. `500MB` or `2GB`. Defaults to the value of the VANTADB_MEMORY_LIMIT environment variable' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -l http -d 'Start HTTP server wrapper (default)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -l mcp -d 'Start MCP server wrapper over stdio'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -l require-auth -d 'Force authentication: refuse to start without an API key'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -l allow-insecure -d 'Allow binding a non-loopback host without an API key (dev only). The server logs a prominent security warning and starts unauthenticated'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "put" -d 'Save a key-value pair to persistent memory'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "get" -d 'Retrieve a value from persistent memory'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "list" -d 'List keys and values in a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "rebuild-index" -d 'Rebuild all database indexes (HNSW, text index, derived indexes)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "audit-index" -d 'Validate text index integrity without repairing'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "repair-text-index" -d 'Repair text index if inconsistencies are detected'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "export" -d 'Export records to a JSON file'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "import" -d 'Import records from a JSON file'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "query" -d 'Execute a structured query (IQL/hybrid)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "status" -d 'Display database health diagnostics and system status'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "backup" -d 'Create a filesystem-level backup of the database directory'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "restore" -d 'Restore the database from a previously created backup directory'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "doctor" -d 'Run comprehensive health diagnostics on the database'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "inspect" -d 'Inspect a single record showing all fields, vectors, and metadata'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "stats" -d 'Display detailed database statistics in human-readable or JSON format'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "completions" -d 'Generate shell completion scripts'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "search" -d 'Search records semantically across a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "delete" -d 'Delete a record by namespace and key'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "delete-by-filter" -d 'Delete all records in a namespace matching a JSON metadata filter'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "count" -d 'Count records in a namespace, optionally filtered by metadata'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "similar-to-key" -d 'Find records similar to a given key using vector similarity search'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "migrate" -d 'Migrate a database to the latest storage schema version'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "namespace" -d 'Manage namespaces'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "snapshot" -d 'Manage instant filesystem snapshots'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "wal" -d 'Manage the Write-Ahead Log (compact, vacuum)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "search-multi" -d 'Search across multiple namespaces and merge results by score'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "search-all" -d 'Search across ALL known namespaces and merge results by score'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "server" -d 'Start the HTTP or MCP server wrapper'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status backup restore doctor inspect stats completions search delete delete-by-filter count similar-to-key migrate namespace snapshot wal search-multi search-all server help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and __fish_seen_subcommand_from migrate" -f -a "plan" -d 'Plan migrations that would be performed'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and __fish_seen_subcommand_from migrate" -f -a "run" -d 'Run migrations to bring formats up to date'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and __fish_seen_subcommand_from migrate" -f -a "check" -d 'Check storage integrity for all formats'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and __fish_seen_subcommand_from namespace" -f -a "list" -d 'List all namespaces'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and __fish_seen_subcommand_from namespace" -f -a "info" -d 'Show record count and details for a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and __fish_seen_subcommand_from snapshot" -f -a "create" -d 'Create an instant filesystem snapshot by hard-linking all data files'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and __fish_seen_subcommand_from snapshot" -f -a "list" -d 'List all existing snapshots'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and __fish_seen_subcommand_from wal" -f -a "compact" -d 'Compact the WAL: flush all data, archive the current WAL file, and start a fresh one'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and __fish_seen_subcommand_from wal" -f -a "vacuum" -d 'Remove tombstoned nodes from HNSW and reclaim space'
