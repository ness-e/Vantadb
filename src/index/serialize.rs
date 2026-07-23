#[cfg(not(feature = "memmap2"))]
use crate::storage::vfile::MmapMut;
#[cfg(feature = "memmap2")]
use memmap2::MmapMut;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::Ordering;
use tracing::{info, warn};

use rand::SeedableRng;

use crate::index::graph::{
    self, CPIndex, HnswNode, IndexBackend, NeighborVec, VECTOR_INDEX_VERSION,
};
use crate::node::{DistanceMetric, FilterBitset, VectorRepresentations};

impl CPIndex {
    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.nodes.len() * 256 + 128);
        self.serialize_to_writer(&mut buf)
            .expect("Vec::write cannot fail");
        buf
    }

    pub fn serialize_to_writer(&self, w: &mut impl Write) -> std::io::Result<()> {
        let header = crate::binary_header::VantaHeader::new(*b"VNDX", VECTOR_INDEX_VERSION, 0);
        let mut pos = 0usize;

        let hdr = header.serialize();
        w.write_all(&hdr)?;
        pos += hdr.len();

        let max_layer_bytes = (self.max_layer.load(Ordering::Acquire) as u64).to_le_bytes();
        w.write_all(&max_layer_bytes)?;
        pos += max_layer_bytes.len();

        for val in [
            (self.config.m as u64).to_le_bytes(),
            (self.config.m_max0 as u64).to_le_bytes(),
            (self.config.ef_construction as u64).to_le_bytes(),
            (self.config.ef_search as u64).to_le_bytes(),
            self.config.ml.to_le_bytes(),
        ] {
            w.write_all(&val)?;
            pos += val.len();
        }

        let metric_byte: u8 = match self.config.distance_metric {
            DistanceMetric::Cosine => 0,
            DistanceMetric::Euclidean => 1,
        };
        w.write_all(&[metric_byte])?;
        pos += 1;

        match self.config.flat_threshold {
            Some(t) => {
                w.write_all(&[1])?;
                w.write_all(&(t as u64).to_le_bytes())?;
                pos += 9;
            }
            None => {
                w.write_all(&[0])?;
                pos += 1;
            }
        }

        match self.get_entry_point() {
            Some(ep) => {
                w.write_all(&[1])?;
                w.write_all(&ep.to_le_bytes())?;
                pos += 17;
            }
            None => {
                w.write_all(&[0])?;
                w.write_all(&0u128.to_le_bytes())?;
                pos += 17;
            }
        }

        let node_count = self.nodes.len() as u64;
        let nc = node_count.to_le_bytes();
        w.write_all(&nc)?;
        pos += nc.len();

        for node_id in self.serialization_order() {
            let Some(node) = self.nodes.get(&node_id) else {
                continue;
            };
            let id_bytes = node.id.to_le_bytes();
            w.write_all(&id_bytes)?;
            pos += id_bytes.len();

            let bs = node.bitset.to_bytes();
            w.write_all(&bs)?;
            pos += bs.len();

            let so = node.storage_offset.to_le_bytes();
            w.write_all(&so)?;
            pos += so.len();

            match &node.vec_data {
                VectorRepresentations::Full(f) => {
                    w.write_all(&[1])?;
                    w.write_all(&(f.len() as u64).to_le_bytes())?;
                    pos += 9;
                    let padding = (4 - (pos % 4)) % 4;
                    if padding > 0 {
                        w.write_all(&[0u8; 4][..padding])?;
                        pos += padding;
                    }
                    for &val in f {
                        let b = val.to_le_bytes();
                        w.write_all(&b)?;
                        pos += b.len();
                    }
                }
                VectorRepresentations::MmapFull(mmap_opt) => {
                    let slice = if let Some(mmap) = mmap_opt {
                        let len = mmap.len() / 4;
                        if len == 0 || len > graph::MAX_VEC_F32_LEN {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("MmapFull invalid len in serialize: len={len}"),
                            ));
                        }
                        // SAFETY: len bounded by MAX_VEC_F32_LEN above.
                        unsafe { std::slice::from_raw_parts(mmap.as_ptr() as *const f32, len) }
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "MmapFull variant is None — cannot serialize",
                        ));
                    };
                    w.write_all(&[1])?;
                    w.write_all(&(slice.len() as u64).to_le_bytes())?;
                    pos += 9;
                    let padding = (4 - (pos % 4)) % 4;
                    if padding > 0 {
                        w.write_all(&[0u8; 4][..padding])?;
                        pos += padding;
                    }
                    for &val in slice {
                        let b = val.to_le_bytes();
                        w.write_all(&b)?;
                        pos += b.len();
                    }
                }
                VectorRepresentations::Binary(b) => {
                    w.write_all(&[2])?;
                    w.write_all(&(b.len() as u64).to_le_bytes())?;
                    pos += 9;
                    for &val in b {
                        let b2 = val.to_le_bytes();
                        w.write_all(&b2)?;
                        pos += b2.len();
                    }
                }
                VectorRepresentations::Turbo(t) => {
                    w.write_all(&[3])?;
                    w.write_all(&(t.len() as u64).to_le_bytes())?;
                    pos += 9;
                    w.write_all(t)?;
                    pos += t.len();
                }
                VectorRepresentations::SQ8(d, scale) => {
                    w.write_all(&[4])?;
                    w.write_all(&(d.len() as u64).to_le_bytes())?;
                    pos += 9;
                    for &v in d {
                        w.write_all(&[v as u8])?;
                        pos += 1;
                    }
                    let sb = scale.to_le_bytes();
                    w.write_all(&sb)?;
                    pos += sb.len();
                }
                VectorRepresentations::None => {
                    w.write_all(&[0])?;
                    w.write_all(&0u64.to_le_bytes())?;
                    pos += 9;
                }
            }

            let layer_count = node.neighbors.len() as u64;
            let lc = layer_count.to_le_bytes();
            w.write_all(&lc)?;
            pos += lc.len();
            for layer in &node.neighbors {
                let neighbor_count = layer.len() as u64;
                let nc = neighbor_count.to_le_bytes();
                w.write_all(&nc)?;
                pos += nc.len();
                for &nid in layer {
                    let nidb = nid.to_le_bytes();
                    w.write_all(&nidb)?;
                    pos += nidb.len();
                }
            }
        }

        Ok(())
    }

    pub fn deserialize_from_bytes(data: &[u8], _force_copy: bool) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind};

        use crate::index::graph::{HnswConfig, ENTRY_POINT_NONE};
        use dashmap::DashMap;
        use portable_atomic::AtomicU128;

        use std::sync::atomic::{AtomicU64, AtomicUsize};

        #[inline]
        fn take_bytes<'a>(
            data: &'a [u8],
            pos: &mut usize,
            n: usize,
            field: &str,
        ) -> std::io::Result<&'a [u8]> {
            if *pos > data.len() || n > data.len() - *pos {
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
        fn read_le_u128(data: &[u8], pos: &mut usize, field: &str) -> std::io::Result<u128> {
            let bytes = take_bytes(data, pos, 16, field)?;
            Ok(u128::from_le_bytes(bytes.try_into().map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("failed to parse {field} as u128: {e}"),
                )
            })?))
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
        if version >= 3 && pos < data.len() {
            config.distance_metric = match take_bytes(data, &mut pos, 1, "distance_metric")?[0] {
                1 => DistanceMetric::Euclidean,
                _ => DistanceMetric::Cosine,
            };
        }
        if version >= 7 && pos < data.len() {
            let ft_exists = take_bytes(data, &mut pos, 1, "flat_threshold_exists")?[0];
            if ft_exists == 1 {
                config.flat_threshold =
                    Some(read_le_u64(data, &mut pos, "flat_threshold")? as usize);
            } else {
                config.flat_threshold = None;
            }
        }

        let ep_exists = take_bytes(data, &mut pos, 1, "ep_exists")?[0];
        let ep_id = read_le_u128(data, &mut pos, "ep_id")?;
        let entry_point = if ep_exists == 1 { Some(ep_id) } else { None };

        let node_count = read_le_u64(data, &mut pos, "node_count")? as usize;

        const MIN_BYTES_PER_NODE: usize = 16 + 4 + 8 + 1 + 8 + 8;
        let remaining = data.len().saturating_sub(pos);
        if node_count > remaining / MIN_BYTES_PER_NODE {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "node_count ({node_count}) exceeds plausible limit for {remaining} remaining bytes",
                ),
            ));
        }

        let nodes: DashMap<u128, HnswNode> = DashMap::with_capacity(node_count);

        for _ in 0..node_count {
            let id = read_le_u128(data, &mut pos, "node id")?;

            let (bitset, consumed) = FilterBitset::from_bytes(&data[pos..])?;
            pos += consumed;

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
                    let mut v = Vec::with_capacity(vec_len);
                    for i in 0..vec_len {
                        let start = i * 4;
                        v.push(f32::from_le_bytes(
                            vec_bytes[start..start + 4].try_into().map_err(|e| {
                                std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    format!("f32 vec chunk at byte {start} expected 4 bytes: {e}"),
                                )
                            })?,
                        ));
                    }
                    VectorRepresentations::Full(v)
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
                    .checked_mul(16)
                    .ok_or_else(|| Error::new(ErrorKind::InvalidData, "neighbor_count overflow"))?;
                let nbr_bytes = take_bytes(data, &mut pos, byte_len, "neighbor ids")?;
                let mut layer_neighbors = NeighborVec::with_capacity(neighbor_count);
                for i in 0..neighbor_count {
                    let start = i * 16;
                    layer_neighbors.push(u128::from_le_bytes(
                        nbr_bytes[start..start + 16].try_into().map_err(|e| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("neighbor id at byte {start} expected 16 bytes: {e}"),
                            )
                        })?,
                    ));
                }
                neighbors.push(layer_neighbors);
            }

            let (inv_cached_norm, norm_sq) =
                graph::cached_norms_for_metric(config.distance_metric, &vec_data);
            nodes.insert(
                id,
                HnswNode {
                    id,
                    bitset,
                    vec_data,
                    neighbors,
                    storage_offset,
                    inv_cached_norm,
                    norm_sq,
                    flags: 0,
                },
            );
        }

        let node_count = nodes.len() as u64;
        Ok(Self {
            nodes,
            max_layer: AtomicUsize::new(max_layer),
            entry_point: AtomicU128::new(entry_point.unwrap_or(ENTRY_POINT_NONE)),
            backend: IndexBackend::InMemory,
            config,
            total_nodes: AtomicU64::new(node_count),
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
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.serialize_to_writer(&mut writer)?;
        writer.flush()?;
        info!(path = %path.display(), node_count = self.nodes.len(), "HNSW index persisted (streaming)");
        Ok(())
    }

    fn warn_validation_violations(index: &CPIndex) {
        if let Err(violations) = index.validate_index() {
            warn!(
                violation_count = violations.len(),
                "HNSW index has integrity violations after deserialization"
            );
            for v in &violations[..violations.len().min(5)] {
                warn!(violation = %v, "HNSW integrity violation");
            }
        }
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

            let file_len = file.metadata().ok().map(|m| m.len()).unwrap_or(0);
            if file_len < 64 {
                warn!("HNSW index file too small ({file_len} bytes) — will rebuild");
                return None;
            }

            // SAFETY: file size verified above — `map_mut` on a file shorter
            // than the mapping causes SIGBUS on access. We checked file_len >= 64
            // which covers the header, so the mapping cannot fault on header reads.
            let mmap = match unsafe { MmapMut::map_mut(&file) } {
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
                    Self::warn_validation_violations(&index);
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
                    Self::warn_validation_violations(&index);
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

        // SAFETY: `file` is a newly created/truncated handle at `data.len()` bytes;
        // `map_mut` validates the pointer internally.
        let mut mapped = unsafe { MmapMut::map_mut(&file)? };
        mapped.copy_from_slice(&data);
        mapped.flush()?;

        let new_index = Self::deserialize_from_bytes(&mapped, false)?;
        self.nodes = new_index.nodes;
        self.entry_point = new_index.entry_point;

        // Drop mmap and file handle before rename (Windows requires the temp file
        // to have no open handles for rename to succeed). Re-create after.
        drop(mapped);
        drop(file);
        std::fs::rename(&temp_path, &path)?;

        let file = OpenOptions::new().read(true).write(true).open(&path)?;
        let new_mmap = unsafe { MmapMut::map_mut(&file)? };
        if let IndexBackend::MMapFile { ref mut mmap, .. } = self.backend {
            *mmap = Some(new_mmap);
        }

        info!(path = %path.display(), node_count = self.nodes.len(), bytes = data.len(), "HNSW MMap synced & zero-copy pointers re-mapped via atomic rename");
        Ok(())
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::index::graph::{HnswConfig, ENTRY_POINT_NONE};
    use crate::index::IndexBackend;
    use crate::node::{DistanceMetric, FilterBitset, VectorRepresentations};
    use portable_atomic::AtomicU128;
    use std::sync::atomic::{AtomicU64, AtomicUsize};

    /// Helper: build a small CPIndex with a single Full vector node.
    fn single_full_node_index() -> CPIndex {
        let nodes = dashmap::DashMap::new();
        let id = 42u128;
        nodes.insert(
            id,
            HnswNode {
                id,
                bitset: FilterBitset::new(),
                vec_data: VectorRepresentations::Full(vec![0.1, 0.2, 0.3, 0.4]),
                neighbors: vec![smallvec::smallvec![99u128]],
                storage_offset: 0,
                inv_cached_norm: 1.0,
                norm_sq: 1.0,
                flags: 0,
            },
        );
        // Also insert the neighbor so validation passes
        nodes.insert(
            99u128,
            HnswNode {
                id: 99,
                bitset: FilterBitset::new(),
                vec_data: VectorRepresentations::Full(vec![0.5, 0.6, 0.7, 0.8]),
                neighbors: vec![smallvec::smallvec![42u128]],
                storage_offset: 0,
                inv_cached_norm: 1.0,
                norm_sq: 1.0,
                flags: 0,
            },
        );
        CPIndex {
            nodes,
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU128::new(42),
            backend: IndexBackend::InMemory,
            config: HnswConfig::default(),
            total_nodes: AtomicU64::new(2),
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
        }
    }

    // ── Round-trip: serialize → deserialize ──

    #[test]
    fn roundtrip_empty_index() {
        let index = CPIndex::new();
        let bytes = index.serialize_to_bytes();
        assert!(bytes.len() >= 16, "header present");
        let deser = CPIndex::deserialize_from_bytes(&bytes, true).unwrap();
        assert_eq!(deser.nodes.len(), 0);
        assert_eq!(deser.config.m, index.config.m);
        assert_eq!(deser.config.distance_metric, index.config.distance_metric);
        assert_eq!(
            deser.max_layer.load(std::sync::atomic::Ordering::Relaxed),
            0
        );
    }

    #[test]
    fn roundtrip_single_full_node() {
        let index = single_full_node_index();
        let bytes = index.serialize_to_bytes();
        let deser = CPIndex::deserialize_from_bytes(&bytes, true).unwrap();
        assert_eq!(deser.nodes.len(), 2);
        let node = deser.nodes.get(&42).unwrap();
        assert_eq!(node.id, 42);
        match &node.vec_data {
            VectorRepresentations::Full(v) => assert_eq!(v.as_slice(), &[0.1, 0.2, 0.3, 0.4]),
            _ => panic!("expected Full"),
        }
        assert_eq!(node.neighbors.len(), 1);
        assert_eq!(node.neighbors[0].as_slice(), &[99u128]);
    }

    #[test]
    fn roundtrip_binary_vector() {
        let nodes = dashmap::DashMap::new();
        nodes.insert(
            1u128,
            HnswNode {
                id: 1,
                bitset: FilterBitset::new(),
                vec_data: VectorRepresentations::Binary(vec![0xDEADBEEFCAFEu64].into_boxed_slice()),
                neighbors: vec![smallvec::smallvec![]],
                storage_offset: 0,
                inv_cached_norm: 0.0,
                norm_sq: 0.0,
                flags: 0,
            },
        );
        let index = CPIndex {
            nodes,
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU128::new(ENTRY_POINT_NONE),
            backend: IndexBackend::InMemory,
            config: HnswConfig::default(),
            total_nodes: AtomicU64::new(1),
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
        };
        let bytes = index.serialize_to_bytes();
        let deser = CPIndex::deserialize_from_bytes(&bytes, true).unwrap();
        let node = deser.nodes.get(&1).unwrap();
        match &node.vec_data {
            VectorRepresentations::Binary(b) => assert_eq!(b.as_ref(), &[0xDEADBEEFCAFEu64]),
            _ => panic!("expected Binary"),
        }
    }

    #[test]
    fn roundtrip_turbo_vector() {
        let nodes = dashmap::DashMap::new();
        nodes.insert(
            1u128,
            HnswNode {
                id: 1,
                bitset: FilterBitset::new(),
                vec_data: VectorRepresentations::Turbo(vec![0xAB, 0xCD].into_boxed_slice()),
                neighbors: vec![smallvec::smallvec![]],
                storage_offset: 0,
                inv_cached_norm: 0.0,
                norm_sq: 0.0,
                flags: 0,
            },
        );
        let index = CPIndex {
            nodes,
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU128::new(ENTRY_POINT_NONE),
            backend: IndexBackend::InMemory,
            config: HnswConfig::default(),
            total_nodes: AtomicU64::new(1),
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
        };
        let bytes = index.serialize_to_bytes();
        let deser = CPIndex::deserialize_from_bytes(&bytes, true).unwrap();
        let node = deser.nodes.get(&1).unwrap();
        match &node.vec_data {
            VectorRepresentations::Turbo(t) => assert_eq!(t.as_ref(), &[0xAB, 0xCD]),
            _ => panic!("expected Turbo"),
        }
    }

    #[test]
    fn roundtrip_sq8_vector() {
        let nodes = dashmap::DashMap::new();
        nodes.insert(
            1u128,
            HnswNode {
                id: 1,
                bitset: FilterBitset::new(),
                vec_data: VectorRepresentations::SQ8(
                    vec![10i8, -20, 30, -40].into_boxed_slice(),
                    2.5,
                ),
                neighbors: vec![smallvec::smallvec![]],
                storage_offset: 0,
                inv_cached_norm: 0.0,
                norm_sq: 0.0,
                flags: 0,
            },
        );
        let index = CPIndex {
            nodes,
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU128::new(ENTRY_POINT_NONE),
            backend: IndexBackend::InMemory,
            config: HnswConfig::default(),
            total_nodes: AtomicU64::new(1),
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
        };
        let bytes = index.serialize_to_bytes();
        let deser = CPIndex::deserialize_from_bytes(&bytes, true).unwrap();
        let node = deser.nodes.get(&1).unwrap();
        match &node.vec_data {
            VectorRepresentations::SQ8(d, scale) => {
                assert_eq!(d.as_ref(), &[10i8, -20, 30, -40]);
                assert!((scale - 2.5).abs() < f32::EPSILON);
            }
            _ => panic!("expected SQ8"),
        }
    }

    #[test]
    fn roundtrip_none_vector() {
        let nodes = dashmap::DashMap::new();
        nodes.insert(
            1u128,
            HnswNode {
                id: 1,
                bitset: FilterBitset::new(),
                vec_data: VectorRepresentations::None,
                neighbors: vec![smallvec::smallvec![]],
                storage_offset: 0,
                inv_cached_norm: 0.0,
                norm_sq: 0.0,
                flags: 0,
            },
        );
        let index = CPIndex {
            nodes,
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU128::new(ENTRY_POINT_NONE),
            backend: IndexBackend::InMemory,
            config: HnswConfig::default(),
            total_nodes: AtomicU64::new(1),
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
        };
        let bytes = index.serialize_to_bytes();
        let deser = CPIndex::deserialize_from_bytes(&bytes, true).unwrap();
        let node = deser.nodes.get(&1).unwrap();
        assert!(node.vec_data.is_none());
    }

    #[test]
    fn roundtrip_multiple_nodes_with_neighbors() {
        let mut index = CPIndex::new();
        // Manually add 3 small fully-connected nodes
        let ids = [10u128, 20, 30];
        for &id in &ids {
            index.nodes.insert(
                id,
                HnswNode {
                    id,
                    bitset: FilterBitset::all_set(),
                    vec_data: VectorRepresentations::Full(vec![id as f32 / 100.0; 4]),
                    neighbors: vec![smallvec::smallvec![
                        ids[(ids.iter().position(|x| *x == id).unwrap() + 1) % 3],
                        ids[(ids.iter().position(|x| *x == id).unwrap() + 2) % 3],
                    ]],
                    storage_offset: id as u64,
                    inv_cached_norm: 1.0,
                    norm_sq: 0.5,
                    flags: 0,
                },
            );
        }
        index.max_layer = AtomicUsize::new(0);
        index.entry_point = AtomicU128::new(10);
        index.total_nodes = AtomicU64::new(3);
        index.config.ef_search = 200;

        let bytes = index.serialize_to_bytes();
        let deser = CPIndex::deserialize_from_bytes(&bytes, true).unwrap();
        assert_eq!(deser.nodes.len(), 3);
        assert_eq!(deser.config.ef_search, 200);

        for &id in &ids {
            let node = deser.nodes.get(&id).unwrap();
            assert_eq!(node.storage_offset, id as u64);
            assert_eq!(node.neighbors.len(), 1);
            assert_eq!(node.neighbors[0].len(), 2);
        }
    }

    #[test]
    fn roundtrip_with_bitset_filter() {
        let mut bs = FilterBitset::new();
        bs.set_bit(0);
        bs.set_bit(2);
        bs.set_bit(127);
        let nodes = dashmap::DashMap::new();
        nodes.insert(
            1u128,
            HnswNode {
                id: 1,
                bitset: bs.clone(),
                vec_data: VectorRepresentations::Full(vec![1.0; 8]),
                neighbors: vec![smallvec::smallvec![]],
                storage_offset: 100,
                inv_cached_norm: 1.0,
                norm_sq: 1.0,
                flags: 0,
            },
        );
        let index = CPIndex {
            nodes,
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU128::new(ENTRY_POINT_NONE),
            backend: IndexBackend::InMemory,
            config: HnswConfig::default(),
            total_nodes: AtomicU64::new(1),
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
        };
        let data = index.serialize_to_bytes();
        let deser = CPIndex::deserialize_from_bytes(&data, true).unwrap();
        let node = deser.nodes.get(&1).unwrap();
        assert!(node.bitset.has_bit(0));
        assert!(!node.bitset.has_bit(1));
        assert!(node.bitset.has_bit(2));
        assert!(node.bitset.has_bit(127));
    }

    // ── Error handling ──

    fn unwrap_io_err<T>(result: std::io::Result<T>) -> std::io::Error {
        match result {
            Err(e) => e,
            Ok(_) => panic!("expected Err"),
        }
    }

    #[test]
    fn deserialize_truncated_header() {
        let err = unwrap_io_err(CPIndex::deserialize_from_bytes(&[0u8; 10], true));
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn deserialize_wrong_magic() {
        let index = CPIndex::new();
        let mut bytes = index.serialize_to_bytes();
        bytes[0..4].copy_from_slice(b"BAD!");
        let err = unwrap_io_err(CPIndex::deserialize_from_bytes(&bytes, true));
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        let msg = err.to_string();
        assert!(
            msg.contains("Index format") || msg.contains("magic"),
            "wrong magic: {msg}"
        );
    }

    #[test]
    fn deserialize_wrong_version() {
        let index = CPIndex::new();
        let mut bytes = index.serialize_to_bytes();
        bytes[4] = 0xFF;
        bytes[5] = 0xFF;
        let err = unwrap_io_err(CPIndex::deserialize_from_bytes(&bytes, true));
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn deserialize_node_count_exceeds_remaining() {
        let index = CPIndex::new();
        let mut bytes = index.serialize_to_bytes();
        // Patch the last 8 bytes (node_count) to an absurdly high value
        let sz = bytes.len();
        let nc_offset = sz - 8;
        bytes[nc_offset..].copy_from_slice(&u64::MAX.to_le_bytes());
        let err = unwrap_io_err(CPIndex::deserialize_from_bytes(&bytes, true));
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn deserialize_truncated_in_middle() {
        let index = single_full_node_index();
        let bytes = index.serialize_to_bytes();
        // Truncate past the "too small" guard (header + max_layer = 24) but before
        // the node data — triggers UnexpectedEof from take_bytes.
        let truncated = &bytes[..bytes.len() - 10];
        let err = unwrap_io_err(CPIndex::deserialize_from_bytes(truncated, true));
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn serialize_mmapfull_none_fails() {
        let nodes = dashmap::DashMap::new();
        nodes.insert(
            1u128,
            HnswNode {
                id: 1,
                bitset: FilterBitset::new(),
                vec_data: VectorRepresentations::MmapFull(None),
                neighbors: vec![smallvec::smallvec![]],
                storage_offset: 0,
                inv_cached_norm: 0.0,
                norm_sq: 0.0,
                flags: 0,
            },
        );
        let index = CPIndex {
            nodes,
            max_layer: AtomicUsize::new(0),
            entry_point: AtomicU128::new(ENTRY_POINT_NONE),
            backend: IndexBackend::InMemory,
            config: HnswConfig::default(),
            total_nodes: AtomicU64::new(1),
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(42)),
        };
        let err = index
            .serialize_to_writer(&mut std::io::Cursor::new(Vec::new()))
            .unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    // ── Persist / load round-trip ──

    #[test]
    fn persist_and_load_roundtrip() {
        let index = single_full_node_index();
        let dir = std::env::temp_dir().join(format!("vantadb_ser_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("index.bin");

        index.persist_to_file(&path).unwrap();
        assert!(path.exists(), "file must exist");

        let loaded = match CPIndex::load_from_file(&path, false) {
            Some(index) => index,
            None => panic!("load should succeed"),
        };
        assert_eq!(loaded.nodes.len(), 2);
        let node = loaded.nodes.get(&42).unwrap();
        match &node.vec_data {
            VectorRepresentations::Full(v) => assert_eq!(v.as_slice(), &[0.1, 0.2, 0.3, 0.4]),
            _ => panic!("expected Full"),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_nonexistent_file_returns_none() {
        let path = std::env::temp_dir().join("vantadb__nonexistent__index.bin");
        let result = CPIndex::load_from_file(&path, false);
        assert!(result.is_none());
    }

    #[test]
    fn euclidean_metric_roundtrip() {
        let mut index = CPIndex::new_with_config(HnswConfig {
            distance_metric: DistanceMetric::Euclidean,
            ..Default::default()
        });
        index.nodes.insert(
            1u128,
            HnswNode {
                id: 1,
                bitset: FilterBitset::new(),
                vec_data: VectorRepresentations::Full(vec![0.5; 4]),
                neighbors: vec![smallvec::smallvec![]],
                storage_offset: 0,
                inv_cached_norm: 1.0,
                norm_sq: 0.25,
                flags: 0,
            },
        );
        index.entry_point = AtomicU128::new(ENTRY_POINT_NONE);
        index.total_nodes = AtomicU64::new(1);

        let data = index.serialize_to_bytes();
        let deser = CPIndex::deserialize_from_bytes(&data, true).unwrap();
        assert_eq!(deser.config.distance_metric, DistanceMetric::Euclidean);
    }

    #[test]
    fn flat_threshold_roundtrip() {
        let mut config = HnswConfig::default();
        config.flat_threshold = Some(5000);
        let index = CPIndex::new_with_config(config);
        let data = index.serialize_to_bytes();
        let deser = CPIndex::deserialize_from_bytes(&data, true).unwrap();
        assert_eq!(deser.config.flat_threshold, Some(5000));

        // None roundtrip
        let mut config2 = HnswConfig::default();
        config2.flat_threshold = None;
        let index2 = CPIndex::new_with_config(config2);
        let data2 = index2.serialize_to_bytes();
        let deser2 = CPIndex::deserialize_from_bytes(&data2, true).unwrap();
        assert_eq!(deser2.config.flat_threshold, None);
    }

    #[test]
    fn load_corrupt_file_returns_none() {
        let dir =
            std::env::temp_dir().join(format!("vantadb_ser_test_corrupt_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("corrupt.bin");
        std::fs::write(&path, &[0u8; 32]).unwrap();
        let result = CPIndex::load_from_file(&path, false);
        assert!(result.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn to_bytes_matches_writer() {
        let index = single_full_node_index();
        let bytes = index.serialize_to_bytes();
        let mut buf = Vec::new();
        index.serialize_to_writer(&mut buf).unwrap();
        assert_eq!(bytes, buf);
    }

    #[test]
    fn config_preserved_after_roundtrip() {
        let mut config = HnswConfig::default();
        config.m = 16;
        config.m_max0 = 32;
        config.ef_construction = 200;
        config.ef_search = 50;
        config.ml = 0.5;
        let index = CPIndex::new_with_config(config.clone());
        let data = index.serialize_to_bytes();
        let deser = CPIndex::deserialize_from_bytes(&data, true).unwrap();
        assert_eq!(deser.config.m, config.m);
        assert_eq!(deser.config.m_max0, config.m_max0);
        assert_eq!(deser.config.ef_construction, config.ef_construction);
        assert_eq!(deser.config.ef_search, config.ef_search);
        assert!((deser.config.ml - config.ml).abs() < f64::EPSILON);
    }
}
