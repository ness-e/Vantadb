use std::sync::OnceLock;
use sysinfo::System;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
    Enterprise, // Heavy hardware: AVX-512, high RAM
    Performance, // Standard server: AVX2/Neon, standard RAM
    Survival,    // Constrained devices: Low RAM or Scalar Fallback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    pub instructions: InstructionSet,
    pub profile: HardwareProfile,
    pub logical_cores: usize,
    pub total_memory: u64, // Total RAM in bytes
    pub vitality_score: u32,
    pub env_hash: u64, // Hash of the static environment for invalidation
}

impl HardwareCapabilities {
    pub fn global() -> &'static Self {
        CAPS.get_or_init(|| HardwareScout::detect())
    }
}

pub struct HardwareScout;

impl HardwareScout {
    const PROFILE_PATH: &'static str = ".connectome_profile";

    pub fn detect() -> HardwareCapabilities {
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
                    Self::log_chameleon_changement(&cached_caps, true);
                    return cached_caps;
                } else {
                    eprintln!("[HARDWARE] ⚠️ Environment signature changed. Re-benchmarking...");
                }
            }
        }

        let instructions = Self::detect_instructions();
        let profile = Self::determine_profile(total_memory, instructions);
        
        let vitality_score = Self::calculate_vitality(total_memory, logical_cores, instructions);

        let caps = HardwareCapabilities {
            instructions,
            profile,
            logical_cores,
            total_memory,
            vitality_score,
            env_hash,
        };

        Self::log_chameleon_changement(&caps, false);
        
        // Save new profile
        if let Ok(json) = serde_json::to_string_pretty(&caps) {
            let _ = fs::write(Self::PROFILE_PATH, json);
        }
        
        caps
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
        let memory_gb = memory / (1024 * 1024 * 1024);
        
        if memory_gb >= 16 && instructions == InstructionSet::Avx512 {
            HardwareProfile::Enterprise
        } else if memory_gb >= 4 && instructions != InstructionSet::Fallback {
            HardwareProfile::Performance
        } else {
            HardwareProfile::Survival
        }
    }

    fn calculate_vitality(memory: u64, cores: usize, instructions: InstructionSet) -> u32 {
        let mem_score = (memory / (1024 * 1024 * 1024)) as u32;
        let core_score = cores as u32;
        let instr_score = match instructions {
            InstructionSet::Avx512 => 10,
            InstructionSet::Avx2 => 5,
            InstructionSet::Neon => 5,
            InstructionSet::Fallback => 1,
        };
        (mem_score * 2) + core_score + instr_score
    }

    fn log_chameleon_changement(caps: &HardwareCapabilities, cached: bool) {
        let instr_str = match caps.instructions {
            InstructionSet::Avx512 => "AVX-512",
            InstructionSet::Avx2 => "AVX2",
            InstructionSet::Neon => "NEON",
            InstructionSet::Fallback => "SCALAR FALLBACK",
        };

        let profile_str = match caps.profile {
            HardwareProfile::Enterprise => "ENTERPRISE",
            HardwareProfile::Performance => "PERFORMANCE",
            HardwareProfile::Survival => "SURVIVAL",
        };

        let ram_gb = caps.total_memory / (1024 * 1024 * 1024);
        // Cortex RAM cap is 25% of total memory
        let cortex_cap_gb = (caps.total_memory / 4) / (1024 * 1024 * 1024);
        
        let source_str = if cached { "CACHED" } else { "DETECTED" };

        eprintln!(
            "\n[HARDWARE] 🦎 MODO CAMALEÓN: [{}] {} | RAM: {}GB (Cortex Cap: {}GB) | NÚCLEOS: {} | VITALITY: {}",
            instr_str, source_str, ram_gb, cortex_cap_gb, caps.logical_cores, caps.vitality_score
        );
        eprintln!(
            "[HARDWARE] 🛡️ PERFIL ACTIVADO: [{}]",
            profile_str
        );
    }
}
