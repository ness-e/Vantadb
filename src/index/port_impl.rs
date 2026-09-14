//! `IndexPort`/`MmapBackend` impls + dyn factories (F3X).
//!
//! The index owns its state; the engine consumes it through the leaf traits.
//! This module bridges concrete index types to the leaf — it names NO `storage`
//! item (the `VectorStoreRef for File` impl lives storage-side in `vfile.rs`,
//! so every edge here points downward or stays in-crate).

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

#[cfg(not(feature = "memmap2"))]
use crate::storage::vfile_mmap::MmapMut;
#[cfg(feature = "memmap2")]
use memmap2::MmapMut;
use rand::SeedableRng;

use crate::error::Error;
use crate::index::graph::{CPIndex, IndexBackend};
use crate::index::VecIndex;
use crate::index_port::{FreshHnswReport, IndexPort, IndexType, MmapBackend, VectorStoreRef};
use crate::node::{DistanceMetric, FilterBitset, VectorRepresentations};

// Sealed-trait impls (H3): the ONLY in-crate implementors (see `index_port::sealed`).
impl crate::index_port::sealed::Sealed for CPIndex {}
impl crate::index_port::sealed::Sealed for IndexBackend {}

/// Open an existing index file behind the cycle-breaking handle.
/// Fresh (never-written) paths fall back to the in-memory/mmap constructors below.
/// Returns `Box` (unique ownership for the init phase; `Arc::from` shares it after).
pub(crate) fn open_index_port(path: &Path, use_mmap: bool) -> Result<Box<dyn IndexPort>, Error> {
    let index = CPIndex::open_index(path, use_mmap)?;
    Ok(Box::new(index))
}

/// Fresh in-memory index behind the handle (mirrors the `InMemory` open path).
pub(crate) fn new_in_memory_port() -> Box<dyn IndexPort> {
    Box::new(CPIndex::new())
}

/// Fresh file-mapped index behind the handle (mirrors the fresh-mmap open path).
pub(crate) fn new_mmap_port(path: PathBuf) -> Box<dyn IndexPort> {
    Box::new(CPIndex::with_backend(IndexBackend::new_mmap(path)))
}

impl MmapBackend for IndexBackend {
    fn is_mmap(&self) -> bool {
        // Inherent method wins over the trait method — deliberate delegation.
        self.is_mmap()
    }

    fn mmap_path(&self) -> Option<&Path> {
        // Same: inherent `IndexBackend::mmap_path` (graph/types.rs).
        self.mmap_path()
    }

    fn mmap_resident_bytes(&self) -> Option<u64> {
        // Same: inherent `IndexBackend::mmap_resident_bytes` (graph/types.rs).
        self.mmap_resident_bytes()
    }

    fn sync_to_mmap(&mut self) -> std::io::Result<()> {
        match self {
            // Flush the live mapping; nothing mapped (or in-memory) = no-op success.
            IndexBackend::MMapFile { mmap: Some(m), .. } => m.flush(),
            _ => Ok(()),
        }
    }
}

impl MmapBackend for CPIndex {
    fn is_mmap(&self) -> bool {
        self.backend.is_mmap()
    }

    fn mmap_path(&self) -> Option<&Path> {
        self.backend.mmap_path()
    }

    fn mmap_resident_bytes(&self) -> Option<u64> {
        self.backend.mmap_resident_bytes()
    }

    fn sync_to_mmap(&mut self) -> std::io::Result<()> {
        // Fully-qualified inherent call — no recursion into this trait method.
        CPIndex::sync_to_mmap(self)
    }
}

impl IndexPort for CPIndex {
    fn open_index(path: &Path, use_mmap: bool) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Self::load_from_file(path, use_mmap).ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("index file missing or unreadable: {}", path.display()),
            ))
        })
    }

    fn rebuild_from_bytes(data: &[u8], mmap_path: Option<PathBuf>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut index = Self::deserialize_from_bytes(data, false)?;
        if let Some(path) = mmap_path {
            // Lazy mapping: `mmap_resident_bytes` re-maps on demand (graph/types.rs).
            index.backend = IndexBackend::MMapFile { path, mmap: None };
        }
        Ok(index)
    }

    fn fresh_box(&self, index_path: PathBuf) -> Result<Box<dyn IndexPort>, Error> {
        // Byte-identical logic to the retired `storage::archive::fresh_index_like`.
        let config = self.config.clone();
        let index = if self.backend.is_mmap() {
            let mut idx = Self::with_backend(IndexBackend::new_mmap(index_path));
            idx.config = config;
            idx
        } else {
            Self::new_with_config(config)
        };
        Ok(Box::new(index))
    }

    fn persist_to_file(&self, path: &Path) -> std::io::Result<()> {
        // Inherent method wins over the trait method — deliberate delegation.
        self.persist_to_file(path)
    }

    fn persist_mmap(&self, path: &Path) -> Result<Box<dyn IndexPort>, Error> {
        // Moved `save_vector_index` mmap dance, byte-identical (F3X): serialize to a
        // temp file + mapping, rename into place with no open handles (Windows),
        // re-map the final file and attach it as the live backend. RCU-friendly:
        // builds a NEW index instead of mutating through the shared handle.
        let data = self.serialize_to_bytes();
        let temp_path = path.with_extension("bin.tmp");

        let result = (|| -> std::io::Result<Box<CPIndex>> {
            // Scope the temp file + its mapping so both are released (dropped)
            // BEFORE the rename. Windows refuses `rename` while the source file
            // has ANY open handle — including a live memory map. This ordering is
            // the same proven pattern used in `CPIndex::sync_to_mmap`.
            {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&temp_path)?;
                file.set_len(data.len() as u64)?;

                // SAFETY: `file` is a newly created/truncated handle at `data.len()` bytes.
                // `MmapMut::map_mut` creates a writable mapping of matching size.
                // The mapped memory is immediately initialized via `copy_from_slice` below.
                let mut mapped = unsafe { MmapMut::map_mut(&file)? };
                mapped.copy_from_slice(&data);
                mapped.flush()?;
                // `mapped` and `file` drop here — no handles left open on `temp_path`.
            }

            // Atomic swap into place. `temp_path` has no open handles now (mapping +
            // File dropped above), so rename succeeds on Windows too. The destination
            // `path` is re-mapped fresh below, after the rename takes effect.
            std::fs::rename(&temp_path, path)?;

            // Re-open the final file and map it as the new zero-copy backend. Deserialize
            // from the in-memory `data` (source of truth) rather than the mapping so the
            // writeable destination handle is opened only after the rename completed.
            let file = OpenOptions::new().read(true).write(true).open(path)?;
            // SAFETY: `path` now holds the full serialized index (`data.len()` bytes),
            // so the mapping size exactly covers the file; `map_mut` validates internally.
            let mapped = unsafe { MmapMut::map_mut(&file)? };

            let mut new_index = CPIndex::deserialize_from_bytes(&data, false)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
            new_index.backend = IndexBackend::MMapFile {
                path: path.to_path_buf(),
                mmap: Some(mapped),
            };
            Ok(Box::new(new_index))
        })();

        match result {
            Ok(new) => Ok(new),
            Err(e) => Err(Error::Io(e)),
        }
    }

    fn search_nearest(
        &self,
        query_vec: &[f32],
        q_1bit: Option<&[u64]>,
        q_3bit: Option<(&[u8], f32)>,
        query_mask: &FilterBitset,
        top_k: usize,
        vector_store: Option<&dyn VectorStoreRef>,
    ) -> Vec<(u128, f32)> {
        // Same-name inherent method wins — deliberate delegation (keeps the
        // `#[tracing::instrument]` span and AUDREP-55 guard in one place).
        CPIndex::search_nearest(
            self,
            query_vec,
            q_1bit,
            q_3bit,
            query_mask,
            top_k,
            vector_store,
        )
    }

    fn search_with_method(
        &self,
        method: IndexType,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
        metric: DistanceMetric,
    ) -> Result<Vec<(u128, f32)>, Error> {
        self.search_with_method(method, query_vec, query_mask, top_k, metric)
    }

    fn search(
        &self,
        query_vec: &[f32],
        query_mask: &FilterBitset,
        top_k: usize,
        vector_store: Option<&dyn VectorStoreRef>,
        distance_metric: DistanceMetric,
    ) -> Vec<(u128, f32)> {
        // Same-name `VecIndex` method via UFCS (no ambiguity: `search` exists
        // only on `VecIndex` for this type, so this never recurses).
        <Self as VecIndex>::search(
            self,
            query_vec,
            query_mask,
            top_k,
            vector_store,
            distance_metric,
        )
    }

    fn repair_orphan_links(&self) -> FreshHnswReport {
        // Inherent method wins — deliberate delegation.
        self.repair_orphan_links()
    }

    fn storage_offset_of(&self, id: u128) -> Option<u64> {
        self.nodes.get(&id).map(|n| n.storage_offset)
    }

    fn stored_vector(&self, id: u128) -> Option<VectorRepresentations> {
        // Rare legacy-rescue path only — the clone never runs on hot paths.
        self.nodes.get(&id).map(|n| n.vec_data.clone())
    }

    fn node_view(&self, id: u128) -> Option<(u64, VectorRepresentations)> {
        self.nodes.get(&id).map(|n| {
            let v = n.value();
            (v.storage_offset, v.vec_data.clone())
        })
    }

    fn is_sq8_vector(&self, id: u128) -> Option<bool> {
        // Mirrors the `collect_actions` predicate shape exactly (`None` = absent).
        self.nodes
            .get(&id)
            .map(|n| matches!(n.vec_data, VectorRepresentations::SQ8(..)))
    }

    fn contains_node(&self, id: u128) -> bool {
        self.nodes.contains_key(&id)
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn total_nodes(&self) -> u64 {
        self.total_nodes.load(Ordering::Relaxed)
    }

    fn decrement_total_nodes(&self, by: u64) {
        // Vacuum path only — mirrors the legacy pairing (only vacuum decrements).
        let _ = self.total_nodes.fetch_sub(by, Ordering::Relaxed);
    }

    fn entry_point(&self) -> Option<u128> {
        self.get_entry_point()
    }

    fn set_entry_point(&self, id: u128) {
        // Inherent method wins (Relaxed ordering inside) — deliberate delegation.
        self.set_entry_point(id)
    }

    fn find_new_entry_point(&self) -> Option<u128> {
        // Inherent method wins — deliberate delegation.
        self.find_new_entry_point()
    }

    fn max_layer(&self) -> usize {
        self.max_layer.load(Ordering::Relaxed)
    }

    fn all_node_ids(&self) -> Vec<u128> {
        self.nodes.iter().map(|e| *e.key()).collect()
    }

    fn layer_neighbors(&self, id: u128, layer: usize) -> Vec<u128> {
        self.neighbor_index
            .get_neighbors_ref(id, layer)
            .map(|r| r.iter().copied().collect())
            .unwrap_or_default()
    }

    fn inline_neighbors(&self, id: u128, layer: usize) -> Vec<u128> {
        self.nodes
            .get(&id)
            .map(|n| {
                n.neighbor_lists
                    .get(layer)
                    .map(|l| l.iter().copied().collect())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    fn scan_entries(&self) -> Vec<(u128, u64, u32)> {
        // (id, storage_offset, flags) snapshot for maintenance/vacuum/stats walks.
        // Callers hold `insert_lock`, so no live-mutation skew vs direct iteration.
        self.nodes
            .iter()
            .map(|e| {
                let n = e.value();
                (*e.key(), n.storage_offset, n.flags)
            })
            .collect()
    }

    fn estimate_memory_bytes(&self) -> usize {
        // Inherent method wins — deliberate delegation.
        self.estimate_memory_bytes()
    }

    fn validate_index(&self) -> Result<(), Vec<String>> {
        // Inherent method wins — deliberate delegation.
        self.validate_index()
    }

    fn node_layers(&self, id: u128) -> usize {
        self.neighbor_index.num_layers(id).unwrap_or(0)
    }

    fn random_level(&self) -> usize {
        crate::index::random_layer_from_config(&self.config, &mut rand::rng())
    }

    fn bulk_levels_seeded(&self, seed: u64, n: usize) -> Vec<usize> {
        // Single RNG stream shared by all n draws — replicates the pre-split
        // `random_layer_from_config(&config, &mut rng)` loop exactly.
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        (0..n)
            .map(|_| crate::index::random_layer_from_config(&self.config, &mut rng))
            .collect()
    }

    fn flat_threshold(&self) -> Option<usize> {
        self.config.flat_threshold
    }

    fn set_flat_threshold(&mut self, threshold: Option<usize>) {
        self.config.flat_threshold = threshold;
    }

    fn distance_metric(&self) -> DistanceMetric {
        self.config.distance_metric
    }

    fn index_kind(&self) -> IndexType {
        self.config.index_type
    }

    fn set_storage_offset(&self, id: u128, offset: u64) -> bool {
        if let Some(mut r) = self.nodes.get_mut(&id) {
            r.storage_offset = offset;
            true
        } else {
            false
        }
    }

    fn remove_node(&self, id: u128) -> bool {
        // Does NOT touch `total_nodes` — legacy pairing (only vacuum decrements).
        self.nodes.remove(&id).is_some()
    }

    fn add_node(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
    ) -> Result<(), Error> {
        self.add(id, bitset, vec_data, storage_offset)
    }

    fn add_node_with_level(
        &self,
        id: u128,
        bitset: FilterBitset,
        vec_data: VectorRepresentations,
        storage_offset: u64,
        level: usize,
    ) -> Result<(), Error> {
        self.add_with_level(id, bitset, vec_data, storage_offset, level)
    }
}
