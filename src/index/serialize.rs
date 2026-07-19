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
use crate::node::{DistanceMetric, FilterBitset, SendPtr, VectorRepresentations};

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
                VectorRepresentations::MmapFull(ptr, len) => {
                    w.write_all(&[1])?;
                    w.write_all(&(*len as u64).to_le_bytes())?;
                    pos += 9;
                    let padding = (4 - (pos % 4)) % 4;
                    if padding > 0 {
                        w.write_all(&[0u8; 4][..padding])?;
                        pos += padding;
                    }
                    if ptr.0.is_null() || *len == 0 || *len > graph::MAX_VEC_F32_LEN {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "MmapFull invalid ptr/len in serialize: ptr={:p} len={}",
                                ptr.0, *len
                            ),
                        ));
                    }
                    // SAFETY: null/zero/overflow guard above ensures `ptr.0` is non-null,
                    // `*len` is bounded by `MAX_VEC_F32_LEN`, and the resulting slice
                    // length is valid for the serialized representation.
                    let slice = unsafe { std::slice::from_raw_parts(ptr.0, *len) };
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

    pub fn deserialize_from_bytes(data: &[u8], force_copy: bool) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind};

        use crate::index::graph::{HnswConfig, ENTRY_POINT_NONE};
        use dashmap::DashMap;
        use portable_atomic::AtomicU128;
        use std::hash::BuildHasherDefault;
        use std::sync::atomic::{AtomicU64, AtomicUsize};

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

        let nodes = DashMap::with_capacity_and_hasher(node_count, BuildHasherDefault::default());

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
                        debug_assert_eq!(
                            vec_bytes.as_ptr().align_offset(4),
                            0,
                            "MmapFull: vec_bytes not 4-byte aligned"
                        );
                        if vec_len == 0 || vec_len > graph::MAX_VEC_F32_LEN {
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
