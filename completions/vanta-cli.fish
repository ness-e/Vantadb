# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_vanta_cli_global_optspecs
    string join \n d/db= v/verbose h/help V/version
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
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "completions" -d 'Generate shell completion scripts'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "search" -d 'Search records semantically across a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "delete" -d 'Delete a record by namespace and key'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "migrate" -d 'Migrate a database to the latest storage schema version'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "namespace" -d 'Manage namespaces'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "server" -d 'Start the HTTP or MCP server wrapper'
complete -c vanta-cli -n "__fish_vanta_cli_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -l namespace -d 'Namespace for the key' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -l key -d 'Key to store the value under' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -l payload -d 'Value to store (payload text)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -l vector -d 'Optional vector embedding (comma-separated f32 values)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand put" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand get" -l namespace -d 'Namespace for the key' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand get" -l key -d 'Key to retrieve the value for' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand get" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand get" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand get" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand list" -l namespace -d 'Namespace to list' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand list" -l limit -d 'Maximum number of records to return' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand list" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand list" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand list" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand rebuild-index" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand rebuild-index" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand rebuild-index" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -l namespace -d 'Optional namespace to audit (audits all if not specified)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -l json -d 'Output results as JSON'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -l deep -d 'Perform deep structural validation'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand audit-index" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand repair-text-index" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand repair-text-index" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand repair-text-index" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand export" -l namespace -d 'Optional namespace to export (exports all if not specified)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand export" -l out -d 'Output file path' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand export" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand export" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand export" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand import" -l input -d 'Input file path' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand import" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand import" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand import" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand query" -l limit -d 'Maximum results to return' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand query" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand query" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand query" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand status" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand status" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand status" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand completions" -l shell -d 'Shell type for the completion script' -r -f -a "bash\t'Bash shell completions'
zsh\t'Zsh shell completions'
fish\t'Fish shell completions'
powershell\t'PowerShell shell completions'"
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand completions" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand completions" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -l namespace -d 'Namespace to search within' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -l query -d 'Text query for semantic/hybrid search' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -l query-vector -d 'Optional explicit vector query (comma-separated f32 values)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -l limit -d 'Maximum number of results' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -l json -d 'Output in JSON format'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand search" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete" -l namespace -d 'Namespace of the record' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete" -l key -d 'Key of the record to delete' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand delete" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate" -l target -d 'Path to the database directory to migrate' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate" -l format -d 'Specific format to migrate (vfile, index, wal, schema, all)' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate" -l dry-run -d 'Report what would be migrated without writing'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate" -l force -d 'Skip confirmation prompts'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand migrate" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -f -a "list" -d 'List all namespaces'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -f -a "info" -d 'Show record count and details for a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and not __fish_seen_subcommand_from list info help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from list" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from info" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "list" -d 'List all namespaces'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "info" -d 'Show record count and details for a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand namespace; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -s p -l port -d 'Port for the HTTP server' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -l host -d 'Host for the HTTP server' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -s d -l db -d 'Path to the database directory. Defaults to the value of the VANTA_DB environment variable, or \'./db\' if neither is set' -r
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -l http -d 'Start HTTP server wrapper (default)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -l mcp -d 'Start MCP server wrapper over stdio'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -s v -l verbose -d 'Enable verbose output'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand server" -s h -l help -d 'Print help'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "put" -d 'Save a key-value pair to persistent memory'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "get" -d 'Retrieve a value from persistent memory'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "list" -d 'List keys and values in a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "rebuild-index" -d 'Rebuild all database indexes (HNSW, text index, derived indexes)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "audit-index" -d 'Validate text index integrity without repairing'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "repair-text-index" -d 'Repair text index if inconsistencies are detected'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "export" -d 'Export records to a JSON file'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "import" -d 'Import records from a JSON file'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "query" -d 'Execute a structured query (IQL/hybrid)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "status" -d 'Display database health diagnostics and system status'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "completions" -d 'Generate shell completion scripts'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "search" -d 'Search records semantically across a namespace'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "delete" -d 'Delete a record by namespace and key'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "migrate" -d 'Migrate a database to the latest storage schema version'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "namespace" -d 'Manage namespaces'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "server" -d 'Start the HTTP or MCP server wrapper'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and not __fish_seen_subcommand_from put get list rebuild-index audit-index repair-text-index export import query status completions search delete migrate namespace server help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and __fish_seen_subcommand_from namespace" -f -a "list" -d 'List all namespaces'
complete -c vanta-cli -n "__fish_vanta_cli_using_subcommand help; and __fish_seen_subcommand_from namespace" -f -a "info" -d 'Show record count and details for a namespace'
