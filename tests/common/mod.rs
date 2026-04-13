#![allow(dead_code)]

use std::time::Instant;
use sysinfo::System;
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Serialize, Deserialize};
use std::fs::{OpenOptions};
use std::io::Write;

pub mod sift_loader;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestMetric {
    pub block_name: String,
    pub duration_secs: f64,
    pub ram_usage_mb: f64,
    pub current_ram_mb: f64,
    pub timestamp: String,
}

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
        
        // Initial snapshot
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
    where F: FnOnce() -> R 
    {
        TerminalReporter::block_header(block_name);
        
        let t0 = Instant::now();
        let result = f();
        let duration = t0.elapsed();
        
        // Measurements
        self.sys.refresh_process(self.pid);
        let end_mem = self.sys.process(self.pid).map(|p| p.memory()).unwrap_or(0);
        let mem_usage_kb = if end_mem > self.start_mem { end_mem - self.start_mem } else { 0 };
        
        let metric = TestMetric {
            block_name: format!("{}: {}", self.test_name, block_name),
            duration_secs: duration.as_secs_f64(),
            ram_usage_mb: mem_usage_kb as f64 / 1024.0,
            current_ram_mb: end_mem as f64 / 1024.0,
            timestamp: chrono::Local::now().to_rfc3339(),
        };

        // Standard Report
        println!("\n  {}", style("CERTIFICATION METRICS").bold().underlined());
        println!("  {} Time:      {:.2}s", style("⏱️").cyan(), metric.duration_secs);
        println!("  {} RAM Usage: {:.2} MB (Current: {:.2} MB)", 
            style("🧠").magenta(), 
            metric.ram_usage_mb,
            metric.current_ram_mb
        );
        
        self.log_metric(metric);
        
        result
    }

    fn log_metric(&self, metric: TestMetric) {
        // Append to JSON list (simplified for now as a line-based JSON for easier concurrent appends)
        if let Ok(json) = serde_json::to_string(&metric) {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(Self::REPORT_FILE) {
                let _ = writeln!(file, "{}", json);
            }
        }
    }
}

pub struct TerminalReporter;

impl TerminalReporter {
    pub fn block_header(title: &str) {
        let bar = "━".repeat(title.len() + 4);
        println!("\n{}", style(format!("┏{}┓", bar)).cyan().dim());
        println!(
            "{}  {}  {}",
            style("┃").cyan().dim(),
            style(title).bold().white(),
            style("┃").cyan().dim()
        );
        println!("{}\n", style(format!("┗{}┛", bar)).cyan().dim());
    }

    #[allow(dead_code)]
    pub fn sub_step(msg: &str) {
        println!("  {} {}", style("➜").cyan().bold(), style(msg).dim());
    }

    pub fn success(msg: &str) {
        println!("  {} {}", Emoji("✅", "OK"), style(msg).green());
    }

    #[allow(dead_code)]
    pub fn info(msg: &str) {
        println!("  {} {}", Emoji("ℹ️ ", "i"), style(msg).blue());
    }

    #[allow(dead_code)]
    pub fn warning(msg: &str) {
        println!("  {} {}", Emoji("⚠️ ", "W"), style(msg).yellow());
    }

    #[allow(dead_code)]
    pub fn create_progress(len: u64, msg: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .expect("Invalid progress template")
            .progress_chars("█▉▊▋▌▍▎▏  "));
        pb.set_message(msg.to_string());
        pb
    }
}
