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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstructionSet {
    Avx512,
    Avx2,
    Neon,
    Fallback, // Explicit scalar loop network of safety
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareProfile {
    Enterprise,  // Heavy hardware: AVX-512, high RAM
    Performance, // Standard server: AVX2/Neon, standard RAM
    LowResource, // Constrained devices: Low RAM or Scalar Fallback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    pub instructions: InstructionSet,
    pub profile: HardwareProfile,
    pub logical_cores: usize,
    /// Total physical RAM of the host machine in bytes (via `sysinfo::System::total_memory()`).
    ///
    /// **This is NOT process-scoped**: it reports what the OS sees, not what VantaDB
    /// has allocated. For per-process metrics, use `metrics::memory_breakdown_snapshot()`
    /// which reports RSS and virtual memory for the current process.
    pub total_memory: u64,
    pub resource_score: u32,
    pub env_hash: u64, // Hash of the static environment for invalidation
}

impl HardwareCapabilities {
    pub fn global() -> &'static Self {
        CAPS.get_or_init(HardwareScout::detect)
    }
}

pub struct HardwareScout;

impl HardwareScout {
    #[cfg(feature = "sysinfo")]
    const PROFILE_PATH: &'static str = ".vanta_profile";

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
                if let Ok(cached_caps) = serde_json::from_str::<HardwareCapabilities>(&data) {
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
            let env_hash = 0;
            let instructions = Self::detect_instructions();
            let logical_cores = 1;
            let total_memory = GIB; // Conservative 1GB default
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
            caps
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
