//! # VantaDB Professional Console
//!
//! Centralized, visually-rich terminal output system.
//! Brand colors: Vanta Black (`#050505`) · Rust Orange (`#CE422B`)
//!
//! ## Usage
//! ```rust
//! use vantadb::console;
//! console::init_logging();
//! console::print_banner();
//! console::ok("RocksDB opened", Some("4 column families"));
//! ```

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// ─── Banner ────────────────────────────────────────────────────────────────

/// Print the VantaDB ASCII banner to stdout.
/// Uses Rust Orange for the name and dim white for the tagline.
pub fn print_banner() {
    let border = style("═").color256(166).to_string(); // Rust Orange border
    let b = border.repeat(50);

    eprintln!();
    eprintln!("  {}", style(&b).color256(166));
    eprintln!("  {}  {}  {}",
        style("║").color256(166),
        style("  ⚡  V A N T A D B   v0.1.0  ⚡  ").bold().color256(166),
        style("║").color256(166),
    );
    eprintln!("  {}  {}  {}",
        style("║").color256(166),
        style("  Embedded Multimodal Database Engine     ").dim().white(),
        style("║").color256(166),
    );
    eprintln!("  {}  {}  {}",
        style("║").color256(166),
        style("  Vector · Graph · Relational in one core ").dim().white(),
        style("║").color256(166),
    );
    eprintln!("  {}", style(&b).color256(166));
    eprintln!();
}

// ─── Logging Initialization ─────────────────────────────────────────────────

/// Initialize `tracing-subscriber` with colored, level-filtered output.
/// Respects the `RUST_LOG` env var (defaults to `info`).
pub fn init_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_ansi(true)
        .compact()
        .init();
}

// ─── Status Lines ───────────────────────────────────────────────────────────

/// `[✔] <label>  (<detail>)`  — success indicator
pub fn ok(label: &str, detail: Option<&str>) {
    let check = style("[✔]").green().bold();
    let lbl   = style(label).white().bold();
    match detail {
        Some(d) => eprintln!("  {}  {:<36} {}", check, lbl, style(d).dim()),
        None    => eprintln!("  {}  {}", check, lbl),
    }
}

/// `[→] <label>  (<detail>)` — progress / in-flight indicator
pub fn progress(label: &str, detail: Option<&str>) {
    let arrow = style("[→]").cyan().bold();
    let lbl   = style(label).white();
    match detail {
        Some(d) => eprintln!("  {}  {:<36} {}", arrow, lbl, style(d).dim()),
        None    => eprintln!("  {}  {}", arrow, lbl),
    }
}

/// `[⚠] <label>  (<detail>)` — warning indicator
pub fn warn(label: &str, detail: Option<&str>) {
    let ico = style("[⚠]").yellow().bold();
    let lbl = style(label).yellow();
    match detail {
        Some(d) => eprintln!("  {}  {:<36} {}", ico, lbl, style(d).dim()),
        None    => eprintln!("  {}  {}", ico, lbl),
    }
}

/// `[✗] <label>  (<detail>)` — error indicator
pub fn error(label: &str, detail: Option<&str>) {
    let ico = style("[✗]").red().bold();
    let lbl = style(label).red().bold();
    match detail {
        Some(d) => eprintln!("  {}  {:<36} {}", ico, lbl, style(d).dim()),
        None    => eprintln!("  {}  {}", ico, lbl),
    }
}

// ─── Section Dividers ───────────────────────────────────────────────────────

/// Print a labeled section header with Rust Orange separator
pub fn section(title: &str) {
    let line = style("─").color256(166).to_string().repeat(48);
    eprintln!();
    eprintln!("  {}  {}  {}",
        style("┤").color256(166),
        style(title).color256(166).bold(),
        style("├").color256(166),
    );
    eprintln!("  {}", style(&line).color256(166).dim());
}

/// Print a simple separator line
pub fn separator() {
    eprintln!("  {}", style("─".repeat(50)).color256(166).dim());
}

// ─── Startup Summary ────────────────────────────────────────────────────────

/// Print the hardware + memory startup summary block
pub fn print_startup_summary(
    profile: &str,
    instructions: &str,
    total_memory_mb: u64,
    rocksdb_budget_mb: u64,
    backend_mode: &str,
    data_dir: &str,
) {
    section("System Configuration");
    eprintln!("  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("Hardware:").dim(),
        style(profile).bold().white(),
    );
    eprintln!("  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("Instructions:").dim(),
        style(instructions).bold().white(),
    );
    eprintln!("  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("Total Memory:").dim(),
        style(format!("{} MB", total_memory_mb)).bold().white(),
    );
    eprintln!("  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("RocksDB Budget:").dim(),
        style(format!("{} MB", rocksdb_budget_mb)).color256(166).bold(),
    );
    eprintln!("  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("HNSW Backend:").dim(),
        style(backend_mode).bold().white(),
    );
    eprintln!("  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("Data Dir:").dim(),
        style(data_dir).dim().white(),
    );
    separator();
    eprintln!();
}

// ─── Ready Message ──────────────────────────────────────────────────────────

/// Print the final "server ready" line
pub fn print_ready(addr: &str) {
    eprintln!();
    eprintln!("  {}  {} {}",
        style("[→]").color256(166).bold(),
        style("Listening on").white(),
        style(addr).color256(166).bold().underlined(),
    );
    eprintln!("  {}  {}",
        style("   ").dim(),
        style("VantaDB is ready for connections.").dim(),
    );
    eprintln!();
}

// ─── Progress Bars ──────────────────────────────────────────────────────────

/// Create a styled progress bar for long operations (insert batch, indexing, etc.)
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.color256(166)} [{bar:40.color256(166)/dim}] {pos}/{len} {msg}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("█▉▊▋▌▍▎▏ ")
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Create a simple spinner for indeterminate operations
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("  {spinner:.color256(166)} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

// ─── Utilities ──────────────────────────────────────────────────────────────

/// Format bytes as human-readable string (B / KB / MB / GB)
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_024 * KB;
    const GB: u64 = 1_024 * MB;
    match bytes {
        b if b >= GB => format!("{:.1} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.1} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.1} KB", b as f64 / KB as f64),
        b            => format!("{} B",  b),
    }
}

/// Format milliseconds as human-readable duration (µs / ms / s)
pub fn format_duration_ms(ms: u128) -> String {
    match ms {
        t if t < 1     => format!("{}µs", t * 1000),
        t if t < 1_000 => format!("{}ms", t),
        t              => format!("{:.2}s", t as f64 / 1000.0),
    }
}
