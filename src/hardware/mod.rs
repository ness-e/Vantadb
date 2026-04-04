use std::sync::OnceLock;
use sysinfo::System;

/// Global Hardware Profile loaded once at startup.
static CAPS: OnceLock<HardwareCapabilities> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionSet {
    Avx512,
    Avx2,
    Neon,
    Fallback, // Explicit scalar loop network of safety
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareProfile {
    Enterprise, // Heavy hardware: AVX-512, high RAM
    Performance, // Standard server: AVX2/Neon, standard RAM
    Survival,    // Constrained devices: Low RAM or Scalar Fallback
}

#[derive(Debug, Clone)]
pub struct HardwareCapabilities {
    pub instructions: InstructionSet,
    pub profile: HardwareProfile,
    pub logical_cores: usize,
    pub total_memory: u64, // Total RAM in bytes
    pub vitality_score: u32,
}

impl HardwareCapabilities {
    pub fn global() -> &'static Self {
        CAPS.get_or_init(|| HardwareScout::detect())
    }
}

pub struct HardwareScout;

impl HardwareScout {
    pub fn detect() -> HardwareCapabilities {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let total_memory = sys.total_memory();
        let logical_cores = sys.cpus().len();

        let instructions = Self::detect_instructions();
        let profile = Self::determine_profile(total_memory, instructions);
        
        let vitality_score = Self::calculate_vitality(total_memory, logical_cores, instructions);

        let caps = HardwareCapabilities {
            instructions,
            profile,
            logical_cores,
            total_memory,
            vitality_score,
        };

        Self::log_chameleon_changement(&caps);
        
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

    fn log_chameleon_changement(caps: &HardwareCapabilities) {
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

        println!(
            "\n[HARDWARE] 🦎 MODO CAMALEÓN: [{}] DETECTADO | RAM: {}GB (Cortex Cap: {}GB) | NÚCLEOS: {} | VITALITY: {}",
            instr_str, ram_gb, cortex_cap_gb, caps.logical_cores, caps.vitality_score
        );
        println!(
            "[HARDWARE] 🛡️ PERFIL ACTIVADO: [{}]",
            profile_str
        );
    }
}
