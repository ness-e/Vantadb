//! Hardware capability detection and CPU feature probing.
//!
//! Detects available SIMD instruction sets (AVX2, AVX-512, NEON, SVE)
//! and system topology, enabling runtime dispatch in vector transforms.

use serde::{Deserialize, Serialize};
#[cfg(feature = "sysinfo")]
use std::collections::hash_map::DefaultHasher;
#[cfg(feature = "sysinfo")]
use std::fs;
#[cfg(feature = "sysinfo")]
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
#[cfg(feature = "sysinfo")]
use sysinfo::System;

const GIB: u64 = 1024 * 1024 * 1024;

/// Global Hardware Profile loaded once at startup.
static CAPS: OnceLock<HardwareCapabilities> = OnceLock::new();

/// CPU instruction set extensions detected at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstructionSet {
    /// AVX-512 (x86_64).
    Avx512,
    /// AVX2 (x86_64).
    Avx2,
    /// NEON (AArch64).
    Neon,
    /// Scalar fallback when no SIMD is available.
    Fallback,
}

/// System-level hardware performance tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareProfile {
    /// Heavy hardware: AVX-512, 16+ GB RAM.
    Enterprise,
    /// Standard server: AVX2/NEON, 4+ GB RAM.
    Performance,
    /// Constrained devices: low RAM or scalar fallback.
    LowResource,
}

/// Detected host hardware capabilities, cached at startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    /// Detected CPU instruction set.
    pub instructions: InstructionSet,
    /// Derived hardware performance profile.
    pub profile: HardwareProfile,
    /// Number of logical CPU cores.
    pub logical_cores: usize,
    /// Total physical RAM of the host machine in bytes (via `sysinfo::System::total_memory()`).
    ///
    /// **This is NOT process-scoped**: it reports what the OS sees, not what VantaDB
    /// has allocated. For per-process metrics, use `metrics::memory_breakdown_snapshot()`
    /// which reports RSS and virtual memory for the current process.
    pub total_memory: u64,
    /// Composite resource score based on RAM, cores, and instructions.
    pub resource_score: u32,
    /// Hash of the static environment for cache-invalidation on hardware change.
    pub env_hash: u64,
}

impl HardwareCapabilities {
    /// Return the global cached hardware capabilities, detecting them on first call.
    pub fn global() -> &'static Self {
        CAPS.get_or_init(HardwareScout::detect)
    }
}

/// Hardware detection utility that probes CPU features and system memory at startup.
pub struct HardwareScout;

impl HardwareScout {
    #[cfg(feature = "sysinfo")]
    const PROFILE_PATH: &'static str = ".vanta_profile";

    /// Probe the current host and return a fully populated `HardwareCapabilities`.
    pub fn detect() -> HardwareCapabilities {
        #[cfg(feature = "sysinfo")]
        {
            let mut sys = System::new_all();
            sys.refresh_all();

            let total_memory = sys.total_memory();
            let logical_cores = sys.cpus().len();

            // Calculate stable environment hash
            let mut hasher = DefaultHasher::new();
            total_memory.hash(&mut hasher);
            logical_cores.hash(&mut hasher);
            if let Some(cpu) = sys.cpus().first() {
                cpu.brand().hash(&mut hasher);
            }
            let env_hash = hasher.finish();

            // Check if we have a valid cached profile
            if let Ok(data) = fs::read_to_string(Self::PROFILE_PATH) {
                if data.len() > 1024 * 1024 {
                    tracing::warn!("Hardware cache file exceeds 1MB, ignoring");
                } else if let Ok(cached_caps) = serde_json::from_str::<HardwareCapabilities>(&data)
                {
                    if cached_caps.env_hash == env_hash {
                        // Cache Hit: Environment unchanged! Perfect cold-start speedup.
                        Self::log_adaptive_status(&cached_caps, true);
                        return cached_caps;
                    } else {
                        tracing::info!("Environment signature changed. Re-benchmarking...");
                    }
                }
            }

            let instructions = Self::detect_instructions();
            let profile = Self::determine_profile(total_memory, instructions);

            let resource_score =
                Self::calculate_resource_score(total_memory, logical_cores, instructions);

            let caps = HardwareCapabilities {
                instructions,
                profile,
                logical_cores,
                total_memory,
                resource_score,
                env_hash,
            };

            Self::log_adaptive_status(&caps, false);

            // Save new profile
            if let Ok(json) = serde_json::to_string_pretty(&caps) {
                let _ = fs::write(Self::PROFILE_PATH, json);
            }

            caps
        }

        #[cfg(not(feature = "sysinfo"))]
        {
            let instructions = Self::detect_instructions();
            let logical_cores = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);
            let total_memory = Self::detect_total_memory_native().unwrap_or(GIB);
            let profile = Self::determine_profile(total_memory, instructions);
            let resource_score =
                Self::calculate_resource_score(total_memory, logical_cores, instructions);

            let caps = HardwareCapabilities {
                instructions,
                profile,
                logical_cores,
                total_memory,
                resource_score,
                env_hash: 0,
            };

            Self::log_adaptive_status(&caps, false);
            caps
        }
    }

    /// Detect total physical RAM via native OS APIs, with no `sysinfo` dependency.
    /// Returns `None` when the platform provides no supported query.
    #[cfg(not(feature = "sysinfo"))]
    fn detect_total_memory_native() -> Option<u64> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let content = fs::read_to_string("/proc/meminfo").ok()?;
            let line = content.lines().find(|l| l.starts_with("MemTotal:"))?;
            let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
            return Some(kb * 1024);
        }

        #[cfg(target_os = "macos")]
        {
            use libc::sysctlbyname;
            let mut mem: u64 = 0;
            let mut len = std::mem::size_of::<u64>();
            // SAFETY: `hw.memsize` is a stable macOS sysctl; writes into a POD u64 buffer.
            let rc = unsafe {
                sysctlbyname(
                    b"hw.memsize\0".as_ptr().cast(),
                    &mut mem as *mut u64 as *mut _,
                    &mut len,
                    std::ptr::null_mut(),
                    0,
                )
            };
            (rc == 0).then_some(mem)
        }

        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::System::SystemInformation::{
                GlobalMemoryStatusEx, MEMORYSTATUSEX,
            };
            let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
            status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            // SAFETY: GlobalMemoryStatusEx fills the zeroed POD struct; dwLength is set first.
            let rc = unsafe { GlobalMemoryStatusEx(&mut status) };
            (rc != 0).then_some(status.ullTotalPhys)
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            None
        }
    }

    fn detect_instructions() -> InstructionSet {
        // Detect x86_64 AVX-512 / AVX2
        #[cfg(target_arch = "x86_64")]
        {
            if std::is_x86_feature_detected!("avx512f") {
                return InstructionSet::Avx512;
            } else if std::is_x86_feature_detected!("avx2") {
                return InstructionSet::Avx2;
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                return InstructionSet::Neon;
            }
        }

        InstructionSet::Fallback
    }

    fn determine_profile(memory: u64, instructions: InstructionSet) -> HardwareProfile {
        let memory_gb = memory / GIB;

        if memory_gb >= 16 && instructions == InstructionSet::Avx512 {
            HardwareProfile::Enterprise
        } else if memory_gb >= 4 && instructions != InstructionSet::Fallback {
            HardwareProfile::Performance
        } else {
            HardwareProfile::LowResource
        }
    }

    fn calculate_resource_score(memory: u64, cores: usize, instructions: InstructionSet) -> u32 {
        let mem_score = (memory / GIB) as u32;
        let core_score = cores as u32;
        let instr_score = match instructions {
            InstructionSet::Avx512 => 10,
            InstructionSet::Avx2 => 5,
            InstructionSet::Neon => 5,
            InstructionSet::Fallback => 1,
        };
        (mem_score * 2) + core_score + instr_score
    }

    #[cfg(feature = "cli")]
    fn log_adaptive_status(caps: &HardwareCapabilities, cached: bool) {
        use console::style;

        // Fixed inner width = 76 chars (between the │ borders)
        const W: usize = 76;

        let instr_label = match caps.instructions {
            InstructionSet::Avx512 => "AVX-512",
            InstructionSet::Avx2 => "AVX2",
            InstructionSet::Neon => "NEON",
            InstructionSet::Fallback => "SCALAR",
        };
        let instr_styled = match caps.instructions {
            InstructionSet::Avx512 => style(instr_label).cyan().bold(),
            InstructionSet::Avx2 => style(instr_label).cyan().bold(),
            InstructionSet::Neon => style(instr_label).cyan().bold(),
            InstructionSet::Fallback => style(instr_label).red().dim(),
        };
        let profile_label = match caps.profile {
            HardwareProfile::Enterprise => "ENTERPRISE",
            HardwareProfile::Performance => "PERFORMANCE",
            HardwareProfile::LowResource => "LOW-RESOURCE",
        };
        let profile_styled = match caps.profile {
            HardwareProfile::Enterprise => style(profile_label).green().bold(),
            HardwareProfile::Performance => style(profile_label).yellow().bold(),
            HardwareProfile::LowResource => style(profile_label).red().bold(),
        };

        let ram_gb = caps.total_memory / GIB;
        let cache_gb = (caps.total_memory / 4) / GIB;
        let source = if cached { "CACHED" } else { "DETECTED" };

        // Helper: render a plain-text version to measure, then build styled line
        let hw_row_plain = format!(
            " ⚡  CPU  {}   RAM {}GB (cache {}GB)  │  {} cores  │  score {}",
            instr_label, ram_gb, cache_gb, caps.logical_cores, caps.resource_score
        );
        let prof_row_plain = format!(" ★  Profile: {}   Source: {}", profile_label, source);

        fn pad_to(plain: &str, width: usize) -> usize {
            width.saturating_sub(plain.chars().count())
        }

        let hw_pad = pad_to(&hw_row_plain, W);
        let prof_pad = pad_to(&prof_row_plain, W);

        let top = format!("  ╭{}╮", "─".repeat(W));
        let bottom = format!("  ╰{}╯", "─".repeat(W));
        let mid = format!("  ├{}┤", "─".repeat(W));
        let blank = format!("  │{}│", " ".repeat(W));

        eprintln!();
        eprintln!("{}", style(&top).color256(240).dim());
        eprintln!("{blank}");
        eprintln!(
            "  │ ⚡  {} [ {} ]   RAM {}GB (cache {}GB)  │  {} cores  │  score {}{}│",
            style("CPU").bold().white(),
            instr_styled,
            style(format!("{}GB", ram_gb)).white(),
            style(format!("{}GB", cache_gb)).white().dim(),
            style(caps.logical_cores).white(),
            style(caps.resource_score).magenta().bold(),
            " ".repeat(hw_pad),
        );
        eprintln!("{mid}");
        eprintln!(
            "  │ ★  Profile: {}   Source: {}{}│",
            profile_styled,
            style(source).white().dim(),
            " ".repeat(prof_pad),
        );
        eprintln!("{blank}");
        eprintln!("{}", style(&bottom).color256(240).dim());
        eprintln!();
    }

    #[cfg(not(feature = "cli"))]
    fn log_adaptive_status(caps: &HardwareCapabilities, _cached: bool) {
        tracing::info!(
            "Hardware Profile: {:?} | Cores: {} | RAM: {}GB | Score: {}",
            caps.profile,
            caps.logical_cores,
            caps.total_memory / GIB,
            caps.resource_score
        );
    }
}

#[cfg(test)]
#[allow(missing_docs, clippy::module_inception)]
mod tests {
    use super::*;

    // ─── InstructionSet ───────────────────────────────────────

    #[test]
    fn test_instruction_set_equality() {
        assert_eq!(InstructionSet::Avx512, InstructionSet::Avx512);
        assert_eq!(InstructionSet::Avx2, InstructionSet::Avx2);
        assert_eq!(InstructionSet::Neon, InstructionSet::Neon);
        assert_eq!(InstructionSet::Fallback, InstructionSet::Fallback);
        assert_ne!(InstructionSet::Avx512, InstructionSet::Avx2);
        assert_ne!(InstructionSet::Avx512, InstructionSet::Fallback);
        assert_ne!(InstructionSet::Neon, InstructionSet::Avx2);
    }

    #[test]
    fn test_instruction_set_debug() {
        let variants = [
            InstructionSet::Avx512,
            InstructionSet::Avx2,
            InstructionSet::Neon,
            InstructionSet::Fallback,
        ];
        for variant in &variants {
            let s = format!("{variant:?}");
            assert!(!s.is_empty());
        }
        assert_eq!(format!("{:?}", InstructionSet::Avx512), "Avx512");
    }

    // ─── HardwareProfile ──────────────────────────────────────

    #[test]
    fn test_hardware_profile_equality() {
        assert_eq!(HardwareProfile::Enterprise, HardwareProfile::Enterprise);
        assert_eq!(HardwareProfile::Performance, HardwareProfile::Performance);
        assert_eq!(HardwareProfile::LowResource, HardwareProfile::LowResource);
        assert_ne!(HardwareProfile::Enterprise, HardwareProfile::Performance);
        assert_ne!(HardwareProfile::LowResource, HardwareProfile::Enterprise);
    }

    #[test]
    fn test_hardware_profile_debug() {
        assert_eq!(format!("{:?}", HardwareProfile::Enterprise), "Enterprise");
        assert_eq!(format!("{:?}", HardwareProfile::Performance), "Performance");
        assert_eq!(format!("{:?}", HardwareProfile::LowResource), "LowResource");
    }

    // ─── HardwareCapabilities ─────────────────────────────────

    #[test]
    fn test_hardware_capabilities_creation_enterprise() {
        let caps = HardwareCapabilities {
            instructions: InstructionSet::Avx512,
            profile: HardwareProfile::Enterprise,
            logical_cores: 32,
            total_memory: 64 * GIB,
            resource_score: 170,
            env_hash: 0xDEAD,
        };
        assert_eq!(caps.instructions, InstructionSet::Avx512);
        assert_eq!(caps.profile, HardwareProfile::Enterprise);
        assert_eq!(caps.logical_cores, 32);
        assert_eq!(caps.total_memory, 64 * GIB);
        assert_eq!(caps.resource_score, 170);
        assert_eq!(caps.env_hash, 0xDEAD);
    }

    #[test]
    fn test_hardware_capabilities_creation_performance() {
        let caps = HardwareCapabilities {
            instructions: InstructionSet::Neon,
            profile: HardwareProfile::Performance,
            logical_cores: 8,
            total_memory: 8 * GIB,
            resource_score: 29,
            env_hash: 0,
        };
        assert_eq!(caps.instructions, InstructionSet::Neon);
        assert_eq!(caps.profile, HardwareProfile::Performance);
        assert_eq!(caps.logical_cores, 8);
        assert_eq!(caps.total_memory, 8 * GIB);
        assert_eq!(caps.resource_score, 29);
    }

    #[test]
    fn test_hardware_capabilities_creation_low_resource() {
        let caps = HardwareCapabilities {
            instructions: InstructionSet::Fallback,
            profile: HardwareProfile::LowResource,
            logical_cores: 1,
            total_memory: GIB,
            resource_score: 4,
            env_hash: 42,
        };
        assert_eq!(caps.instructions, InstructionSet::Fallback);
        assert_eq!(caps.profile, HardwareProfile::LowResource);
        assert_eq!(caps.logical_cores, 1);
        assert_eq!(caps.total_memory, GIB);
        assert_eq!(caps.resource_score, 4);
    }

    // ─── HardwareScout::determine_profile ─────────────────────

    #[test]
    fn test_determine_profile_enterprise() {
        assert_eq!(
            HardwareScout::determine_profile(16 * GIB, InstructionSet::Avx512),
            HardwareProfile::Enterprise,
            "16GB + Avx512 → Enterprise"
        );
        assert_eq!(
            HardwareScout::determine_profile(32 * GIB, InstructionSet::Avx512),
            HardwareProfile::Enterprise,
            "32GB + Avx512 → Enterprise"
        );
    }

    #[test]
    fn test_determine_profile_performance() {
        assert_eq!(
            HardwareScout::determine_profile(4 * GIB, InstructionSet::Avx2),
            HardwareProfile::Performance,
            "4GB + AVX2 → Performance"
        );
        assert_eq!(
            HardwareScout::determine_profile(8 * GIB, InstructionSet::Neon),
            HardwareProfile::Performance,
            "8GB + NEON → Performance"
        );
        assert_eq!(
            HardwareScout::determine_profile(16 * GIB, InstructionSet::Avx2),
            HardwareProfile::Performance,
            "16GB + AVX2 → Performance (no Avx512)"
        );
    }

    #[test]
    fn test_determine_profile_low_resource_due_to_ram() {
        assert_eq!(
            HardwareScout::determine_profile(2 * GIB, InstructionSet::Avx2),
            HardwareProfile::LowResource,
            "2GB + AVX2 → LowResource"
        );
        assert_eq!(
            HardwareScout::determine_profile(GIB, InstructionSet::Neon),
            HardwareProfile::LowResource,
            "1GB + NEON → LowResource"
        );
        assert_eq!(
            HardwareScout::determine_profile(0, InstructionSet::Avx512),
            HardwareProfile::LowResource,
            "0GB + AVX512 → LowResource"
        );
    }

    #[test]
    fn test_determine_profile_low_resource_due_to_fallback() {
        assert_eq!(
            HardwareScout::determine_profile(8 * GIB, InstructionSet::Fallback),
            HardwareProfile::LowResource,
            "8GB + Fallback → LowResource"
        );
        assert_eq!(
            HardwareScout::determine_profile(64 * GIB, InstructionSet::Fallback),
            HardwareProfile::LowResource,
            "64GB + Fallback → LowResource"
        );
    }

    // ─── HardwareScout::calculate_resource_score ──────────────

    #[test]
    fn test_resource_score_avx512() {
        // mem_score = 64 / 1 = 64, core_score = 32, instr_score = 10
        // total = 64*2 + 32 + 10 = 170
        assert_eq!(
            HardwareScout::calculate_resource_score(64 * GIB, 32, InstructionSet::Avx512),
            170
        );
    }

    #[test]
    fn test_resource_score_avx2() {
        // mem_score = 8, core_score = 8, instr_score = 5
        // total = 8*2 + 8 + 5 = 29
        assert_eq!(
            HardwareScout::calculate_resource_score(8 * GIB, 8, InstructionSet::Avx2),
            29
        );
    }

    #[test]
    fn test_resource_score_neon() {
        // mem_score = 4, core_score = 4, instr_score = 5
        // total = 4*2 + 4 + 5 = 17
        assert_eq!(
            HardwareScout::calculate_resource_score(4 * GIB, 4, InstructionSet::Neon),
            17
        );
    }

    #[test]
    fn test_resource_score_fallback() {
        // mem_score = 1, core_score = 1, instr_score = 1
        // total = 1*2 + 1 + 1 = 4
        assert_eq!(
            HardwareScout::calculate_resource_score(GIB, 1, InstructionSet::Fallback),
            4
        );
    }

    #[test]
    fn test_resource_score_zero_memory() {
        // mem_score = 0, core_score = 1, instr_score = 1
        // total = 0*2 + 1 + 1 = 2
        assert_eq!(
            HardwareScout::calculate_resource_score(0, 1, InstructionSet::Fallback),
            2
        );
    }

    // ─── HardwareScout::detect_instructions ───────────────────

    #[test]
    fn test_detect_instructions_returns_valid_variant() {
        let instr = HardwareScout::detect_instructions();
        match instr {
            InstructionSet::Avx512
            | InstructionSet::Avx2
            | InstructionSet::Neon
            | InstructionSet::Fallback => {}
        }
    }

    // ─── HardwareScout::detect (fallback path) ────────────────

    #[test]
    fn test_detect_returns_valid_caps() {
        let caps = HardwareScout::detect();
        // Without sysinfo: real values; with sysinfo: real values
        assert!(caps.logical_cores >= 1, "at least 1 core");
        assert!(caps.total_memory >= GIB, "at least 1GB RAM");
        match caps.instructions {
            InstructionSet::Avx512
            | InstructionSet::Avx2
            | InstructionSet::Neon
            | InstructionSet::Fallback => {}
        }
        match caps.profile {
            HardwareProfile::Enterprise
            | HardwareProfile::Performance
            | HardwareProfile::LowResource => {}
        }
    }

    #[cfg(not(feature = "sysinfo"))]
    #[test]
    fn test_native_memory_detection_reports_real_ram() {
        // Native detection must never return the hardcoded 1GB default on a
        // supported desktop OS — it should report actual physical RAM.
        if let Some(mem) = HardwareScout::detect_total_memory_native() {
            assert!(mem > GIB, "native RAM detection reported {mem} bytes");
        }
    }

    // ─── HardwareCapabilities::global ─────────────────────────

    #[test]
    fn test_global_caps_is_singleton() {
        let a = HardwareCapabilities::global() as *const _ as usize;
        let b = HardwareCapabilities::global() as *const _ as usize;
        assert_eq!(a, b, "global() should return the same static reference");
    }

    #[test]
    fn test_global_caps_has_reasonable_values() {
        let caps = HardwareCapabilities::global();
        assert!(caps.logical_cores >= 1);
        assert!(caps.total_memory >= GIB);
        assert!(caps.resource_score >= 1);
    }

    // ─── HardwareScout::log_adaptive_status (non-cli) ─────────

    #[test]
    fn test_log_adaptive_status_does_not_panic() {
        let caps = HardwareCapabilities {
            instructions: InstructionSet::Avx2,
            profile: HardwareProfile::Performance,
            logical_cores: 8,
            total_memory: 16 * GIB,
            resource_score: 50,
            env_hash: 0,
        };
        // Should not panic, just log via tracing
        HardwareScout::log_adaptive_status(&caps, false);
        HardwareScout::log_adaptive_status(&caps, true);
    }

    // ─── Serialization round-trip ─────────────────────────────

    #[test]
    fn test_hardware_capabilities_serialization() {
        let caps = HardwareCapabilities {
            instructions: InstructionSet::Avx2,
            profile: HardwareProfile::Performance,
            logical_cores: 8,
            total_memory: 16 * GIB,
            resource_score: 50,
            env_hash: 42,
        };
        let json = serde_json::to_string(&caps).expect("serialize");
        let deserialized: HardwareCapabilities = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.instructions, InstructionSet::Avx2);
        assert_eq!(deserialized.profile, HardwareProfile::Performance);
        assert_eq!(deserialized.logical_cores, 8);
        assert_eq!(deserialized.total_memory, 16 * GIB);
        assert_eq!(deserialized.resource_score, 50);
        assert_eq!(deserialized.env_hash, 42);
    }
}
