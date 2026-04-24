#![allow(dead_code)]

use console::{style, Emoji};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use sysinfo::System;

pub mod sift_loader;

// ─── Global State for Final Summary & Progress ────────────────

static TEST_RESULTS: Mutex<Vec<TestSummary>> = Mutex::new(Vec::new());
static MULTI_PROGRESS: OnceLock<MultiProgress> = OnceLock::new();
static GLOBAL_BAR: OnceLock<ProgressBar> = OnceLock::new();

#[derive(Clone)]
struct TestSummary {
    name: String,
    success: bool,
    duration: std::time::Duration,
    mem_delta_mb: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestMetric {
    pub block_name: String,
    pub duration_secs: f64,
    pub ram_usage_mb: f64,
    pub current_ram_mb: f64,
    pub timestamp: String,
}

// ─── Internal Helpers ────────────────────────────────────────

fn get_multi_progress() -> &'static MultiProgress {
    MULTI_PROGRESS.get_or_init(|| MultiProgress::new())
}

fn get_global_bar() -> &'static ProgressBar {
    GLOBAL_BAR.get_or_init(|| {
        let pb = ProgressBar::new(10);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} \x1b[1;37m[{elapsed_precise}]\x1b[0m [{bar:40.cyan/blue}] \x1b[1;36m{pos}/{len}\x1b[0m \x1b[37m{msg}\x1b[0m")
            .expect("Invalid progress template")
            .progress_chars("█▉▊▋▌▍▎▏  "));
        get_multi_progress().add(pb)
    })
}

// ─── Terminal Reporter (Aesthetics) ──────────────────────────

pub struct TerminalReporter;

impl TerminalReporter {
    pub fn suite_banner(name: &str, total_tests: u64) {
        let width = 60;
        let line = "═".repeat(width);
        println!("\n{}", style(format!("╔{}╗", line)).yellow().bold());
        println!(
            "{} {:^width$} {}",
            style("║").yellow().bold(),
            style(name).white().bold(),
            style("║").yellow().bold(),
            width = width
        );
        println!("{}\n", style(format!("╚{}╝", line)).yellow().bold());

        let bar = get_global_bar();
        bar.set_length(total_tests);
        bar.set_message("Initializing test suites...");
    }

    pub fn block_header(title: &str) {
        let width = 50;
        let bar = "━".repeat(width);
        println!("\n{}", style(format!("┏{}┓", bar)).cyan().bold());
        println!(
            "{} {:^width$} {}",
            style("┃").cyan().bold(),
            style(title).white().bold(),
            style("┃").cyan().bold(),
            width = width
        );
        println!("{}\n", style(format!("┗{}┛", bar)).cyan().bold());
    }

    pub fn sub_step(msg: &str) {
        println!("  {} {}", style("➜").cyan().bold(), style(msg).dim());
    }

    pub fn success(msg: &str) {
        println!("  {} {}", Emoji("✅", "OK"), style(msg).green());
    }

    pub fn info(msg: &str) {
        println!("  {} {}", Emoji("ℹ️ ", "i"), style(msg).blue());
    }

    pub fn warning(msg: &str) {
        println!("  {} {}", Emoji("⚠️ ", "W"), style(msg).yellow());
    }

    pub fn create_progress(len: u64, msg: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {spinner:.cyan} {msg:.white} [{bar:25.cyan/blue}] {pos}/{len}")
                .expect("Invalid progress template")
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        pb.set_message(msg.to_string());
        get_multi_progress().add(pb)
    }

    pub fn print_certification_summary() {
        let bar = get_global_bar();
        bar.finish_with_message("All tests completed.");

        let results = TEST_RESULTS.lock().unwrap();
        if results.is_empty() {
            return;
        }

        println!("\n\x1b[1;37m┌{}┐", "─".repeat(74));
        println!("│{:^74}│", "VANTA DB - OPERATIONAL INTEGRITY REPORT");
        println!(
            "├{}┬────────────┬────────────┬────────────┤",
            "─".repeat(34)
        );
        println!(
            "│ {:<32} │ {:<10} │ {:<10} │ {:<10} │",
            "Component / Test Case", "Status", "Time", "RAM Δ"
        );
        println!(
            "├{}┼────────────┼────────────┼────────────┤",
            "─".repeat(34)
        );

        let mut total_time = std::time::Duration::from_secs(0);
        let mut total_passed = 0;

        for res in results.iter() {
            let status = if res.success {
                "\x1b[32mPASS\x1b[0m"
            } else {
                "\x1b[31mFAIL\x1b[0m"
            };
            println!(
                "│ {:<32} │ {:^20} │ {:>10.2?} │ {:>7.1} MB │",
                res.name, status, res.duration, res.mem_delta_mb
            );
            total_time += res.duration;
            if res.success {
                total_passed += 1;
            }
        }

        println!(
            "├{}┴────────────┴────────────┴────────────┤",
            "─".repeat(34)
        );
        let summary_text = format!(
            "TOTAL: {}/{} PASSED | AGGREGATE TIME: {:?}",
            total_passed,
            results.len(),
            total_time
        );
        println!("│ {:<72} │", summary_text);
        println!("└{}┘\x1b[0m\n", "─".repeat(74));
    }
}

// ─── VantaHarness (Reporting & Metrics) ──────────────────────────

pub struct VantaHarness {
    sys: System,
    pid: sysinfo::Pid,
    _start_time: Instant,
    start_mem: u64,
    test_name: String,
}

impl VantaHarness {
    const REPORT_FILE: &'static str = "vanta_certification.json";

    pub fn new(test_name: &str) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let pid = sysinfo::get_current_pid().expect("Failed to get PID");
        sys.refresh_process(pid);
        let start_mem = sys.process(pid).map(|p| p.memory()).unwrap_or(0);

        Self {
            sys,
            pid,
            _start_time: Instant::now(),
            start_mem,
            test_name: test_name.to_string(),
        }
    }

    pub fn execute<F, R>(&mut self, block_name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        TerminalReporter::block_header(block_name);
        let t0 = Instant::now();
        let result = f();
        let duration = t0.elapsed();

        self.sys.refresh_process(self.pid);
        let end_mem = self.sys.process(self.pid).map(|p| p.memory()).unwrap_or(0);
        let mem_usage_kb = end_mem.saturating_sub(self.start_mem);

        let metric = TestMetric {
            block_name: format!("{}: {}", self.test_name, block_name),
            duration_secs: duration.as_secs_f64(),
            ram_usage_mb: mem_usage_kb as f64 / 1024.0,
            current_ram_mb: end_mem as f64 / 1024.0,
            timestamp: chrono::Local::now().to_rfc3339(),
        };

        self.log_metric(metric);
        result
    }

    /// Async-aware variant of `execute` for use inside `#[tokio::test]`.
    /// Prevents the `futures::executor::block_on` + tokio reactor deadlock.
    pub async fn execute_async<F, Fut, R>(&mut self, block_name: &str, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        TerminalReporter::block_header(block_name);
        let t0 = Instant::now();
        let result = f().await;
        let duration = t0.elapsed();

        self.sys.refresh_process(self.pid);
        let end_mem = self.sys.process(self.pid).map(|p| p.memory()).unwrap_or(0);
        let mem_usage_kb = end_mem.saturating_sub(self.start_mem);

        let metric = TestMetric {
            block_name: format!("{}: {}", self.test_name, block_name),
            duration_secs: duration.as_secs_f64(),
            ram_usage_mb: mem_usage_kb as f64 / 1024.0,
            current_ram_mb: end_mem as f64 / 1024.0,
            timestamp: chrono::Local::now().to_rfc3339(),
        };

        self.log_metric(metric);
        result
    }

    fn log_metric(&self, metric: TestMetric) {
        if let Ok(json) = serde_json::to_string(&metric) {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(Self::REPORT_FILE)
            {
                let _ = writeln!(file, "{}", json);
            }
        }
    }
}

// ─── Atomic VantaSession (To prevent Interleaving) ───────────

pub struct VantaSession {
    name: String,
    steps: Vec<String>,
    start_time: Instant,
    start_mem: u64,
}

impl VantaSession {
    pub fn begin(name: &str) -> Self {
        let bar = get_global_bar();
        bar.set_message(format!("Running: {}", name));

        let mut sys = System::new_all();
        sys.refresh_all();
        let pid = sysinfo::get_current_pid().unwrap();
        sys.refresh_process(pid);
        let start_mem = sys.process(pid).map(|p| p.memory()).unwrap_or(0);

        Self {
            name: name.to_string(),
            steps: Vec::new(),
            start_time: Instant::now(),
            start_mem,
        }
    }

    pub fn step(&mut self, msg: &str) {
        self.steps.push(format!(
            "  {} {}",
            style("➜").cyan().bold(),
            style(msg).dim()
        ));
    }

    pub fn success(&mut self, msg: &str) {
        self.steps
            .push(format!("  {} {}", Emoji("✅", "OK"), style(msg).green()));
    }

    pub fn finish(self, success: bool) {
        let duration = self.start_time.elapsed();
        let mut sys = System::new_all();
        let pid = sysinfo::get_current_pid().unwrap();
        sys.refresh_process(pid);
        let end_mem = sys.process(pid).map(|p| p.memory()).unwrap_or(0);
        let mem_delta = (end_mem as f64 - self.start_mem as f64) / 1024.0;

        let mut output = String::new();
        let bar = "━".repeat(50);
        output.push_str(&format!("\n\x1b[1;36m┏{}┓\n", bar));
        output.push_str(&format!("┃ {:^50} ┃\n", self.name));
        output.push_str(&format!("┗{}┛\x1b[0m\n", bar));

        for step in &self.steps {
            output.push_str(step);
            output.push_str("\n");
        }

        if success {
            output.push_str(&format!(
                "\n  \x1b[1;32mRESULT: SUCCESS ({:?}, {:.1} MB)\x1b[0m\n",
                duration, mem_delta
            ));
        } else {
            output.push_str(&format!(
                "\n  \x1b[1;31mRESULT: FAILED ({:?})\x1b[0m\n",
                duration
            ));
        }

        let bar = get_global_bar();
        bar.println(output);
        bar.inc(1);

        TEST_RESULTS.lock().unwrap().push(TestSummary {
            name: self.name,
            success,
            duration,
            mem_delta_mb: mem_delta,
        });
    }
}
