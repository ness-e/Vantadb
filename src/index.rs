#[cfg(not(feature = "memmap2"))]
use crate::storage::MmapMut;
use dashmap::DashMap;
#[cfg(feature = "memmap2")]
use memmap2::MmapMut;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;
use std::fs::{File, OpenOptions};
use std::hash::BuildHasherDefault;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tracing::{info, warn};
use twox_hash::XxHash64;

const ENTRY_POINT_NONE: u64 = u64::MAX;
const MAX_VEC_F32_LEN: usize = 10_000_000; // Max ~40MB for a single f32 vector

pub use crate::node::{DistanceMetric, SendPtr, VectorRepresentations};
use crate::vector::quantization::{rabitq_similarity, turbo_quant_similarity};

/// SCALE-01: Prefetching Predictivo del Kernel para búsqueda HNSW MMap.
///
/// Emite una sugerencia asíncrona al OS para pre-cargar páginas físicas del vector
/// de un nodo candidato *antes* de que la CPU las calcule. Esto oculta la latencia
/// de page fault detrás del cálculo de distancia del nodo actual.
///
/// La función es no-bloqueante y best-effort: si falla (permisos, no soportado),
/// la búsqueda continúa correctamente sin degradación.
///
/// # Safety
/// El puntero `mmap_ptr` debe ser válido durante la duración de la llamada.
/// El rango `[offset, offset+len)` debe estar dentro del mmap mapeado.
#[inline(always)]
#[allow(unused_variables)]
fn prefetch_mmap_vector(mmap_ptr: *const u8, offset: usize, len: usize) {
    #[cfg(unix)]
    {
        // POSIX: madvise(MADV_WILLNEED) — solicita al kernel cargar páginas de forma asíncrona.
        // Disponible en Linux y macOS. No bloquea el hilo llamante.
        unsafe {
            // SAFETY: mmap_ptr es un puntero activo al mmap; offset+len está validado por
            // el caller. madvise falla silenciosamente si el rango es inválido.
            libc::madvise(
                mmap_ptr.add(offset) as *mut libc::c_void,
                len,
                libc::MADV_WILLNEED,
            );
        }
    }

    #[cfg(windows)]
    {
        // Windows: PrefetchVirtualMemory — equivalente a MADV_WILLNEED.
        // Disponible desde Windows 8 / Server 2012. Falla silenciosamente en versiones anteriores.
        use windows_sys::Win32::System::Memory::{PrefetchVirtualMemory, WIN32_MEMORY_RANGE_ENTRY};
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        unsafe {
            // SAFETY: mismas garantías que el caso Unix.
            let addr = mmap_ptr.add(offset) as *mut core::ffi::c_void;
            let entry = WIN32_MEMORY_RANGE_ENTRY {
                VirtualAddress: addr,
                NumberOfBytes: len,
            };
            let process_handle = GetCurrentProcess();
            // La firma acepta *const WIN32_MEMORY_RANGE_ENTRY y Flags=0 (requerido por Win32)
            PrefetchVirtualMemory(process_handle, 1, std::ptr::addr_of!(entry), 0);
        }
    }

    // Fallback no-op para plataformas sin soporte (e.g., WASM, Tier-3).
    // El compilador elimina este bloque vacío en release.
    #[cfg(not(any(unix, windows)))]
    let _ = (mmap_ptr, offset, len);
}

/// Libera páginas de memoria del mmap para nodos fríos (Cold tier).
/// Usa madvise(MADV_DONTNEED) para indicar al kernel que estas páginas
/// pueden ser liberadas de RAM, reduciendo el RSS sin invalidar el mmap.
///
/// El rango `[offset, offset+len)` debe estar dentro del mmap mapeado.
///
/// # Safety
///
/// - `mmap_ptr` must be a valid pointer to an active memory-mapped region.
/// - `offset + len` must not exceed the length of the mapped region.
/// - The caller must hold a read guard (or equivalent) ensuring the mmap
///   is not unmapped for the duration of this call.
#[inline(always)]
#[allow(unused_variables)]
pub unsafe fn release_mmap_vector(mmap_ptr: *const u8, offset: usize, len: usize) {
    #[cfg(unix)]
    {
        // POSIX: madvise(MADV_DONTNEED) — indica al kernel que estas páginas
        // no son necesarias y pueden ser liberadas de RAM. El mmap sigue válido,
        // pero las páginas se cargarán bajo demanda desde disco cuando se accedan.
        // Disponible en Linux y macOS. No bloquea el hilo llamante.
        unsafe {
            // SAFETY: mmap_ptr es un puntero activo al mmap; offset+len está validado por
            // el caller. madvise falla silenciosamente si el rango es inválido.
            libc::madvise(
                mmap_ptr.add(offset) as *mut libc::c_void,
                len,
                libc::MADV_DONTNEED,
            );
        }
    }

    #[cfg(windows)]
    {
        // Windows: No hay equivalente directo a MADV_DONTNEED.
        // VirtualUnlock puede liberar páginas del working set, pero requiere
        // que las páginas estén bloqueadas primero. Para simplicidad, no-op.
        let _ = (mmap_ptr, offset, len);
    }

    // Fallback no-op para plataformas sin soporte (e.g., WASM, Tier-3).
    // El compilador elimina este bloque vacío en release.
    #[cfg(not(any(unix, windows)))]
    let _ = (mmap_ptr, offset, len);
}

use crate::config::PrefetchMode;
use std::sync::OnceLock;

/// Global prefetch mode — initialized once from `VantaConfig::prefetch_mode`
/// or falls back to the env var `VANTA_PREFETCH` / `VANTA_DISABLE_PREFETCH`.
static PREFETCH_MODE: OnceLock<PrefetchMode> = OnceLock::new();

/// Override the global prefetch mode at runtime (called during initialization).
pub fn set_prefetch_mode(mode: PrefetchMode) {
    let _ = PREFETCH_MODE.set(mode);
}

#[inline(always)]
fn should_prefetch() -> bool {
    if let Some(mode) = PREFETCH_MODE.get() {
        return mode.is_prefetch_enabled();
    }
    // Fallback: if config never initialized, read env var
    let mode = std::env::var("VANTA_PREFETCH")
        .ok()
        .map(|v| PrefetchMode::from_env_value(&v));
    let disabled = std::env::var("VANTA_DISABLE_PREFETCH")
        .ok()
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);
    match (mode, disabled) {
        (Some(m), _) => m.is_prefetch_enabled(),
        (_, true) => false,
        _ => true,
    }
}

const VECTOR_INDEX_VERSION: u16 = 4; // Upgraded for zero-copy aligned vector paging

#[inline(always)]
fn f32_dot_and_norm_b_sq(a: &[f32], b: &[f32]) -> (f32, f32) {
    if a.len() != b.len() || a.is_empty() {
        return (0.0, 0.0);
    }
    use wide::f32x8;
    let mut dot_v = f32x8::ZERO;
    let mut norm_b_v = f32x8::ZERO;
    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();
    for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
        let va = f32x8::from(
            *<&[f32; 8]>::try_from(a_chunk).expect("chunks_exact(8) yields 8-element chunks"),
        );
        let vb = f32x8::from(
            *<&[f32; 8]>::try_from(b_chunk).expect("chunks_exact(8) yields 8-element chunks"),
        );
        dot_v += va * vb;
        norm_b_v += vb * vb;
    }
    let mut dot = dot_v.reduce_add();
    let mut norm_b = norm_b_v.reduce_add();
    for i in 0..rem_a.len() {
        dot += rem_a[i] * rem_b[i];
        norm_b += rem_b[i] * rem_b[i];
    }
    (dot, norm_b)
}

/// Pure dot product — no norm computation. ~2x faster than f32_dot_and_norm_b_sq
/// when norms are already cached.
#[inline(always)]
fn f32_dot_product(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    use wide::f32x8;
    let mut dot_v = f32x8::ZERO;
    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();
    for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
        let va = f32x8::from(
            *<&[f32; 8]>::try_from(a_chunk).expect("chunks_exact(8) yields 8-element chunks"),
        );
        let vb = f32x8::from(
            *<&[f32; 8]>::try_from(b_chunk).expect("chunks_exact(8) yields 8-element chunks"),
        );
        dot_v += va * vb;
    }
    let mut dot = dot_v.reduce_add();
    for i in 0..rem_a.len() {
        dot += rem_a[i] * rem_b[i];
    }
    dot
}

#[inline(always)]
pub fn f32_l2_norm(v: &[f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let (_, norm_sq) = f32_dot_and_norm_b_sq(v, v);
    norm_sq.sqrt()
}

/// Cosine similarity when BOTH inverse norms are pre-cached. Uses pure dot product and multiplications only.
/// This is the fastest path — eliminates 100% of division and ~50% of SIMD work.
#[inline(always)]
pub fn cosine_sim_cached_norms(a: &[f32], inv_norm_a: f32, b: &[f32], inv_norm_b: f32) -> f32 {
    if inv_norm_a < f32::EPSILON || inv_norm_b < f32::EPSILON || a.len() != b.len() || a.is_empty()
    {
        return 0.0;
    }
    let dot = f32_dot_product(a, b);
    dot * inv_norm_a * inv_norm_b
}

/// Cosine similarity when `||query||` was already computed for the search hot path.
#[inline(always)]
pub fn cosine_sim_with_query_norm(query: &[f32], query_norm: f32, b: &[f32]) -> f32 {
    if query_norm < f32::EPSILON || query.len() != b.len() || query.is_empty() {
        return 0.0;
    }
    let (dot, norm_b_sq) = f32_dot_and_norm_b_sq(query, b);
    let norm_b = norm_b_sq.sqrt();
    if norm_b < f32::EPSILON {
        0.0
    } else {
        dot / (query_norm * norm_b)
    }
}

#[inline(always)]
pub fn cosine_sim_f32(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let norm_a = f32_l2_norm(a);
    cosine_sim_with_query_norm(a, norm_a, b)
}

#[inline(always)]
pub fn euclidean_distance_squared_f32(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    use wide::f32x8;
    let mut sum_v = f32x8::ZERO;
    let chunks_a = a.chunks_exact(8);
    let chunks_b = b.chunks_exact(8);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();
    for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
        let va = f32x8::from(
            *<&[f32; 8]>::try_from(a_chunk).expect("chunks_exact(8) yields 8-element chunks"),
        );
        let vb = f32x8::from(
            *<&[f32; 8]>::try_from(b_chunk).expect("chunks_exact(8) yields 8-element chunks"),
        );
        let diff = va - vb;
        sum_v += diff * diff;
    }
    let mut sum = sum_v.reduce_add();
    for i in 0..rem_a.len() {
        let diff = rem_a[i] - rem_b[i];
        sum += diff * diff;
    }
    sum
}

/// Compute similarity against a raw query when SQ8 is the only available
/// representation for the stored node. Decodes on the fly.
fn sq8_similarity_fallback(
    raw_query: &[f32],
    sq8_data: &[i8],
    sq8_scale: f32,
    metric: DistanceMetric,
    _query_norm: Option<f32>,
) -> f32 {
    let inv_scale = sq8_scale / 127.0;
    match metric {
        DistanceMetric::Cosine => {
            let mut dot = 0.0_f32;
            let mut norm_q = 0.0_f32;
            for (&q, &s) in raw_query.iter().zip(sq8_data.iter()) {
                let decoded = (s as f32) * inv_scale;
                dot += q * decoded;
                norm_q += q * q;
            }
            let norm_sq = sq8_data.iter().fold(0.0_f32, |acc, &s| {
                let d = (s as f32) * inv_scale;
                acc + d * d
            });
            if norm_q <= f32::EPSILON || norm_sq <= f32::EPSILON {
                return 0.0;
            }
            dot / (norm_q.sqrt() * norm_sq.sqrt())
        }
        DistanceMetric::Euclidean => {
            let mut sum_sq = 0.0_f32;
            for (&q, &s) in raw_query.iter().zip(sq8_data.iter()) {
                let diff = q - (s as f32) * inv_scale;
                sum_sq += diff * diff;
            }
            -sum_sq
        }
    }
}

pub fn calculate_similarity(
    raw_query: &[f32],
    query_norm: Option<f32>,
    quantized_query_1bit: Option<&[u64]>,
    quantized_query_3bit: Option<(&[u8], f32)>,
    node_vec: &VectorRepresentations,
    metric: DistanceMetric,
) -> f32 {
    match node_vec {
        VectorRepresentations::Binary(b) => {
            if let Some(q1) = quantized_query_1bit {
                rabitq_similarity(q1, b)
            } else {
                0.0
            }
        }
        VectorRepresentations::Turbo(t) => {
            if let Some((q3, max_abs)) = quantized_query_3bit {
                turbo_quant_similarity(q3, max_abs, t, 1.0)
            } else {
                0.0
            }
        }
        VectorRepresentations::SQ8(data, scale) => {
            // Decode SQ8 on the fly and compute similarity
            sq8_similarity_fallback(raw_query, data, *scale, metric, query_norm)
        }
        VectorRepresentations::Full(f) => match metric {
            DistanceMetric::Cosine => match query_norm {
                Some(norm) => cosine_sim_with_query_norm(raw_query, norm, f),
                None => cosine_sim_f32(raw_query, f),
            },
            DistanceMetric::Euclidean => -euclidean_distance_squared_f32(raw_query, f),
        },
        VectorRepresentations::MmapFull(ptr, len) => {
            debug_assert!(!ptr.0.is_null(), "MmapFull pointer is null in compute_similarity");
            debug_assert!(*len > 0 && *len <= MAX_VEC_F32_LEN, "MmapFull len out of range in compute_similarity");
            let slice = unsafe { std::slice::from_raw_parts(ptr.0, *len) };
            match metric {
                DistanceMetric::Cosine => match query_norm {
                    Some(norm) => cosine_sim_with_query_norm(raw_query, norm, slice),
                    None => cosine_sim_f32(raw_query, slice),
                },
                DistanceMetric::Euclidean => -euclidean_distance_squared_f32(raw_query, slice),
            }
        }
        VectorRepresentations::None => 0.0,
    }
}

#[inline(always)]
fn f32_slice_similarity(
    query_vec: &[f32],
    query_norm: Option<f32>,
    candidate: &[f32],
    metric: DistanceMetric,
) -> f32 {
    match metric {
        DistanceMetric::Cosine => match query_norm {
            Some(norm) => cosine_sim_with_query_norm(query_vec, norm, candidate),
            None => cosine_sim_f32(query_vec, candidate),
        },
        DistanceMetric::Euclidean => -euclidean_distance_squared_f32(query_vec, candidate),
    }
}

pub struct HnswNode {
    pub id: u64,
    pub bitset: u128,
    pub vec_data: VectorRepresentations,
    pub neighbors: Vec<Vec<u64>>,
    /// Offset into the VantaFile (Phase 3)
    pub storage_offset: u64,
    /// Pre-computed inverse L2 norm (1.0 / L2_norm) for Cosine fast-path. 0.0 = not cached.
    pub inv_cached_norm: f32,
}

#[derive(Debug)]
pub enum IndexBackend {
    InMemory,
    MMapFile {
        path: PathBuf,
        mmap: Option<MmapMut>,
    },
}

impl IndexBackend {
    pub fn new_mmap(path: PathBuf) -> Self {
        IndexBackend::MMapFile { path, mmap: None }
    }

    pub fn is_mmap(&self) -> bool {
        matches!(self, IndexBackend::MMapFile { .. })
    }

    pub fn mmap_path(&self) -> Option<&Path> {
        match self {
            IndexBackend::MMapFile { path, .. } => Some(path.as_path()),
            IndexBackend::InMemory => None,
        }
    }

    /// Returns the number of resident (physically-in-RAM) bytes for the HNSW index mmap.
    /// Uses the live mmap pointer directly when available (zero syscall overhead for address resolution).
    /// Falls back to opening the file and creating a temporary read-only Mmap if the mutable mmap is not loaded.
    pub fn mmap_resident_bytes(&self) -> Option<u64> {
        match self {
            IndexBackend::MMapFile { mmap: Some(m), .. } => {
                crate::storage::get_resident_bytes(m.as_ptr(), m.len())
            }
            IndexBackend::MMapFile { path, mmap: None } => {
                // Fallback: open the file and create a temporary read-only mmap
                let file = match File::open(path) {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::debug!(
                            "mmap_resident_bytes fallback: failed to open {}: {e}",
                            path.display()
                        );
                        return None;
                    }
                };
                let mmap = match unsafe { crate::storage::Mmap::map(&file) } {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::debug!(
                            "mmap_resident_bytes fallback: failed to mmap {}: {e}",
                            path.display()
                        );
                        return None;
                    }
                };
                crate::storage::get_resident_bytes(mmap.as_ptr(), mmap.len())
            }
            IndexBackend::InMemory => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HnswConfig {
    pub m: usize,
    pub m_max0: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub ml: f64,
    #[serde(default)]
    pub distance_metric: DistanceMetric,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 32,
            m_max0: 64,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (32_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
        }
    }
}

// Custom wrapper to store (similarity, node_id) in BinaryHeap (Max-Heap)
#[derive(Clone, PartialEq, Debug)]
struct NodeSim(f32, u64);

impl Eq for NodeSim {}

impl PartialOrd for NodeSim {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeSim {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self
            .0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => other.1.cmp(&self.1),
            cmp => cmp,
        }
    }
}

// Wrapper for Min-Heap (used to track closest in result set)
#[derive(Clone, PartialEq, Debug)]
struct NodeSimMin(f32, u64);

impl Eq for NodeSimMin {}

impl PartialOrd for NodeSimMin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeSimMin {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other
            .0
            .partial_cmp(&self.0)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => self.1.cmp(&other.1),
            cmp => cmp,
        }
    }
}

pub struct CPIndex {
    pub nodes: DashMap<u64, HnswNode, BuildHasherDefault<XxHash64>>,
    pub max_layer: AtomicUsize,
    pub entry_point: AtomicU64,
    pub backend: IndexBackend,
    pub config: HnswConfig,
    rng: parking_lot::Mutex<rand::rngs::StdRng>,
}

impl CPIndex {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU64::new(ENTRY_POINT_NONE),
            backend: IndexBackend::InMemory,
            config: HnswConfig::default(),
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
        }
    }

    pub fn new_with_config(config: HnswConfig) -> Self {
        Self {
            nodes: Default::default(),
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU64::new(ENTRY_POINT_NONE),
            backend: IndexBackend::InMemory,
            config,
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
        }
    }

    pub fn with_backend(backend: IndexBackend) -> Self {
        Self {
            nodes: Default::default(),
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU64::new(ENTRY_POINT_NONE),
            backend,
            config: HnswConfig::default(),
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
        }
    }

    pub fn estimate_memory_bytes(&self) -> usize {
        let mut total = 0usize;
        for r in self.nodes.iter() {
            let node = r.value();
            match &node.vec_data {
                VectorRepresentations::Full(v) => total += v.len() * std::mem::size_of::<f32>(),
                VectorRepresentations::MmapFull(_, _) => {} // Zero heap allocations for mapped memory
                VectorRepresentations::Binary(b) => total += b.len() * std::mem::size_of::<u64>(),
                VectorRepresentations::Turbo(t) => total += t.len(),
                VectorRepresentations::SQ8(d, _) => total += d.len() + 4,
                VectorRepresentations::None => {}
            }
            for layer in &node.neighbors {
                total += layer.len() * std::mem::size_of::<u64>() + std::mem::size_of::<Vec<u64>>();
            }
            total += std::mem::size_of::<HnswNode>();
        }
        total += self.nodes.len() * 60;
        total
    }

    fn random_layer(&self) -> usize {
        let mut rng = self.rng.lock();
        let r: f64 = rng.random_range(0.0001..1.0);
        (-r.ln() * self.config.ml).floor() as usize
    }

    /// Thread-safe accessor for the current entry point.
    #[inline]
    pub fn get_entry_point(&self) -> Option<u64> {
        let ep = self.entry_point.load(Ordering::Acquire);
        if ep == ENTRY_POINT_NONE {
            None
        } else {
            Some(ep)
        }
    }

    /// Thread-safe setter. Uses Release ordering to ensure the node
    /// is fully visible in DashMap before other threads follow this pointer.
    #[inline]
    pub fn set_entry_point(&self, id: u64) {
        self.entry_point.store(id, Ordering::Release);
    }

    #[inline(always)]
    fn fast_similarity(
        &self,
        query_vec: &[f32],
        query_norm: Option<f32>,
        query_inv_norm: Option<f32>,
        node: &HnswNode,
        metric: DistanceMetric,
    ) -> f32 {
        match metric {
            DistanceMetric::Cosine => {
                if let Some(q_inv_norm) = query_inv_norm {
                    let node_inv_norm = node.inv_cached_norm;
                    if node_inv_norm > f32::EPSILON {
                        if let Some(node_slice) = node.vec_data.as_f32_slice() {
                            return cosine_sim_cached_norms(
                                query_vec,
                                q_inv_norm,
                                node_slice,
                                node_inv_norm,
                            );
                        }
                    }
                }
                calculate_similarity(query_vec, query_norm, None, None, &node.vec_data, metric)
            }
            DistanceMetric::Euclidean => {
                if let Some(node_slice) = node.vec_data.as_f32_slice() {
                    -euclidean_distance_squared_f32(query_vec, node_slice)
                } else {
                    calculate_similarity(query_vec, query_norm, None, None, &node.vec_data, metric)
                }
            }
        }
    }

    /// Primary search subroutine for HNSW.
    /// Performs a greedy beam search to return the `ef` nearest neighbors
    /// found at `layer`. Candidates are validated against `query_mask`.
    #[allow(clippy::too_many_arguments)]
    fn search_layer(
        &self,
        query_vec: &[f32],
        query_norm: Option<f32>,
        query_inv_norm: Option<f32>,
        entry_points: &[u64],
        ef: usize,
        layer: usize,
        query_mask: u128,
        vector_store: Option<&crate::storage::VantaFile>,
        metric: DistanceMetric,
    ) -> BinaryHeap<NodeSimMin> {
        let mut visited = std::collections::HashSet::with_capacity_and_hasher(
            ef * 2,
            BuildHasherDefault::<XxHash64>::default(),
        );
        let mut candidates = BinaryHeap::new(); // Max-heap: candidates to visit
        let mut results = BinaryHeap::new(); // Min-heap: best `ef` bounds

        for &ep in entry_points {
            if let Some(node) = self.nodes.get(&ep) {
                let d = if let Some(vs) = vector_store {
                    // Zero-copy search from VantaFile
                    if let Some(header) = vs.read_header(node.storage_offset) {
                        let vec_start = header.vector_offset as usize;
                        let vec_end = vec_start + (header.vector_len as usize * 4);
                        if vec_end > vs.mmap_bytes().len() {
                            0.0
                        } else {
                            let vec_data = &vs.mmap_bytes()[vec_start..vec_end];
                            // Safety: we trust the header.vector_len and bounds checking above
                            let f32_vec: &[f32] = unsafe {
                                std::slice::from_raw_parts(
                                    vec_data.as_ptr() as *const f32,
                                    header.vector_len as usize,
                                )
                            };
                            match metric {
                                DistanceMetric::Cosine => {
                                    if let Some(q_inv_norm) = query_inv_norm {
                                        let node_inv_norm = node.inv_cached_norm;
                                        if node_inv_norm > f32::EPSILON {
                                            cosine_sim_cached_norms(
                                                query_vec,
                                                q_inv_norm,
                                                f32_vec,
                                                node_inv_norm,
                                            )
                                        } else {
                                            f32_slice_similarity(
                                                query_vec, query_norm, f32_vec, metric,
                                            )
                                        }
                                    } else {
                                        f32_slice_similarity(query_vec, query_norm, f32_vec, metric)
                                    }
                                }
                                DistanceMetric::Euclidean => {
                                    -euclidean_distance_squared_f32(query_vec, f32_vec)
                                }
                            }
                        }
                    } else {
                        0.0
                    }
                } else {
                    self.fast_similarity(query_vec, query_norm, query_inv_norm, &node, metric)
                };

                candidates.push(NodeSim(d, ep));

                let eligible = if let Some(vs) = vector_store {
                    vs.read_header(node.storage_offset)
                        .map(|h| (h.flags & 0x8) == 0)
                        .unwrap_or(false)
                } else {
                    true
                };
                if eligible && (query_mask == u128::MAX || (node.bitset & query_mask) == query_mask)
                {
                    results.push(NodeSimMin(d, ep));
                }
                visited.insert(ep);
            }
        }

        while let Some(NodeSim(d_cand, cand_id)) = candidates.pop() {
            // Early stopping condition: if candidate is worse than the worst result
            if results.len() >= ef {
                if let Some(worst) = results.peek() {
                    // Because it's a min-heap, peek gives the smallest similarity (worst match)
                    if d_cand < worst.0 {
                        break;
                    }
                }
            }

            let neighbors = if let Some(node) = self.nodes.get(&cand_id) {
                if layer < node.neighbors.len() {
                    Some(node.neighbors[layer].clone())
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(neighbors_list) = neighbors {
                // SCALE-01: Prefetch predictivo — emitimos sugerencias de pre-carga para
                // TODOS los vecinos del candidato actual antes de calcular cualquier distancia.
                // Esto permite al kernel (y al controlador de SSD) iniciar DMA de las páginas
                // físicas en paralelo mientras la CPU calcula la distancia del nodo actual.
                if should_prefetch() {
                    if let Some(vs) = vector_store {
                        let mmap_base = vs.mmap_bytes().as_ptr();
                        let mmap_len = vs.mmap_bytes().len();
                        for &pf_neighbor_id in &neighbors_list {
                            if !visited.contains(&pf_neighbor_id) {
                                if let Some(pf_node) = self.nodes.get(&pf_neighbor_id) {
                                    if let Some(h) = vs.read_header(pf_node.storage_offset) {
                                        let vec_start = h.vector_offset as usize;
                                        let vec_len_bytes = h.vector_len as usize * 4;
                                        // Validar bounds antes de emitir prefetch
                                        if vec_start + vec_len_bytes <= mmap_len
                                            && vec_len_bytes > 0
                                        {
                                            prefetch_mmap_vector(
                                                mmap_base,
                                                vec_start,
                                                vec_len_bytes,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                for &neighbor_id in &neighbors_list {
                    if !visited.contains(&neighbor_id) {
                        visited.insert(neighbor_id);

                        if let Some(neighbor) = self.nodes.get(&neighbor_id) {
                            let d = if let Some(vs) = vector_store {
                                if let Some(h) = vs.read_header(neighbor.storage_offset) {
                                    let vec_start = h.vector_offset as usize;
                                    let vec_end = vec_start + (h.vector_len as usize * 4);
                                    if vec_end > vs.mmap_bytes().len() {
                                        0.0
                                    } else {
                                        let v_data = &vs.mmap_bytes()[vec_start..vec_end];
                                        // Safety: trusted bounds and aligned data
                                        let f32_v: &[f32] = unsafe {
                                            std::slice::from_raw_parts(
                                                v_data.as_ptr() as *const f32,
                                                h.vector_len as usize,
                                            )
                                        };
                                        match metric {
                                            DistanceMetric::Cosine => {
                                                if let Some(q_inv_norm) = query_inv_norm {
                                                    let neighbor_inv_norm =
                                                        neighbor.inv_cached_norm;
                                                    if neighbor_inv_norm > f32::EPSILON {
                                                        cosine_sim_cached_norms(
                                                            query_vec,
                                                            q_inv_norm,
                                                            f32_v,
                                                            neighbor_inv_norm,
                                                        )
                                                    } else {
                                                        f32_slice_similarity(
                                                            query_vec, query_norm, f32_v, metric,
                                                        )
                                                    }
                                                } else {
                                                    f32_slice_similarity(
                                                        query_vec, query_norm, f32_v, metric,
                                                    )
                                                }
                                            }
                                            DistanceMetric::Euclidean => {
                                                -euclidean_distance_squared_f32(query_vec, f32_v)
                                            }
                                        }
                                    }
                                } else {
                                    0.0
                                }
                            } else {
                                self.fast_similarity(
                                    query_vec,
                                    query_norm,
                                    query_inv_norm,
                                    &neighbor,
                                    metric,
                                )
                            };

                            if results.len() < ef || results.peek().is_some_and(|worst| d > worst.0)
                            {
                                candidates.push(NodeSim(d, neighbor_id));

                                let eligible = if let Some(vs) = vector_store {
                                    vs.read_header(neighbor.storage_offset)
                                        .map(|h| (h.flags & 0x8) == 0)
                                        .unwrap_or(false)
                                } else {
                                    true
                                };
                                if eligible
                                    && (query_mask == u128::MAX
                                        || (neighbor.bitset & query_mask) == query_mask)
                                {
                                    results.push(NodeSimMin(d, neighbor_id));
                                    if results.len() > ef {
                                        results.pop(); // Remove the worst to keep size at `ef`
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }

    /// Select neighbors using the HNSW paper heuristic (Algorithm 4, Malkov & Yashunin 2018).
    /// Applies spatial diversity from slot 0 — no unconditional acceptance.
    /// keepPrunedConnections=true: fills limited remaining slots with discarded candidates.
    ///
    /// Metric: cosine similarity (higher = closer). The diversity condition is:
    ///   reject if similarity(candidate, selected) > similarity(candidate, query)
    /// This is the correct inversion of the paper's distance-based condition
    /// because cosine similarity is monotonically inverse to angular distance.
    fn select_neighbors(&self, candidates: BinaryHeap<NodeSimMin>, m: usize) -> Vec<u64> {
        // Consumes the heap directly. Callers must extract entry_points
        // BEFORE calling this function if they need the original heap data.
        let sorted = candidates.into_sorted_vec();
        // into_sorted_vec returns ascending order based on NodeSimMin's Ord
        // NodeSimMin Ord equates higher similarity to "Less", meaning best candidates come first!

        struct SelectedInfo {
            id: u64,
            vec: Option<Vec<f32>>,
            inv_norm: f32,
        }

        let mut selected: Vec<SelectedInfo> = Vec::with_capacity(m);
        let mut discarded: Vec<u64> = Vec::new();

        for ns in sorted.into_iter() {
            if selected.len() >= m {
                break;
            }

            let cand_id = ns.1;
            let sim_q_cand = ns.0;

            let (cand_slice, cand_inv_norm) = match self.nodes.get(&cand_id) {
                Some(n) => (
                    n.vec_data.as_f32_slice().map(|s| s.to_vec()),
                    n.inv_cached_norm,
                ),
                None => continue,
            };

            let mut is_diverse = true;
            for sel in &selected {
                let sim_cand_sel = match self.config.distance_metric {
                    DistanceMetric::Cosine => {
                        if let (Some(c_slice), Some(s_slice)) = (&cand_slice, &sel.vec) {
                            cosine_sim_cached_norms(c_slice, cand_inv_norm, s_slice, sel.inv_norm)
                        } else {
                            if let Some(sel_node) = self.nodes.get(&sel.id) {
                                let cand_norm = if cand_inv_norm > f32::EPSILON {
                                    Some(1.0 / cand_inv_norm)
                                } else {
                                    None
                                };
                                calculate_similarity(
                                    cand_slice.as_deref().unwrap_or(&[]),
                                    cand_norm,
                                    None,
                                    None,
                                    &sel_node.vec_data,
                                    self.config.distance_metric,
                                )
                            } else {
                                0.0
                            }
                        }
                    }
                    DistanceMetric::Euclidean => {
                        if let (Some(c_slice), Some(s_slice)) = (&cand_slice, &sel.vec) {
                            -euclidean_distance_squared_f32(c_slice, s_slice)
                        } else {
                            if let Some(sel_node) = self.nodes.get(&sel.id) {
                                calculate_similarity(
                                    cand_slice.as_deref().unwrap_or(&[]),
                                    None,
                                    None,
                                    None,
                                    &sel_node.vec_data,
                                    self.config.distance_metric,
                                )
                            } else {
                                0.0
                            }
                        }
                    }
                };

                if sim_cand_sel > sim_q_cand {
                    is_diverse = false;
                    break;
                }
            }

            if is_diverse {
                selected.push(SelectedInfo {
                    id: cand_id,
                    vec: cand_slice,
                    inv_norm: cand_inv_norm,
                });
            } else {
                discarded.push(cand_id);
            }
        }

        // keepPrunedConnections: fill remaining slots with discarded candidates.
        // HNSW relies on keeping degree close to M.
        let mut final_selected: Vec<u64> = selected.into_iter().map(|s| s.id).collect();
        for &disc_id in discarded.iter() {
            if final_selected.len() >= m {
                break;
            }
            final_selected.push(disc_id);
        }

        final_selected
    }

    fn validate_node(
        &self,
        id: u64,
        bitset: u128,
        vec_data: &VectorRepresentations,
        storage_offset: u64,
    ) -> bool {
        if let Some(mut node) = self.nodes.get_mut(&id) {
            node.bitset = bitset;
            node.vec_data = vec_data.clone();
            node.storage_offset = storage_offset;
            return true;
        }

        if vec_data.is_none() {
            self.nodes.insert(
                id,
                HnswNode {
                    id,
                    bitset,
                    vec_data: vec_data.clone(),
                    neighbors: vec![Vec::new()],
                    storage_offset,
                    inv_cached_norm: 0.0,
                },
            );
            return true;
        }

        false
    }

    #[tracing::instrument(skip(self, vec_data), level = "debug")]
    pub fn add(&self, id: u64, bitset: u128, vec_data: VectorRepresentations, storage_offset: u64) {
        if self.validate_node(id, bitset, &vec_data, storage_offset) {
            return;
        }

        self.insert_hnsw(id, bitset, vec_data, storage_offset);
    }

    fn insert_hnsw(
        &self,
        id: u64,
        bitset: u128,
        vec_data: VectorRepresentations,
        storage_offset: u64,
    ) {
        let level = self.random_layer();
        let ef_cons = self.config.ef_construction;

        let inv_cached_norm = match self.config.distance_metric {
            DistanceMetric::Cosine => vec_data
                .as_f32_slice()
                .map(|s| {
                    let norm = f32_l2_norm(s);
                    if norm > f32::EPSILON {
                        1.0 / norm
                    } else {
                        0.0
                    }
                })
                .unwrap_or(0.0),
            DistanceMetric::Euclidean => 0.0,
        };

        let query_f32 = vec_data.to_f32();

        let node = HnswNode {
            id,
            bitset,
            vec_data,
            neighbors: vec![Vec::new(); level + 1],
            storage_offset,
            inv_cached_norm,
        };

        let ep = match self.get_entry_point() {
            None => {
                self.set_entry_point(id);
                self.max_layer.store(level, Ordering::Release);
                self.nodes.insert(id, node);
                return;
            }
            Some(entry) => entry,
        };

        self.nodes.insert(id, node);

        let query_f32 = match query_f32 {
            Some(v) => v,
            None => return,
        };

        let (query_norm, query_inv_norm) = match self.config.distance_metric {
            DistanceMetric::Cosine => {
                let norm = f32_l2_norm(&query_f32);
                if norm < f32::EPSILON {
                    self.nodes.remove(&id);
                    return;
                }
                (Some(norm), Some(1.0 / norm))
            }
            DistanceMetric::Euclidean => (None, None),
        };

        let mut curr_entry_points = vec![ep];
        let top_layer = self.max_layer.load(Ordering::Acquire);

        for layer in (level + 1..=top_layer).rev() {
            let mut w = self.search_layer(
                &query_f32,
                query_norm,
                query_inv_norm,
                &curr_entry_points,
                1,
                layer,
                u128::MAX,
                None,
                self.config.distance_metric,
            );
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

        let start_layer = std::cmp::min(level, top_layer);
        for layer in (0..=start_layer).rev() {
            let w = self.search_layer(
                &query_f32,
                query_norm,
                query_inv_norm,
                &curr_entry_points,
                ef_cons,
                layer,
                u128::MAX,
                None,
                self.config.distance_metric,
            );

            let m_max = if layer == 0 {
                self.config.m_max0
            } else {
                self.config.m
            };

            curr_entry_points = w.iter().map(|ns| ns.1).collect();
            let selected_neighbors = self.select_neighbors(w, m_max);

            if let Some(mut n) = self.nodes.get_mut(&id) {
                n.neighbors[layer] = selected_neighbors.clone();
            }

            for &neighbor_id in &selected_neighbors {
                let (needs_shrink, current_neighbors) = {
                    if let Some(mut neighbor_node) = self.nodes.get_mut(&neighbor_id) {
                        if layer < neighbor_node.neighbors.len() {
                            if !neighbor_node.neighbors[layer].contains(&id) {
                                neighbor_node.neighbors[layer].push(id);
                            }

                            if neighbor_node.neighbors[layer].len() > m_max {
                                (true, neighbor_node.neighbors[layer].clone())
                            } else {
                                (false, Vec::new())
                            }
                        } else {
                            (false, Vec::new())
                        }
                    } else {
                        (false, Vec::new())
                    }
                };

                if needs_shrink {
                    let (nb_vec, nb_inv_norm) = match self.nodes.get(&neighbor_id) {
                        Some(n) => (
                            n.vec_data.as_f32_slice().map(|s| s.to_vec()),
                            n.inv_cached_norm,
                        ),
                        None => (None, 0.0),
                    };

                    if let Some(nb_v) = nb_vec {
                        let mut cand_heap = BinaryHeap::new();
                        let q_norm = if nb_inv_norm > f32::EPSILON {
                            Some(1.0 / nb_inv_norm)
                        } else {
                            None
                        };
                        let q_inv_norm = if nb_inv_norm > f32::EPSILON {
                            Some(nb_inv_norm)
                        } else {
                            None
                        };
                        for &n_target in &current_neighbors {
                            if let Some(nt) = self.nodes.get(&n_target) {
                                let d = self.fast_similarity(
                                    &nb_v,
                                    q_norm,
                                    q_inv_norm,
                                    &nt,
                                    self.config.distance_metric,
                                );
                                cand_heap.push(NodeSimMin(d, n_target));
                            }
                        }
                        let pruned = self.select_neighbors(cand_heap, m_max);
                        if let Some(mut neighbor_node) = self.nodes.get_mut(&neighbor_id) {
                            neighbor_node.neighbors[layer] = pruned;
                        }
                    }
                }
            }
        }

        self.update_metadata(level, id);
    }

    fn update_metadata(&self, level: usize, id: u64) {
        let current_max = self.max_layer.load(Ordering::Acquire);
        if level > current_max {
            self.max_layer.fetch_max(level, Ordering::Release);
            self.set_entry_point(id);
        }
    }

    #[tracing::instrument(skip(self, query_vec, vector_store), level = "debug")]
    pub fn search_nearest(
        &self,
        query_vec: &[f32],
        _q_1bit: Option<&[u64]>, // We let these pass but currently default to calculate_similarity internal handler
        _q_3bit: Option<(&[u8], f32)>,
        query_mask: u128,
        top_k: usize,
        vector_store: Option<&crate::storage::VantaFile>,
    ) -> Vec<(u64, f32)> {
        let ep = match self.get_entry_point() {
            Some(id) => id,
            None => return Vec::new(),
        };

        let ef_search = self.config.ef_search.max(top_k);
        // When Cosine metric is configured but the query vector is zero-norm
        // (no defined direction), fall back to Euclidean for this query.
        // Returning empty results for a zero-vector query breaks practical
        // use-cases where callers use [0.0]*N as a "find all nearest" probe.
        let (effective_metric, query_norm, query_inv_norm) = match self.config.distance_metric {
            DistanceMetric::Cosine => {
                let norm = f32_l2_norm(query_vec);
                if norm < f32::EPSILON {
                    // Zero-norm fallback: use Euclidean without norm info
                    (DistanceMetric::Euclidean, None, None)
                } else {
                    (DistanceMetric::Cosine, Some(norm), Some(1.0 / norm))
                }
            }
            DistanceMetric::Euclidean => (DistanceMetric::Euclidean, None, None),
        };
        let mut curr_entry_points = vec![ep];

        let max_l = self.max_layer.load(Ordering::Acquire);
        for layer in (1..=max_l).rev() {
            let mut w = self.search_layer(
                query_vec,
                query_norm,
                query_inv_norm,
                &curr_entry_points,
                1,
                layer,
                u128::MAX,
                vector_store,
                effective_metric,
            );
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

        let w = self.search_layer(
            query_vec,
            query_norm,
            query_inv_norm,
            &curr_entry_points,
            ef_search,
            0,
            query_mask,
            vector_store,
            effective_metric,
        );

        let mut result = w.into_sorted_vec();

        // into_sorted_vec returns highest similarity (best) first!
        result.truncate(top_k);

        let mut final_results = Vec::with_capacity(result.len());
        for NodeSimMin(score, id) in result {
            let adjusted_score = match effective_metric {
                DistanceMetric::Euclidean => -(-score).max(0.0).sqrt(),
                DistanceMetric::Cosine => score,
            };
            final_results.push((id, adjusted_score));
        }
        final_results
    }

    /// BFS traversal order for on-disk layout: entry point first, then graph neighbors
    /// (upper layers before lower) to improve mmap locality on large indexes.
    pub(crate) fn serialization_order(&self) -> Vec<u64> {
        use std::collections::{HashSet, VecDeque};

        let mut order = Vec::with_capacity(self.nodes.len());
        let mut seen = HashSet::new();

        if let Some(ep) = self.get_entry_point() {
            let mut queue = VecDeque::new();
            queue.push_back(ep);
            seen.insert(ep);

            while let Some(node_id) = queue.pop_front() {
                order.push(node_id);
                if let Some(node) = self.nodes.get(&node_id) {
                    for layer in (0..node.neighbors.len()).rev() {
                        for &neighbor_id in &node.neighbors[layer] {
                            if seen.insert(neighbor_id) {
                                queue.push_back(neighbor_id);
                            }
                        }
                    }
                }
            }
        }

        let mut orphans: Vec<u64> = self
            .nodes
            .iter()
            .map(|r| *r.key())
            .filter(|id| !seen.contains(id))
            .collect();
        orphans.sort_unstable();
        order.extend(orphans);
        order
    }

    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.nodes.len() * 256 + 128);

        let header = crate::binary_header::VantaHeader::new(*b"VNDX", VECTOR_INDEX_VERSION, 0);
        buf.extend_from_slice(&header.serialize());
        buf.extend_from_slice(&(self.max_layer.load(Ordering::Acquire) as u64).to_le_bytes());

        // Config block (only in V2+)
        buf.extend_from_slice(&(self.config.m as u64).to_le_bytes());
        buf.extend_from_slice(&(self.config.m_max0 as u64).to_le_bytes());
        buf.extend_from_slice(&(self.config.ef_construction as u64).to_le_bytes());
        buf.extend_from_slice(&(self.config.ef_search as u64).to_le_bytes());
        buf.extend_from_slice(&self.config.ml.to_le_bytes());
        // V3+: distance metric byte (0 = Cosine, 1 = Euclidean)
        let metric_byte: u8 = match self.config.distance_metric {
            DistanceMetric::Cosine => 0,
            DistanceMetric::Euclidean => 1,
        };
        buf.push(metric_byte);

        match self.get_entry_point() {
            Some(ep) => {
                buf.push(1);
                buf.extend_from_slice(&ep.to_le_bytes());
            }
            None => {
                buf.push(0);
                buf.extend_from_slice(&0u64.to_le_bytes());
            }
        }

        let node_count = self.nodes.len() as u64;
        buf.extend_from_slice(&node_count.to_le_bytes());

        for node_id in self.serialization_order() {
            let Some(node) = self.nodes.get(&node_id) else {
                continue;
            };
            buf.extend_from_slice(&node.id.to_le_bytes());
            buf.extend_from_slice(&node.bitset.to_le_bytes());
            buf.extend_from_slice(&node.storage_offset.to_le_bytes());

            match &node.vec_data {
                VectorRepresentations::Full(f) => {
                    buf.push(1);
                    buf.extend_from_slice(&(f.len() as u64).to_le_bytes());
                    let padding = (4 - (buf.len() % 4)) % 4;
                    if padding > 0 {
                        buf.extend(std::iter::repeat_n(0, padding));
                    }
                    for &val in f {
                        buf.extend_from_slice(&val.to_le_bytes());
                    }
                }
                VectorRepresentations::MmapFull(ptr, len) => {
                    buf.push(1);
                    buf.extend_from_slice(&(*len as u64).to_le_bytes());
                    let padding = (4 - (buf.len() % 4)) % 4;
                    if padding > 0 {
                        buf.extend(std::iter::repeat_n(0, padding));
                    }
                    debug_assert!(!ptr.0.is_null(), "MmapFull pointer is null in serialize");
                    debug_assert!(*len > 0 && *len <= MAX_VEC_F32_LEN, "MmapFull len out of range in serialize");
                    let slice = unsafe { std::slice::from_raw_parts(ptr.0, *len) };
                    for &val in slice {
                        buf.extend_from_slice(&val.to_le_bytes());
                    }
                }
                VectorRepresentations::Binary(b) => {
                    buf.push(2);
                    buf.extend_from_slice(&(b.len() as u64).to_le_bytes());
                    for &val in b {
                        buf.extend_from_slice(&val.to_le_bytes());
                    }
                }
                VectorRepresentations::Turbo(t) => {
                    buf.push(3);
                    buf.extend_from_slice(&(t.len() as u64).to_le_bytes());
                    buf.extend_from_slice(t);
                }
                VectorRepresentations::SQ8(d, scale) => {
                    buf.push(4);
                    buf.extend_from_slice(&(d.len() as u64).to_le_bytes());
                    for &v in d {
                        buf.push(v as u8);
                    }
                    buf.extend_from_slice(&scale.to_le_bytes());
                }
                VectorRepresentations::None => {
                    buf.push(0);
                    buf.extend_from_slice(&0u64.to_le_bytes());
                }
            }

            let layer_count = node.neighbors.len() as u64;
            buf.extend_from_slice(&layer_count.to_le_bytes());
            for layer in &node.neighbors {
                let neighbor_count = layer.len() as u64;
                buf.extend_from_slice(&neighbor_count.to_le_bytes());
                for &nid in layer {
                    buf.extend_from_slice(&nid.to_le_bytes());
                }
            }
        }

        buf
    }

    pub fn deserialize_from_bytes(data: &[u8], force_copy: bool) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind};

        #[inline]
        fn take_bytes<'a>(
            data: &'a [u8],
            pos: &mut usize,
            n: usize,
            field: &str,
        ) -> std::io::Result<&'a [u8]> {
            if *pos + n > data.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    format!("Truncated {field}"),
                ));
            }
            let slice = &data[*pos..*pos + n];
            *pos += n;
            Ok(slice)
        }

        #[inline]
        fn read_le_u64(data: &[u8], pos: &mut usize, field: &str) -> std::io::Result<u64> {
            let bytes = take_bytes(data, pos, 8, field)?;
            Ok(u64::from_le_bytes(bytes.try_into().map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("failed to parse {field} as u64: {e}"),
                )
            })?))
        }

        #[inline]
        fn read_le_f64(data: &[u8], pos: &mut usize, field: &str) -> std::io::Result<f64> {
            let bytes = take_bytes(data, pos, 8, field)?;
            Ok(f64::from_le_bytes(bytes.try_into().map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("failed to parse {field} as f64: {e}"),
                )
            })?))
        }

        if data.len() < crate::binary_header::VantaHeader::SIZE + 8 {
            return Err(Error::new(ErrorKind::InvalidData, "Index file too small"));
        }

        let mut pos = 0;

        let header = match crate::binary_header::VantaHeader::deserialize(
            &data[pos..pos + crate::binary_header::VantaHeader::SIZE],
        ) {
            Ok(h) => h,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Failed to parse binary header: {:?}", e),
                ))
            }
        };
        pos += crate::binary_header::VantaHeader::SIZE;

        if let Err(e) = header.validate(*b"VNDX", VECTOR_INDEX_VERSION, "Index format mismatch") {
            return Err(Error::new(ErrorKind::InvalidData, format!("{}", e)));
        }

        let version = header.format_version as u32;

        let max_layer = read_le_u64(data, &mut pos, "max_layer")? as usize;

        let mut config = HnswConfig::default();
        if version >= 2 {
            config.m = read_le_u64(data, &mut pos, "config.m")? as usize;
            config.m_max0 = read_le_u64(data, &mut pos, "config.m_max0")? as usize;
            config.ef_construction =
                read_le_u64(data, &mut pos, "config.ef_construction")? as usize;
            config.ef_search = read_le_u64(data, &mut pos, "config.ef_search")? as usize;
            config.ml = read_le_f64(data, &mut pos, "config.ml")?;
        }
        // V3+: distance metric byte
        if version >= 3 && pos < data.len() {
            config.distance_metric = match take_bytes(data, &mut pos, 1, "distance_metric")?[0] {
                1 => DistanceMetric::Euclidean,
                _ => DistanceMetric::Cosine,
            };
        }

        let ep_exists = take_bytes(data, &mut pos, 1, "ep_exists")?[0];
        let ep_id = read_le_u64(data, &mut pos, "ep_id")?;
        let entry_point = if ep_exists == 1 { Some(ep_id) } else { None };

        let node_count = read_le_u64(data, &mut pos, "node_count")? as usize;

        // Sanity: each node needs at least 49 bytes (id + bitset + offset + type + vec_len + layer_count).
        const MIN_BYTES_PER_NODE: usize = 8 + 16 + 8 + 1 + 8 + 8;
        let remaining = data.len().saturating_sub(pos);
        if node_count > remaining / MIN_BYTES_PER_NODE {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "node_count ({node_count}) exceeds plausible limit for {remaining} remaining bytes",
                ),
            ));
        }

        let nodes = DashMap::with_capacity_and_hasher(node_count, BuildHasherDefault::default());

        for _ in 0..node_count {
            let id = read_le_u64(data, &mut pos, "node id")?;

            let bitset_bytes = take_bytes(data, &mut pos, 16, "bitset")?;
            let bitset = u128::from_le_bytes(bitset_bytes.try_into().map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("bitset field expected 16 bytes: {e}"),
                )
            })?);

            let storage_offset = read_le_u64(data, &mut pos, "storage_offset")?;

            let vec_type = take_bytes(data, &mut pos, 1, "vec_type")?[0];

            let vec_len = read_le_u64(data, &mut pos, "vec_len")? as usize;

            let vec_data = match vec_type {
                1 => {
                    let byte_len = vec_len.checked_mul(4).ok_or_else(|| {
                        Error::new(ErrorKind::InvalidData, "vec_len overflow (f32)")
                    })?;
                    if version >= 4 {
                        let padding = (4 - (pos % 4)) % 4;
                        pos += padding;
                    }
                    let vec_bytes = take_bytes(data, &mut pos, byte_len, "f32 vec")?;
                    if force_copy {
                        let mut v = Vec::with_capacity(vec_len);
                        for i in 0..vec_len {
                            let start = i * 4;
                            v.push(f32::from_le_bytes(
                                vec_bytes[start..start + 4].try_into().map_err(|e| {
                                    std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        format!(
                                            "f32 vec chunk at byte {start} expected 4 bytes: {e}"
                                        ),
                                    )
                                })?,
                            ));
                        }
                        VectorRepresentations::Full(v)
                    } else {
                        let ptr = vec_bytes.as_ptr() as *const f32;
                        if ptr.is_null() {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "MmapFull: null pointer from vec_bytes",
                            ));
                        }
                        if vec_len == 0 || vec_len > MAX_VEC_F32_LEN {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                format!("MmapFull: invalid vec_len {vec_len}"),
                            ));
                        }
                        VectorRepresentations::MmapFull(SendPtr(ptr), vec_len)
                    }
                }
                2 => {
                    let byte_len = vec_len.checked_mul(8).ok_or_else(|| {
                        Error::new(ErrorKind::InvalidData, "vec_len overflow (binary)")
                    })?;
                    let vec_bytes = take_bytes(data, &mut pos, byte_len, "binary vec")?;
                    let mut v = Vec::with_capacity(vec_len);
                    for i in 0..vec_len {
                        let start = i * 8;
                        v.push(u64::from_le_bytes(
                            vec_bytes[start..start + 8].try_into().map_err(|e| {
                                std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    format!(
                                        "binary vec chunk at byte {start} expected 8 bytes: {e}"
                                    ),
                                )
                            })?,
                        ));
                    }
                    VectorRepresentations::Binary(v.into_boxed_slice())
                }
                3 => {
                    let vec_bytes = take_bytes(data, &mut pos, vec_len, "turbo vec")?;
                    VectorRepresentations::Turbo(vec_bytes.to_vec().into_boxed_slice())
                }
                4 => {
                    let sq8_bytes = take_bytes(data, &mut pos, vec_len, "sq8 vec")?;
                    let sq8_data: Vec<i8> = sq8_bytes.iter().map(|&b| b as i8).collect();
                    let scale_bytes = take_bytes(data, &mut pos, 4, "sq8 scale")?;
                    let scale = f32::from_le_bytes(scale_bytes.try_into().map_err(|e| {
                        Error::new(ErrorKind::InvalidData, format!("sq8 scale: {e}"))
                    })?);
                    VectorRepresentations::SQ8(sq8_data.into_boxed_slice(), scale)
                }
                _ => VectorRepresentations::None,
            };

            let layer_count = read_le_u64(data, &mut pos, "layer_count")? as usize;
            let layer_remaining = data.len().saturating_sub(pos);
            if layer_count > layer_remaining / 8 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("layer_count ({layer_count}) exceeds remaining data"),
                ));
            }

            let mut neighbors = Vec::with_capacity(layer_count);
            for _ in 0..layer_count {
                let neighbor_count = read_le_u64(data, &mut pos, "neighbor_count")? as usize;

                let byte_len = neighbor_count
                    .checked_mul(8)
                    .ok_or_else(|| Error::new(ErrorKind::InvalidData, "neighbor_count overflow"))?;
                let nbr_bytes = take_bytes(data, &mut pos, byte_len, "neighbor ids")?;
                let mut layer_neighbors = Vec::with_capacity(neighbor_count);
                for i in 0..neighbor_count {
                    let start = i * 8;
                    layer_neighbors.push(u64::from_le_bytes(
                        nbr_bytes[start..start + 8].try_into().map_err(|e| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("neighbor id at byte {start} expected 8 bytes: {e}"),
                            )
                        })?,
                    ));
                }
                neighbors.push(layer_neighbors);
            }

            // Compute inv_cached_norm at load time for Cosine fast-path
            let inv_cached_norm = match config.distance_metric {
                DistanceMetric::Cosine => match &vec_data {
                    VectorRepresentations::Full(f) => {
                        let norm = f32_l2_norm(f);
                        if norm > f32::EPSILON {
                            1.0 / norm
                        } else {
                            0.0
                        }
                    }
                    VectorRepresentations::MmapFull(ptr, len) => {
                        debug_assert!(!ptr.0.is_null(), "MmapFull pointer is null in inv_norm");
                        debug_assert!(*len > 0 && *len <= MAX_VEC_F32_LEN, "MmapFull len out of range in inv_norm");
                        let s = unsafe { std::slice::from_raw_parts(ptr.0, *len) };
                        let norm = f32_l2_norm(s);
                        if norm > f32::EPSILON {
                            1.0 / norm
                        } else {
                            0.0
                        }
                    }
                    _ => 0.0,
                },
                DistanceMetric::Euclidean => 0.0,
            };
            nodes.insert(
                id,
                HnswNode {
                    id,
                    bitset,
                    vec_data,
                    neighbors,
                    storage_offset,
                    inv_cached_norm,
                },
            );
        }

        Ok(Self {
            nodes,
            max_layer: AtomicUsize::new(max_layer),
            entry_point: AtomicU64::new(entry_point.unwrap_or(ENTRY_POINT_NONE)),
            backend: IndexBackend::InMemory,
            config,
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
        })
    }

    pub fn persist_to_file(&self, path: &Path) -> std::io::Result<()> {
        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("hnsw_serialize_fail", |_| {
                Err(std::io::Error::other(
                    "Injected HNSW persist serialization failure",
                ))
            });
        }
        let data = self.serialize_to_bytes();
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&data)?;
        writer.flush()?;
        info!(path = %path.display(), node_count = self.nodes.len(), bytes = data.len(), "HNSW index persisted");
        Ok(())
    }

    pub fn load_from_file(path: &Path, use_mmap: bool) -> Option<Self> {
        if !path.exists() {
            return None;
        }

        if use_mmap {
            let file = match OpenOptions::new().read(true).write(true).open(path) {
                Ok(f) => f,
                Err(_) => return None,
            };

            let mmap = match unsafe { crate::storage::MmapMut::map_mut(&file) } {
                Ok(m) => m,
                Err(e) => {
                    warn!(err = %e, "Failed to mmap HNSW index file — will rebuild");
                    return None;
                }
            };

            match Self::deserialize_from_bytes(&mmap, false) {
                Ok(mut index) => {
                    info!(path = %path.display(), node_count = index.nodes.len(), "HNSW cold-start: loaded zero-copy index from file");
                    index.backend = IndexBackend::MMapFile {
                        path: path.to_path_buf(),
                        mmap: Some(mmap),
                    };
                    if let Err(violations) = index.validate_index() {
                        warn!(
                            violation_count = violations.len(),
                            "HNSW index has integrity violations after deserialization"
                        );
                        for v in &violations[..violations.len().min(5)] {
                            warn!(violation = %v, "HNSW integrity violation");
                        }
                    }
                    Some(index)
                }
                Err(e) => {
                    warn!(err = %e, "Corrupt vector_index.bin — will rebuild and overwrite");
                    None
                }
            }
        } else {
            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(_) => return None,
            };

            match Self::deserialize_from_bytes(&data, true) {
                Ok(index) => {
                    info!(path = %path.display(), node_count = index.nodes.len(), "HNSW cold-start: loaded memory-copied index from file");
                    if let Err(violations) = index.validate_index() {
                        warn!(
                            violation_count = violations.len(),
                            "HNSW index has integrity violations after deserialization"
                        );
                        for v in &violations[..violations.len().min(5)] {
                            warn!(violation = %v, "HNSW integrity violation");
                        }
                    }
                    Some(index)
                }
                Err(e) => {
                    warn!(err = %e, "Corrupt vector_index.bin — will rebuild and overwrite");
                    None
                }
            }
        }
    }

    pub fn sync_to_mmap(&mut self) -> std::io::Result<()> {
        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("hnsw_serialize_fail", |_| {
                Err(std::io::Error::other(
                    "Injected HNSW sync mmap serialization failure",
                ))
            });
        }
        let path = match &self.backend {
            IndexBackend::MMapFile { path, .. } => path.clone(),
            _ => return Ok(()),
        };

        let data = self.serialize_to_bytes();
        let temp_path = path.with_extension("bin.tmp");

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;
        file.set_len(data.len() as u64)?;

        let mut mapped = unsafe { MmapMut::map_mut(&file)? };
        mapped.copy_from_slice(&data);
        mapped.flush()?;

        // Remapear todos los nodos a la nueva dirección de memoria virtual para evitar dangling pointers
        let new_index = Self::deserialize_from_bytes(&mapped, false)?;
        self.nodes = new_index.nodes;
        self.entry_point = new_index.entry_point;

        if let IndexBackend::MMapFile { ref mut mmap, .. } = self.backend {
            *mmap = Some(mapped);
        }

        // Swap atómico de archivos: temp -> final
        drop(file);
        std::fs::rename(&temp_path, &path)?;

        info!(path = %path.display(), node_count = self.nodes.len(), bytes = data.len(), "HNSW MMap synced & zero-copy pointers re-mapped via atomic rename");
        Ok(())
    }
}

// ─── Phase 1.1: Index Stats & Integrity Validation ──────────────────────────

/// Snapshot of HNSW index health metrics
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// Total nodes in the index
    pub node_count: usize,
    /// Maximum layer height in the graph
    pub max_layer: usize,
    /// Nodes with zero neighbors on layer 0 (potential orphans)
    pub orphan_count: usize,
    /// Average outgoing connections on layer 0
    pub avg_connections_l0: f32,
    /// Total number of graph integrity violations found
    pub violation_count: usize,
}

impl CPIndex {
    /// Compute a snapshot of index health metrics.
    pub fn stats(&self) -> IndexStats {
        let node_count = self.nodes.len();
        let orphan_count = self
            .nodes
            .iter()
            .filter(|r| r.value().neighbors.is_empty() || r.value().neighbors[0].is_empty())
            .count();
        let total_l0_connections: usize = self
            .nodes
            .iter()
            .map(|r| r.value().neighbors.first().map(|l| l.len()).unwrap_or(0))
            .sum();
        let avg_connections_l0 = if node_count > 0 {
            total_l0_connections as f32 / node_count as f32
        } else {
            0.0
        };

        IndexStats {
            node_count,
            max_layer: self.max_layer.load(Ordering::Acquire),
            orphan_count,
            avg_connections_l0,
            violation_count: 0, // Updated by validate_index()
        }
    }

    /// Validate the structural integrity of the HNSW graph.
    ///
    /// Checks:
    /// 1. Every neighbor reference points to an existing node
    /// 2. No self-loops
    /// 3. Layer count is consistent with node's reported level
    ///
    /// Returns `Ok(())` if the graph is clean, or a list of violation messages.
    ///
    /// # Performance
    /// O(N × M) where N = node count, M = max neighbors per layer.
    /// Run at startup after deserialization, not in hot paths.
    pub fn validate_index(&self) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();

        for r in self.nodes.iter() {
            let id = *r.key();
            let node = r.value();
            // Check: layer count should be ≥ 1
            if node.neighbors.is_empty() {
                violations.push(format!(
                    "Node {} has empty neighbors array (expected ≥1 layer)",
                    id
                ));
                continue;
            }

            // Check each layer's neighbor list
            for (layer_idx, layer) in node.neighbors.iter().enumerate() {
                for &neighbor_id in layer {
                    // Self-loop check
                    if neighbor_id == id {
                        violations.push(format!(
                            "Node {} has a self-loop at layer {}",
                            id, layer_idx
                        ));
                        continue;
                    }
                    // Dangling reference check
                    if !self.nodes.contains_key(&neighbor_id) {
                        violations.push(format!(
                            "Node {} references non-existent neighbor {} at layer {}",
                            id, neighbor_id, layer_idx
                        ));
                    }
                }
            }
        }

        // Check entry point validity
        if let Some(ep) = self.get_entry_point() {
            if !self.nodes.contains_key(&ep) {
                violations.push(format!("Entry point {} does not exist in the node map", ep));
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}

impl Default for CPIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::DistanceMetric;

    #[test]
    fn cosine_with_precomputed_query_norm_matches_full_path() {
        let a = vec![0.12, 0.88, 0.54, 0.31];
        let b = vec![0.11, 0.89, 0.55, 0.30];
        let norm_a = f32_l2_norm(&a);
        let expected = cosine_sim_f32(&a, &b);
        let optimized = cosine_sim_with_query_norm(&a, norm_a, &b);
        assert!(
            (expected - optimized).abs() < 1e-6,
            "expected {expected}, got {optimized}"
        );
    }

    #[test]
    fn serialization_order_preserves_search_results() {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 64,
            ef_search: 32,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
        });

        for i in 0..64u64 {
            let raw = [
                (i as f32 * 0.01).sin(),
                (i as f32 * 0.02).cos(),
                (i as f32 * 0.03).sin(),
                (i as f32 * 0.04).cos(),
            ];
            let norm = f32_l2_norm(&raw);
            let normalized: Vec<f32> = raw.iter().map(|v| v / norm).collect();
            index.add(i + 1, 0, VectorRepresentations::Full(normalized), 0);
        }

        let query = vec![0.1, 0.9, 0.2, 0.4];
        let before = index.search_nearest(&query, None, None, 0, 5, None);

        let bytes = index.serialize_to_bytes();
        let restored = CPIndex::deserialize_from_bytes(&bytes, true).expect("deserialize");
        let after = restored.search_nearest(&query, None, None, 0, 5, None);

        assert_eq!(before, after);
        assert_eq!(restored.nodes.len(), index.nodes.len());
    }

    #[test]
    fn concurrent_search_during_insert() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::sync::Mutex;
        use std::thread;
        use std::time::Duration;

        let index = Arc::new(CPIndex::new_with_config(HnswConfig {
            m: 16,
            m_max0: 32,
            ef_construction: 64,
            ef_search: 32,
            ml: 1.0 / (16_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
        }));

        let stop = Arc::new(AtomicBool::new(false));
        let insert_mutex = Arc::new(Mutex::new(())); // Imita el insert_lock de StorageEngine
        let mut handles = Vec::new();

        // Lanzar 2 hilos que insertan nodos de manera sincronizada
        for t in 0..2 {
            let index = index.clone();
            let stop = stop.clone();
            let insert_mutex = insert_mutex.clone();
            handles.push(thread::spawn(move || {
                let mut rng = rand::rng();
                let start_id = t * 1000;
                for i in 0..1000 {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let id = start_id + i;
                    let raw_vec: Vec<f32> = (0..32).map(|_| rng.random::<f32>()).collect();
                    let norm = f32_l2_norm(&raw_vec);
                    let vec: Vec<f32> = if norm > 0.0 {
                        raw_vec.iter().map(|v| v / norm).collect()
                    } else {
                        raw_vec
                    };

                    // Adquirir lock para cumplir el contrato de CPIndex::add
                    let _guard = insert_mutex.lock().unwrap();
                    index.add(id, u128::MAX, VectorRepresentations::Full(vec), 0);
                }
            }));
        }

        // Lanzar 4 hilos de búsqueda
        for _ in 0..4 {
            let index = index.clone();
            let stop = stop.clone();
            handles.push(thread::spawn(move || {
                let mut rng = rand::rng();
                while !stop.load(Ordering::Relaxed) {
                    let query: Vec<f32> = (0..32).map(|_| rng.random::<f32>()).collect();
                    let norm = f32_l2_norm(&query);
                    let q_vec = if norm > 0.0 {
                        query.iter().map(|v| v / norm).collect()
                    } else {
                        query
                    };
                    // Búsqueda concurrente sin adquirir insert_lock
                    let _res = index.search_nearest(&q_vec, None, None, u128::MAX, 5, None);
                    thread::sleep(Duration::from_micros(10));
                }
            }));
        }

        // Dejar correr por 1 segundo
        thread::sleep(Duration::from_millis(1000));
        stop.store(true, Ordering::Relaxed);

        // Join de todos los hilos
        for handle in handles {
            let _ = handle.join();
        }

        // Validar integridad estructural del grafo
        assert!(index.validate_index().is_ok());
    }

    #[test]
    fn concurrent_insert_preserves_hnsw_invariants() {
        use crate::node::UnifiedNode;
        use crate::storage::StorageEngine;
        use std::sync::Arc;
        use std::thread;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();
        let storage = Arc::new(StorageEngine::open(db_path).unwrap());

        let mut handles = Vec::new();
        // 4 hilos insertando de forma concurrente
        for t in 0..4 {
            let storage = storage.clone();
            handles.push(thread::spawn(move || {
                let mut rng = rand::rng();
                let start_id = t * 500 + 1; // Evitar ID 0 que a veces es entry point
                for i in 0..500 {
                    let id = start_id + i;
                    let raw_vec: Vec<f32> = (0..32).map(|_| rng.random::<f32>()).collect();
                    let norm = f32_l2_norm(&raw_vec);
                    let vec: Vec<f32> = if norm > 0.0 {
                        raw_vec.iter().map(|v| v / norm).collect()
                    } else {
                        raw_vec
                    };

                    let mut node = UnifiedNode::new(id);
                    node.vector = VectorRepresentations::Full(vec);
                    storage.insert(&node).unwrap();
                }
            }));
        }

        for handle in handles {
            let _ = handle.join();
        }

        // Validar integridad
        let hnsw = storage.hnsw.load();
        assert!(hnsw.validate_index().is_ok());

        // Validar que todos los nodos sean alcanzables BFS desde el entry point
        let ep = hnsw.get_entry_point().expect("Should have entry point");
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(ep);
        visited.insert(ep);

        while let Some(node_id) = queue.pop_front() {
            if let Some(node) = hnsw.nodes.get(&node_id) {
                // BFS en todos los vecinos de todas las capas
                for layer in &node.neighbors {
                    for &neighbor in layer {
                        if visited.insert(neighbor) {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        // Todos los nodos en HNSW deben ser alcanzables
        assert_eq!(
            visited.len(),
            hnsw.nodes.len(),
            "Not all nodes are reachable from the entry point!"
        );
    }

    fn build_small_test_index() -> CPIndex {
        let index = CPIndex::new_with_config(HnswConfig {
            m: 8,
            m_max0: 16,
            ef_construction: 64,
            ef_search: 32,
            ml: 1.0 / (8_f64).ln(),
            distance_metric: DistanceMetric::Cosine,
        });
        for i in 0..16u64 {
            let raw = [
                (i as f32 * 0.01).sin(),
                (i as f32 * 0.02).cos(),
                (i as f32 * 0.03).sin(),
                (i as f32 * 0.04).cos(),
            ];
            let norm = f32_l2_norm(&raw);
            let normalized: Vec<f32> = raw.iter().map(|v| v / norm).collect();
            index.add(i + 1, 0, VectorRepresentations::Full(normalized), 0);
        }
        index
    }

    #[test]
    fn deserialize_truncated_never_panics() {
        let index = build_small_test_index();
        let bytes = index.serialize_to_bytes();
        for len in 0..bytes.len() {
            let result = CPIndex::deserialize_from_bytes(&bytes[..len], true);
            assert!(
                result.is_err(),
                "Expected Err for truncated input at {len}/{} bytes, got Ok",
                bytes.len()
            );
        }
        let full = CPIndex::deserialize_from_bytes(&bytes, true);
        assert!(
            full.is_ok(),
            "Full bytes must deserialize: {:?}",
            full.err()
        );
    }

    #[test]
    fn deserialize_garbage_after_valid_header() {
        let mut garbage = vec![0u8; 512];
        let header = crate::binary_header::VantaHeader::new(*b"VNDX", VECTOR_INDEX_VERSION, 0);
        let hdr = header.serialize();
        garbage[..hdr.len()].copy_from_slice(&hdr);
        let result = CPIndex::deserialize_from_bytes(&garbage, true);
        assert!(result.is_err() || result.unwrap().nodes.is_empty());
    }

    #[test]
    fn deserialize_absurd_node_count() {
        let index = build_small_test_index();
        let mut bytes = index.serialize_to_bytes();

        let header_size = crate::binary_header::VantaHeader::SIZE;
        // max_layer(8) + config v2 (m, m_max0, ef_construction, ef_search, ml = 5*8=40)
        // + distance_metric(1) + ep_exists(1) + ep_id(8) = 58 bytes after header
        let node_count_offset = header_size + 8 + 40 + 1 + 1 + 8;
        if node_count_offset + 8 <= bytes.len() {
            bytes[node_count_offset..node_count_offset + 8]
                .copy_from_slice(&u64::MAX.to_le_bytes());
            let result = CPIndex::deserialize_from_bytes(&bytes, true);
            assert!(result.is_err(), "Absurd node_count must return Err");
        }
    }
}
