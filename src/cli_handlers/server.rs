//! Server/status command handlers — status, server, completions.

use clap::CommandFactory;
use console::Term;

use crate::cli::Cli;
use crate::cli::Shell;
use crate::cli_handlers::fmt::{header_style, info_style};
use crate::cli_handlers::{create_spinner, open_database, print_info, print_warning, MIB};
use crate::config::VantaConfig;
use crate::error::Result;

#[tracing::instrument]
/// Display database health diagnostics and system status
pub fn cmd_status(db_path: &str, verbose: bool) -> Result<()> {
    let path = std::path::Path::new(db_path);
    let term = Term::stdout();

    if !path.exists() {
        let metrics = crate::metrics::operational_metrics_snapshot();
        let _ = term.write_line("");
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╔═══════════════════════════════════════════════════════════╗")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("║               VantaDB Status Dashboard                    ║")
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╠═══════════════════════════════════════════════════════════╣")
        ));
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to("║  📁 Database Information                                  ║")
        ));
        let _ = term.write_line(&format!("║     Path:           {:<38} ║", db_path));
        let _ = term.write_line(&format!(
            "║     Backend:        {:<38} ║",
            "Uninitialized (directory not found)"
        ));
        let _ = term.write_line(&format!("║     Read-only:      {:<38} ║", "Yes (Fallback)"));
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to("║  💾 Storage Statistics                                    ║")
        ));
        let _ = term.write_line(&format!("║     HNSW Nodes:     {:<38} ║", "0 (Empty)"));
        let _ = term.write_line(&format!("║     Cache entries:  {:<38} ║", "0"));
        let _ = term.write_line(&format!("║     Logical size:   {:<38} ║", "0 MB"));
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to("║  ⚡ Performance Metrics                                   ║")
        ));
        let _ = term.write_line(&format!(
            "║     Startup time:   {:<38} ║",
            format!("{} ms", metrics.startup_ms)
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style()
                .apply_to("╚═══════════════════════════════════════════════════════════╝")
        ));
        print_warning(&format!(
            "Database is not initialized. Run a mutation (like `put`) to create it at '{}'",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");

    let engine = open_database(db_path, true)?;
    let stats = engine.get_memory_stats();
    let metrics = crate::metrics::operational_metrics_snapshot();

    spinner.finish_and_clear();

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╔═══════════════════════════════════════════════════════════╗")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("║               VantaDB Status Dashboard                    ║")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╠═══════════════════════════════════════════════════════════╣")
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  📁 Database Information                                  ║")
    ));
    let _ = term.write_line(&format!("║     Path:           {:<38} ║", db_path));
    let _ = term.write_line(&format!(
        "║     Backend:        {:<38} ║",
        format!("{:?}", engine.backend_kind())
    ));
    let _ = term.write_line(&format!(
        "║     Read-only:      {:<38} ║",
        if engine.read_only { "Yes" } else { "No" }
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  💾 Storage Statistics                                    ║")
    ));
    let _ = term.write_line(&format!("║     HNSW Nodes:     {:<38} ║", stats.node_count));
    let _ = term.write_line(&format!(
        "║     Cache entries:  {:<38} ║",
        stats.cache_entries
    ));
    let _ = term.write_line(&format!(
        "║     Logical size:   {:<38} ║",
        format!("{} MB", stats.logical_bytes / MIB)
    ));
    if let Some(rss) = stats.physical_rss {
        let _ = term.write_line(&format!(
            "║     Physical RSS:   {:<38} ║",
            format!("{} MB", rss / MIB)
        ));
    }
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  ⚡ Performance Metrics                                   ║")
    ));
    let _ = term.write_line(&format!(
        "║     Startup time:   {:<38} ║",
        format!("{} ms", metrics.startup_ms)
    ));
    let _ = term.write_line(&format!(
        "║     WAL replay:     {:<38} ║",
        format!(
            "{} ms ({} records)",
            metrics.wal_replay_ms, metrics.wal_records_replayed
        )
    ));
    let _ = term.write_line(&format!(
        "║     ANN rebuild:    {:<38} ║",
        format!("{} ms", metrics.ann_rebuild_ms)
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to("║  📦 Data Operations                                       ║")
    ));
    let _ = term.write_line(&format!(
        "║     Records exported:{:<37} ║",
        metrics.records_exported
    ));
    let _ = term.write_line(&format!(
        "║     Records imported:{:<37} ║",
        metrics.records_imported
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╚═══════════════════════════════════════════════════════════╝")
    ));

    if verbose {
        let _ = term.write_line("");
        print_info("Verbose mode: Extended metrics available via VantaOperationalMetrics API");
    }

    Ok(())
}

#[tracing::instrument]
/// Start the HTTP or MCP server wrapper for the database
pub fn cmd_server(
    db_path: &str,
    http: bool,
    mcp: bool,
    port: Option<u16>,
    host: Option<String>,
    require_auth: bool,
    _verbose: bool,
) -> Result<()> {
    let mcp_mode = mcp && !http;

    if mcp_mode {
        return cmd_server_mcp(db_path, port, host, require_auth);
    }

    #[cfg(feature = "server")]
    {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            crate::error::VantaError::RuntimeError(format!("Failed to start tokio runtime: {e}"))
        })?;

        rt.block_on(cmd_server_http(db_path, port, host, require_auth))
    }

    #[cfg(not(feature = "server"))]
    {
        Err(crate::error::VantaError::CliError(
            "HTTP server requires the 'server' feature. Rebuild with: cargo build --features server"
                .to_string(),
        ))
    }
}

#[cfg(feature = "server")]
async fn cmd_server_http(
    db_path: &str,
    port: Option<u16>,
    host: Option<String>,
    require_auth: bool,
) -> Result<()> {
    let config = VantaConfig {
        storage_path: db_path.to_string(),
        port: port.unwrap_or(8080),
        host: host.unwrap_or_else(|| "127.0.0.1".to_string()),
        require_auth,
        ..Default::default()
    };

    crate::cli_server::run(config).await
}

fn cmd_server_mcp(
    db_path: &str,
    port: Option<u16>,
    host: Option<String>,
    require_auth: bool,
) -> Result<()> {
    use crate::error::VantaError;

    let binary_name = "vantadb-server";
    let exe_name = if cfg!(windows) {
        format!("{}.exe", binary_name)
    } else {
        binary_name.to_string()
    };

    let build_cmd = |binary: &std::path::Path| -> std::process::Command {
        let mut cmd = std::process::Command::new(binary);
        cmd.env("VANTA_DB", db_path);
        if let Some(p) = port {
            cmd.env("VANTADB_PORT", p.to_string());
        }
        if let Some(ref h) = host {
            cmd.env("VANTADB_HOST", h);
        }
        if require_auth {
            cmd.env("VANTADB_REQUIRE_AUTH", "true");
        }
        cmd.arg("--mcp");
        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());
        cmd
    };

    let mut child = match build_cmd(std::path::Path::new(&exe_name)).spawn() {
        Ok(c) => c,
        Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => {
            if let Ok(mut current_exe) = std::env::current_exe() {
                current_exe.set_file_name(&exe_name);
                if current_exe.exists() {
                    build_cmd(&current_exe).spawn().map_err(|e| {
                        VantaError::CliError(format!(
                            "Failed to start vantadb-server from {}: {e}",
                            current_exe.display()
                        ))
                    })?
                } else {
                    return Err(VantaError::CliError(format!(
                        "vantadb-server binary not found. \
                         Searched PATH for '{}' and CLI directory for '{}'. \
                         The MCP server requires the vantadb-server binary (compiled with the 'server' feature). \
                         Install it via 'cargo build --bin vantadb-server' or place it alongside this binary.",
                        exe_name,
                        current_exe.display()
                    )));
                }
            } else {
                return Err(VantaError::CliError(format!(
                    "vantadb-server binary '{}' not found in PATH. \
                     Current executable path could not be determined. \
                     Ensure vantadb-server is installed and available in PATH.",
                    exe_name
                )));
            }
        }
        Err(e) => {
            return Err(VantaError::CliError(format!(
                "Failed to spawn vantadb-server process (db_path={}): {e}",
                db_path
            )));
        }
    };

    let status = child.wait().map_err(|e| {
        VantaError::CliError(format!(
            "Error waiting for vantadb-server process (db_path={}): {e}",
            db_path
        ))
    })?;

    if !status.success() {
        if let Some(code) = status.code() {
            std::process::exit(code);
        } else {
            return Err(VantaError::CliError(format!(
                "vantadb-server terminated by signal (db_path={})",
                db_path
            )));
        }
    }

    Ok(())
}

#[tracing::instrument]
/// Generate shell completion scripts for the given shell type
pub fn cmd_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let shell: clap_complete::Shell = shell.into();
    clap_complete::generate(shell, &mut cmd, "vanta-cli", &mut std::io::stdout());
}
