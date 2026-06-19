#![allow(dead_code)]

#[cfg(feature = "cli")]
use console::{style, Emoji};
use fs2::FileExt;
#[cfg(feature = "cli")]
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Seek, Write};
use std::time::Instant;
use sysinfo::System;

pub mod sift_loader;

#[cfg(feature = "cli")]
thread_local! {
    static MULTI_PROGRESS: std::cell::RefCell<Option<MultiProgress>> = const { std::cell::RefCell::new(None) };
    static GLOBAL_BAR: std::cell::RefCell<Option<ProgressBar>> = const { std::cell::RefCell::new(None) };
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestMetric {
    pub schema_version: u32,
    pub block_name: String,
    pub duration_secs: f64,
    pub process_memory_delta_mb: f64,
    pub process_memory_current_mb: f64,
    pub process_virtual_memory_delta_mb: f64,
    pub process_virtual_memory_current_mb: f64,
    pub memory_source: String,
    pub memory_confidence: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestRunReport {
    pub timestamp: String,
    pub metrics: std::collections::HashMap<String, TestMetric>,
}

#[derive(Clone, Copy, Debug, Default)]
struct ProcessMemorySample {
    used_bytes: u64,
    virtual_bytes: u64,
}

fn bytes_to_mb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn sample_process_memory(sys: &mut System, pid: sysinfo::Pid) -> ProcessMemorySample {
    sys.refresh_process(pid);
    match sys.process(pid) {
        Some(process) => ProcessMemorySample {
            used_bytes: process.memory(),
            virtual_bytes: process.virtual_memory(),
        },
        None => ProcessMemorySample::default(),
    }
}

// ─── Internal Helpers ────────────────────────────────────────

#[cfg(feature = "cli")]
fn with_multi_progress<F, R>(f: F) -> R
where
    F: FnOnce(&mut MultiProgress) -> R,
{
    MULTI_PROGRESS.with(|cell| {
        let mut mp = cell.borrow_mut();
        let mp = mp.get_or_insert_with(|| {
            // Stderr IS a TTY under `cargo test --nocapture`, unlike stdout which
            // is captured. Routing here ensures indicatif can update lines in-place
            // instead of printing a new line on every tick / set_message call.
            MultiProgress::with_draw_target(ProgressDrawTarget::stderr_with_hz(10))
        });
        f(mp)
    })
}

#[cfg(feature = "cli")]
fn with_global_bar<F, R>(f: F) -> R
where
    F: FnOnce(&mut ProgressBar) -> R,
{
    GLOBAL_BAR.with(|cell| {
        let mut bar = cell.borrow_mut();
        let bar = bar.get_or_insert_with(|| {
            let pb =
                ProgressBar::with_draw_target(Some(10), ProgressDrawTarget::stderr_with_hz(10));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} \x1b[1;37m[{elapsed_precise}]\x1b[0m \
                         [{bar:40.cyan/blue}] \x1b[1;36m{pos}/{len}\x1b[0m \
                         \x1b[37m{msg}\x1b[0m",
                    )
                    .expect("Invalid progress template")
                    .progress_chars("█▉▊▋▌▍▎▏  "),
            );
            // steady_tick makes indicatif own the render loop; without it every
            // .set_message() call triggers a fresh print when stdout isn't a TTY.
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            with_multi_progress(|mp| mp.add(pb))
        });
        f(bar)
    })
}

// ─── Terminal Reporter (Aesthetics) ──────────────────────────

pub struct TerminalReporter;

#[cfg(feature = "cli")]
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

        with_global_bar(|bar| {
            bar.set_length(total_tests);
            bar.set_message("Initializing test suites...");
        });
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
        let pb = ProgressBar::with_draw_target(Some(len), ProgressDrawTarget::stderr_with_hz(10));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {spinner:.cyan} {msg:.white} [{bar:30.cyan/blue}] {pos}/{len} ({eta})")
                .expect("Invalid progress template")
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        with_multi_progress(|mp| mp.add(pb))
    }

    pub fn print_certification_summary() {
        with_global_bar(|bar| {
            bar.finish_with_message("All tests completed.");
        });
    }
}

// Stub impl when CLI feature is absent — methods become no-ops so tests compile.
#[cfg(not(feature = "cli"))]
pub struct ProgressBar;

#[cfg(not(feature = "cli"))]
impl ProgressBar {
    pub fn inc(&self, _amount: u64) {}
    pub fn finish_and_clear(&self) {}
}

#[cfg(not(feature = "cli"))]
impl TerminalReporter {
    pub fn suite_banner(_name: &str, _total_tests: u64) {}
    pub fn block_header(_title: &str) {}
    pub fn sub_step(_msg: &str) {}
    pub fn success(_msg: &str) {}
    pub fn info(_msg: &str) {}
    pub fn warning(_msg: &str) {}
    pub fn create_progress(_len: u64, _msg: &str) -> ProgressBar {
        ProgressBar
    }
    pub fn print_certification_summary() {}
}

// ─── VantaHarness (Reporting & Metrics) ──────────────────────────

pub struct VantaHarness {
    sys: System,
    pid: sysinfo::Pid,
    _start_time: Instant,
    start_memory: ProcessMemorySample,
    test_name: String,
}

impl VantaHarness {
    const REPORT_FILE: &'static str = "vanta_certification.json";
    const REPORT_FILE_ENV: &'static str = "VANTA_CERT_REPORT";

    pub fn new(test_name: &str) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let pid = sysinfo::get_current_pid().expect("Failed to get PID");
        let start_memory = sample_process_memory(&mut sys, pid);

        Self {
            sys,
            pid,
            _start_time: Instant::now(),
            start_memory,
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

        let end_memory = sample_process_memory(&mut self.sys, self.pid);

        let metric = TestMetric {
            schema_version: 1,
            block_name: format!("{}: {}", self.test_name, block_name),
            duration_secs: duration.as_secs_f64(),
            process_memory_delta_mb: bytes_to_mb(
                end_memory
                    .used_bytes
                    .saturating_sub(self.start_memory.used_bytes),
            ),
            process_memory_current_mb: bytes_to_mb(end_memory.used_bytes),
            process_virtual_memory_delta_mb: bytes_to_mb(
                end_memory
                    .virtual_bytes
                    .saturating_sub(self.start_memory.virtual_bytes),
            ),
            process_virtual_memory_current_mb: bytes_to_mb(end_memory.virtual_bytes),
            memory_source: "sysinfo::Process::{memory,virtual_memory} (bytes)".to_string(),
            memory_confidence: "process_only".to_string(),
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

        let end_memory = sample_process_memory(&mut self.sys, self.pid);

        let metric = TestMetric {
            schema_version: 1,
            block_name: format!("{}: {}", self.test_name, block_name),
            duration_secs: duration.as_secs_f64(),
            process_memory_delta_mb: bytes_to_mb(
                end_memory
                    .used_bytes
                    .saturating_sub(self.start_memory.used_bytes),
            ),
            process_memory_current_mb: bytes_to_mb(end_memory.used_bytes),
            process_virtual_memory_delta_mb: bytes_to_mb(
                end_memory
                    .virtual_bytes
                    .saturating_sub(self.start_memory.virtual_bytes),
            ),
            process_virtual_memory_current_mb: bytes_to_mb(end_memory.virtual_bytes),
            memory_source: "sysinfo::Process::{memory,virtual_memory} (bytes)".to_string(),
            memory_confidence: "process_only".to_string(),
            timestamp: chrono::Local::now().to_rfc3339(),
        };

        self.log_metric(metric);
        result
    }

    fn log_metric(&self, metric: TestMetric) {
        let report_file =
            std::env::var(Self::REPORT_FILE_ENV).unwrap_or_else(|_| Self::REPORT_FILE.to_string());

        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&report_file)
        {
            Ok(f) => f,
            Err(_) => return,
        };

        if file.lock_exclusive().is_err() {
            return;
        }

        let mut content = String::new();
        let read_success = file.read_to_string(&mut content).is_ok();

        let reports = if read_success && !content.trim().is_empty() {
            match serde_json::from_str::<Vec<TestRunReport>>(&content) {
                Ok(mut parsed_reports) => {
                    let should_create_new = match parsed_reports.last() {
                        None => true,
                        Some(last_report) => {
                            let last_time_str = last_report
                                .metrics
                                .values()
                                .map(|m| &m.timestamp)
                                .max()
                                .unwrap_or(&last_report.timestamp);

                            match (
                                chrono::DateTime::parse_from_rfc3339(last_time_str),
                                chrono::DateTime::parse_from_rfc3339(&metric.timestamp),
                            ) {
                                (Ok(t1), Ok(t2)) => {
                                    let diff = t2.signed_duration_since(t1);
                                    diff.num_seconds().abs() >= 300
                                }
                                _ => true,
                            }
                        }
                    };

                    if should_create_new {
                        let mut map = std::collections::HashMap::new();
                        map.insert(metric.block_name.clone(), metric.clone());
                        parsed_reports.push(TestRunReport {
                            timestamp: metric.timestamp.clone(),
                            metrics: map,
                        });
                    } else {
                        parsed_reports
                            .last_mut()
                            .unwrap()
                            .metrics
                            .insert(metric.block_name.clone(), metric);
                    }
                    parsed_reports
                }
                Err(_) => {
                    let mut all_metrics = parse_all_metrics_resilient(&content);
                    all_metrics.push(metric);
                    group_metrics_into_reports(all_metrics)
                }
            }
        } else {
            let mut map = std::collections::HashMap::new();
            map.insert(metric.block_name.clone(), metric.clone());
            vec![TestRunReport {
                timestamp: metric.timestamp.clone(),
                metrics: map,
            }]
        };

        if let Ok(serialized) = serde_json::to_string_pretty(&reports) {
            let _ = file.set_len(0);
            let _ = file.seek(std::io::SeekFrom::Start(0));
            let _ = file.write_all(serialized.as_bytes());
            let _ = file.sync_all();
        }

        let _ = file.unlock();
    }
}

fn parse_all_metrics_resilient(content: &str) -> Vec<TestMetric> {
    let mut metrics = Vec::new();

    if let Ok(reports) = serde_json::from_str::<Vec<TestRunReport>>(content) {
        for report in reports {
            metrics.extend(report.metrics.into_values());
        }
        return metrics;
    }

    if let Ok(old_metrics) = serde_json::from_str::<Vec<TestMetric>>(content) {
        return old_metrics;
    }

    #[derive(Deserialize)]
    struct TestRunReportLegacy {
        metrics: Vec<TestMetric>,
    }

    if let Ok(reports) = serde_json::from_str::<Vec<TestRunReportLegacy>>(content) {
        for report in reports {
            metrics.extend(report.metrics);
        }
        return metrics;
    }

    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let start = i;
            let mut depth = 1;
            let mut in_string = false;
            let mut escape = false;
            i += 1;
            while i < bytes.len() && depth > 0 {
                let c = bytes[i];
                if escape {
                    escape = false;
                } else if c == b'\\' {
                    escape = true;
                } else if c == b'"' {
                    in_string = !in_string;
                } else if !in_string {
                    if c == b'{' {
                        depth += 1;
                    } else if c == b'}' {
                        depth -= 1;
                    }
                }
                i += 1;
            }
            if depth == 0 {
                if let Some(chunk) = content.get(start..i) {
                    if let Ok(metric) = serde_json::from_str::<TestMetric>(chunk) {
                        metrics.push(metric);
                    } else if let Ok(report) = serde_json::from_str::<TestRunReport>(chunk) {
                        metrics.extend(report.metrics.into_values());
                    } else if let Ok(report) = serde_json::from_str::<TestRunReportLegacy>(chunk) {
                        metrics.extend(report.metrics);
                    }
                }
            }
        } else {
            i += 1;
        }
    }

    metrics
}

fn group_metrics_into_reports(mut metrics: Vec<TestMetric>) -> Vec<TestRunReport> {
    if metrics.is_empty() {
        return Vec::new();
    }

    metrics.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let mut reports: Vec<TestRunReport> = Vec::new();

    for metric in metrics {
        let should_create_new = match reports.last() {
            None => true,
            Some(last_report) => {
                let last_time_str = last_report
                    .metrics
                    .values()
                    .map(|m| &m.timestamp)
                    .max()
                    .unwrap_or(&last_report.timestamp);

                match (
                    chrono::DateTime::parse_from_rfc3339(last_time_str),
                    chrono::DateTime::parse_from_rfc3339(&metric.timestamp),
                ) {
                    (Ok(t1), Ok(t2)) => {
                        let diff = t2.signed_duration_since(t1);
                        diff.num_seconds().abs() >= 300
                    }
                    _ => true,
                }
            }
        };

        if should_create_new {
            let mut map = std::collections::HashMap::new();
            map.insert(metric.block_name.clone(), metric.clone());
            reports.push(TestRunReport {
                timestamp: metric.timestamp.clone(),
                metrics: map,
            });
        } else {
            reports
                .last_mut()
                .unwrap()
                .metrics
                .insert(metric.block_name.clone(), metric);
        }
    }

    reports
}

// ─── Atomic VantaSession (To prevent Interleaving) ───────────

pub struct VantaSession {
    name: String,
    steps: Vec<String>,
    start_time: Instant,
    start_memory: ProcessMemorySample,
}

impl VantaSession {
    pub fn begin(name: &str) -> Self {
        #[cfg(feature = "cli")]
        with_global_bar(|bar| {
            bar.set_message(format!("Running: {}", name));
        });

        let mut sys = System::new_all();
        sys.refresh_all();
        let pid = sysinfo::get_current_pid().unwrap();
        let start_memory = sample_process_memory(&mut sys, pid);

        Self {
            name: name.to_string(),
            steps: Vec::new(),
            start_time: Instant::now(),
            start_memory,
        }
    }

    pub fn step(&mut self, msg: &str) {
        #[cfg(feature = "cli")]
        self.steps.push(format!(
            "  {} {}",
            style("➜").cyan().bold(),
            style(msg).dim()
        ));
        #[cfg(not(feature = "cli"))]
        self.steps.push(format!("  ➜ {}", msg));
    }

    pub fn success(&mut self, msg: &str) {
        #[cfg(feature = "cli")]
        self.steps
            .push(format!("  {} {}", Emoji("✅", "OK"), style(msg).green()));
        #[cfg(not(feature = "cli"))]
        self.steps.push(format!("  OK {}", msg));
    }

    pub fn finish(self, success: bool) {
        let duration = self.start_time.elapsed();
        let mut sys = System::new_all();
        let pid = sysinfo::get_current_pid().unwrap();
        let end_memory = sample_process_memory(&mut sys, pid);
        let mem_delta = bytes_to_mb(
            end_memory
                .used_bytes
                .saturating_sub(self.start_memory.used_bytes),
        );

        let mut output = String::new();
        let bar = "━".repeat(50);
        output.push_str(&format!("\n\x1b[1;36m┏{}┓\n", bar));
        output.push_str(&format!("┃ {:^50} ┃\n", self.name));
        output.push_str(&format!("┗{}┛\x1b[0m\n", bar));

        for step in &self.steps {
            output.push_str(step);
            output.push('\n');
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

        #[cfg(feature = "cli")]
        with_global_bar(|bar| {
            bar.println(output);
            bar.inc(1);
        });
        #[cfg(not(feature = "cli"))]
        print!("{}", output);
    }
}
