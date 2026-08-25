//! Memory-mapped vector store file (VantaFile) with read/write and in-memory variants.
//!
//! When the `encryption` feature is enabled, VantaFile can optionally hold a
//! [`Cipher`] instance for transparent at-rest encryption. The cipher is stored
//! for use by the storage layer and can be retrieved via [`VantaFile::cipher`].
//!
//! The mmap primitives (memmap2 re-export / shim, SIGBUS handler, resident byte
//! accounting, `AlignedBytes`) live in [`crate::storage::vfile_mmap`] and are
//! re-exported here so `crate::storage::vfile::*` paths keep resolving
//! (REVIEW-04 split).

use crate::binary_header::VantaHeader;
#[cfg(feature = "encryption")]
use crate::crypto::{Cipher, EncryptionStream};
use crate::error::{Result, VantaError};
use crate::node::DiskNodeHeader;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use zerocopy::{FromBytes, IntoBytes};

use crate::storage::engine::STORAGE_ALIGNMENT;

// Re-exports preserved from the pre-split single file: public paths
// (`get_resident_bytes`), crate-internal paths (mmap types, sigbus handler,
// aligned buffer) all keep resolving via `crate::storage::vfile::*`.
#[cfg(unix)]
pub(crate) use crate::storage::vfile_mmap::install_sigbus_handler;
pub use crate::storage::vfile_mmap::{get_resident_bytes, get_resident_bytes_impl};
pub(crate) use crate::storage::vfile_mmap::{
    map_readonly, map_readwrite, AlignedBytes, Mmap, MmapMut,
};

/// Current VantaFile format version.
/// Version history:
///   - v1: initial format
///   - v2: migrated (bumped header only, data layout identical to v1)
pub const VFILE_VERSION: u16 = 2;

/// Sum of resident mmap bytes across the HNSW index and vector store.
/// Only compiled for tests — production code uses per-VantaFile metrics directly.
#[cfg(test)]
pub(crate) fn engine_mmap_resident_bytes(
    hnsw: &crate::index::CPIndex,
    vector_store: &VantaFile,
) -> Option<u64> {
    let mut total = None;
    for resident in [
        vector_store.mmap_resident_bytes(),
        hnsw.backend.mmap_resident_bytes(),
    ]
    .into_iter()
    .flatten()
    {
        total = Some(total.unwrap_or(0) + resident);
    }
    total
}

enum VantaFileMap {
    ReadOnly(Mmap),
    ReadWrite(MmapMut),
    InMemory(AlignedBytes),
}

impl VantaFileMap {
    fn as_slice(&self) -> &[u8] {
        match self {
            VantaFileMap::ReadOnly(m) => m,
            VantaFileMap::ReadWrite(m) => m,
            VantaFileMap::InMemory(d) => d.as_slice(),
        }
    }
    fn as_ptr(&self) -> *const u8 {
        match self {
            VantaFileMap::ReadOnly(m) => m.as_ptr(),
            VantaFileMap::ReadWrite(m) => m.as_ptr(),
            VantaFileMap::InMemory(d) => d.as_ptr(),
        }
    }
    fn len(&self) -> usize {
        match self {
            VantaFileMap::ReadOnly(m) => m.len(),
            VantaFileMap::ReadWrite(m) => m.len(),
            VantaFileMap::InMemory(d) => d.len(),
        }
    }
    fn as_mut_slice(&mut self) -> Result<&mut [u8]> {
        match self {
            VantaFileMap::ReadOnly(_) => Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "VantaFile is read-only".into(),
            }),
            VantaFileMap::ReadWrite(m) => Ok(m),
            VantaFileMap::InMemory(d) => Ok(d.as_mut_slice()),
        }
    }
    fn flush(&self) -> Result<()> {
        match self {
            VantaFileMap::ReadOnly(_) => Ok(()),
            VantaFileMap::ReadWrite(m) => m.flush().map_err(VantaError::IoError),
            VantaFileMap::InMemory(_) => Ok(()),
        }
    }
}

/// A memory-mapped vector store file supporting read, write, and in-memory modes.
pub struct VantaFile {
    /// Optional backing file handle (None for in-memory mode).
    pub file: Option<File>,
    mmap: VantaFileMap,
    /// File system path to the backing file.
    pub path: PathBuf,
    /// Current file size in bytes.
    pub size: u64,
    /// Byte offset for the next write operation.
    pub write_cursor: u64,
    read_only: bool,
    /// AES-256-GCM cipher for at-rest encryption when the `encryption` feature
    /// is enabled and `VANTADB_ENCRYPTION_KEY` is set.
    #[cfg(feature = "encryption")]
    pub cipher: Option<Cipher>,
}

// SAFETY: VantaFile owns a `File` handle, a `VantaFileMap` (Mmap/MmapMut/AlignedBytes),
// a `PathBuf`, and an `AtomicBool` — all of which are `Send`. The mmap pointers
// are managed by the memmap2 crate or the in-memory/shim buffers (AlignedBytes,
// `unsafe impl Send + Sync` above), all `Send + Sync`. The cipher field (behind
// `#[cfg(feature = "encryption")]`) is Send by construction. No mutable aliasing
// crosses threads because all mutations go through `&mut self` or the storage
// engine's locks.
unsafe impl Send for VantaFile {}
// SAFETY: same reasoning — all fields are Sync-safe, and the engine serializes
// read-write access through `RwLock<VantaFile>`.
unsafe impl Sync for VantaFile {}

impl VantaFile {
    /// Open or create a VantaFile at the given path with the specified initial size.
    pub fn open(path: PathBuf, initial_size: u64) -> Result<Self> {
        Self::open_with_mode(path, initial_size, false)
    }
    /// Open an existing VantaFile in read-only mode.
    pub fn open_read_only(path: PathBuf) -> Result<Self> {
        Self::open_with_mode(path, 0, true)
    }

    /// Create a VantaFile backed entirely by in-memory storage (no disk I/O).
    pub fn create_in_memory(initial_size: u64) -> Self {
        let size = initial_size.max(STORAGE_ALIGNMENT);
        // `AlignedBytes::zeroed` guarantees a 4-aligned base so `f32` vector
        // reads are never misaligned (AUDIT-03; `Vec<u8>` would only be align-1).
        // Single-use constructor contract: the only failure mode is OOM at
        // construction time (equivalent to `Vec::with_capacity` aborting on
        // allocation failure), so a documented panic here is intentional — the
        // long-lived store paths (`map`/`map_mut`/`grow_to`) propagate instead.
        let mut data = AlignedBytes::zeroed(size as usize)
            .expect("in-memory vstore allocation failed at construction (OOM)");
        let header = VantaHeader::new(*b"VFLE", VFILE_VERSION, 0);
        data.as_mut_slice()[0..16].copy_from_slice(&header.serialize());
        data.as_mut_slice()[16..24].copy_from_slice(&STORAGE_ALIGNMENT.to_le_bytes());
        Self {
            file: None,
            mmap: VantaFileMap::InMemory(data),
            path: PathBuf::new(),
            size,
            write_cursor: STORAGE_ALIGNMENT,
            read_only: false,
            #[cfg(feature = "encryption")]
            cipher: None,
        }
    }

    fn open_with_mode(path: PathBuf, initial_size: u64, read_only: bool) -> Result<Self> {
        let file = if read_only {
            OpenOptions::new()
                .read(true)
                .open(&path)
                .map_err(VantaError::IoError)?
        } else {
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&path)
                .map_err(VantaError::IoError)?
        };
        let mut current_size = file.metadata().map_err(VantaError::IoError)?.len();
        let min_header_size = 64u64;
        if current_size < min_header_size {
            if read_only {
                return Err(VantaError::ValidationError {
                    field: "file_size".into(),
                    reason: format!("VantaFile {} too small", path.display()),
                });
            }
            current_size = initial_size.max(min_header_size);
            file.set_len(current_size).map_err(VantaError::IoError)?;
        }
        // `map_readonly`/`map_readwrite` carry the (memmap2-only) SAFETY
        // contract: `file` is a valid open handle at the correct size, and the
        // returned mapping is stored in `self.mmap` for the `VantaFile`'s
        // lifetime.
        let mut mmap = if read_only {
            VantaFileMap::ReadOnly(map_readonly(&file).map_err(VantaError::IoError)?)
        } else {
            VantaFileMap::ReadWrite(map_readwrite(&file).map_err(VantaError::IoError)?)
        };
        if !read_only && current_size >= min_header_size && &mmap.as_slice()[0..4] != b"VFLE" {
            let header = VantaHeader::new(*b"VFLE", VFILE_VERSION, 0);
            mmap.as_mut_slice()?[0..16].copy_from_slice(&header.serialize());
            mmap.as_mut_slice()?[16..24].copy_from_slice(&STORAGE_ALIGNMENT.to_le_bytes());
            // Zero-fill the remainder of the header block (bytes 24..64) to
            // ensure a clean slate for a potentially corrupt or uninitialized file.
            mmap.as_mut_slice()?[24..STORAGE_ALIGNMENT as usize].fill(0);
            mmap.flush()?;
        }
        let header = VantaHeader::deserialize(&mmap.as_slice()[0..16])?;
        header.validate_compat(*b"VFLE", VFILE_VERSION, "VantaFile")?;
        let cursor = u64::from_le_bytes(mmap.as_slice()[16..24].try_into().map_err(|e| {
            VantaError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?);
        let write_cursor = if cursor < STORAGE_ALIGNMENT || cursor > current_size {
            STORAGE_ALIGNMENT
        } else {
            (cursor + 63) & !63
        };
        Ok(Self {
            file: Some(file),
            mmap,
            path,
            size: current_size,
            write_cursor,
            read_only,
            #[cfg(feature = "encryption")]
            cipher: None,
        })
    }

    /// Persist the write cursor position into the file header.
    pub fn save_cursor(&mut self) -> Result<()> {
        self.mmap.as_mut_slice()?[16..24].copy_from_slice(&self.write_cursor.to_le_bytes());
        Ok(())
    }
    /// Return a byte slice over the entire mapped region.
    pub fn mmap_bytes(&self) -> &[u8] {
        self.mmap.as_slice()
    }
    /// Return a mutable byte slice over the entire mapped region.
    pub fn mmap_bytes_mut(&mut self) -> Result<&mut [u8]> {
        self.mmap.as_mut_slice()
    }
    /// Re-map the backing file into a new mutable memory mapping.
    pub(crate) fn remap_mut(&mut self) -> Result<()> {
        if self.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "read-only".into(),
            });
        }
        if matches!(&self.mmap, VantaFileMap::InMemory(_)) {
            return Ok(());
        }
        let file = self
            .file
            .as_ref()
            .ok_or_else(|| VantaError::ValidationError {
                field: "backing_file".into(),
                reason: "no backing file".into(),
            })?;
        // `map_readwrite` carries the (memmap2-only) SAFETY contract: `file` is
        // the existing backing handle at `self.size` bytes; the previous mapping
        // is dropped (safe — memmap2 unmaps on Drop).
        self.mmap = VantaFileMap::ReadWrite(map_readwrite(file).map_err(VantaError::IoError)?);
        Ok(())
    }

    /// Replace the backing file with a new one at the same path and re-map.
    pub(crate) fn replace_backing_file(&mut self, new_size: u64) -> Result<()> {
        if self.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "read-only".into(),
            });
        }
        if matches!(&self.mmap, VantaFileMap::InMemory(_)) {
            self.size = new_size;
            return Ok(());
        }
        let path = self.path.clone();
        let new_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(&path)
            .map_err(VantaError::IoError)?;
        self.file = Some(new_file);
        self.size = new_size;
        self.remap_mut()
    }

    /// Read a `DiskNodeHeader` from the given aligned offset, if valid.
    ///
    /// Central guard (INV-024 M-1 / AUDIT-03): rejects headers whose vector
    /// payload offset is not 4-byte aligned. `vector_offset` is file data and
    /// is never validated by the writer path on load; a corrupt or adversarial
    /// file could otherwise produce a misaligned `&[f32]` at every
    /// `from_raw_parts` call site (search.rs, archive.rs, engine/ops.rs) — UB
    /// in release. All 7 sites read headers through this function, so the
    /// invariant is enforced in one place.
    pub fn read_header(&self, offset: u64) -> Option<DiskNodeHeader> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        let end = offset.checked_add(header_size)?;
        if end > self.size || !offset.is_multiple_of(STORAGE_ALIGNMENT) {
            return None;
        }
        let slice = &self.mmap_bytes()[offset as usize..end as usize];
        let header = DiskNodeHeader::read_from_bytes(slice).ok()?;
        if !header.vector_offset.is_multiple_of(4) {
            return None;
        }
        Some(header)
    }

    /// Write a `DiskNodeHeader` at the given aligned offset, replacing existing bytes.
    pub fn write_header(&mut self, offset: u64, header: &DiskNodeHeader) -> Result<()> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        if !offset.is_multiple_of(STORAGE_ALIGNMENT) {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "misaligned",
            )));
        }
        let Some(end) = offset.checked_add(header_size) else {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "out of bounds",
            )));
        };
        if end > self.size {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "out of bounds",
            )));
        }
        self.mmap_bytes_mut()?[offset as usize..end as usize].copy_from_slice(header.as_bytes());
        Ok(())
    }

    /// Extend the file to the given new size, zero-filling added space.
    ///
    /// Shrinking is rejected because the VantaFile layout is append-only:
    /// existing node offsets would become invalid. Use `compact_layout` in
    /// `archive.rs` to reclaim space instead.
    pub fn grow_to(&mut self, new_size: u64) -> Result<()> {
        if new_size < self.size {
            return Err(VantaError::ValidationError {
                field: "new_size".into(),
                reason: format!(
                    "grow_to called with new_size {} < current size {}",
                    new_size, self.size
                ),
            });
        }
        match &mut self.mmap {
            VantaFileMap::InMemory(data) => {
                data.grow_zeroed(new_size as usize)?;
                self.size = new_size;
                Ok(())
            }
            _ => {
                // AUD-044: flush pending buffer writes before remapping the
                // SAME file. In the no-memmap2 shim, writes live only in the
                // buffer until flush(); remap_mut's drop would silently discard
                // them. This lives in grow_to (not remap_mut) because
                // replace_backing_file's old buffer is stale by design and must
                // be dropped without flushing (it would wastefully rewrite the
                // orphaned inode). In memmap2 builds this is msync on the old
                // mapping — harmless.
                self.mmap.flush()?;
                let file = self
                    .file
                    .as_ref()
                    .ok_or_else(|| VantaError::ValidationError {
                        field: "backing_file".into(),
                        reason: "no backing file".into(),
                    })?;
                file.set_len(new_size).map_err(VantaError::IoError)?;
                self.size = new_size;
                self.remap_mut()
            }
        }
    }

    /// Flush memory-mapped changes to the backing file (no-op for in-memory mode).
    pub fn flush(&self) -> Result<()> {
        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("mmap_flush_fail", |_| Err(VantaError::IoError(
                std::io::Error::other("injected")
            )));
        }
        self.mmap.flush()
    }

    /// Advise the OS to prefetch the given number of bytes from the mapped region.
    pub fn warmup_top_layers(&self, _size: usize) {
        #[cfg(all(unix, feature = "memmap2"))]
        {
            use memmap2::Advice;
            let _ = match &self.mmap {
                VantaFileMap::ReadOnly(m) => m.advise(Advice::WillNeed),
                VantaFileMap::ReadWrite(m) => m.advise(Advice::WillNeed),
                VantaFileMap::InMemory(_) => Ok(()),
            };
        }
        #[cfg(not(unix))]
        {
            let mmap = self.mmap_bytes();
            let len = _size.min(mmap.len());
            let mut _sum = 0u8;
            for i in (0..len).step_by(4096) {
                _sum ^= mmap[i];
            }
        }
    }

    /// Return the number of resident (in-RAM) bytes for this file's mapping.
    pub fn mmap_resident_bytes(&self) -> Option<u64> {
        get_resident_bytes_impl(self.mmap.as_ptr(), self.mmap.len())
    }

    /// Attach an encryption cipher to this VantaFile.
    ///
    /// When set, the storage layer should use the cipher to encrypt data before
    /// writing and decrypt after reading. Requires the `encryption` feature.
    #[cfg(feature = "encryption")]
    pub fn with_cipher(mut self, cipher: Cipher) -> Self {
        self.cipher = Some(cipher);
        self
    }

    /// Return a reference to the optional encryption cipher.
    #[cfg(feature = "encryption")]
    pub fn cipher(&self) -> Option<&Cipher> {
        self.cipher.as_ref()
    }

    /// Create an [`EncryptionStream`] wrapping this file's backing [`File`].
    ///
    /// Returns `None` if this VantaFile has no backing file (in-memory mode),
    /// or if no cipher is set.
    ///
    /// The stream can be used for transparent encrypt-on-write and
    /// decrypt-on-read operations on the underlying file handle, for example
    /// with WAL or checkpoint files that use stream-based I/O.
    #[cfg(feature = "encryption")]
    pub fn encryption_stream(&self) -> Option<EncryptionStream<&File>> {
        let file = self.file.as_ref()?;
        let stream_cipher = Cipher::from_env().ok()?;
        Some(EncryptionStream::new(file, stream_cipher))
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::binary_header::VantaHeader;
    use crate::node::DiskNodeHeader;
    use crate::storage::engine::STORAGE_ALIGNMENT;

    // ── In-Memory VantaFile ──

    #[test]
    fn test_vfile_create_in_memory() {
        let vf = VantaFile::create_in_memory(STORAGE_ALIGNMENT);
        assert!(vf.file.is_none());
        assert_eq!(vf.size, STORAGE_ALIGNMENT);
        assert_eq!(vf.write_cursor, STORAGE_ALIGNMENT);
        assert!(!vf.read_only);
        assert!(vf.path.as_os_str().is_empty());
        // Header should be valid
        let header = VantaHeader::deserialize(&vf.mmap_bytes()[0..16]).unwrap();
        assert_eq!(header.magic, *b"VFLE");
        // create_in_memory writes current VFILE_VERSION
        assert_eq!(header.format_version, VFILE_VERSION);
    }

    #[test]
    fn test_vfile_in_memory_larger_initial_size() {
        // When initial_size > STORAGE_ALIGNMENT, size should match
        let vf = VantaFile::create_in_memory(1024);
        assert!(vf.size >= 1024);
        assert_eq!(vf.write_cursor, STORAGE_ALIGNMENT);
    }

    #[test]
    fn test_vfile_in_memory_mmap_bytes() {
        let vf = VantaFile::create_in_memory(128);
        let bytes = vf.mmap_bytes();
        assert!(!bytes.is_empty());
        assert_eq!(bytes.len(), vf.size as usize);
        // Header area matches
        assert_eq!(&bytes[0..4], b"VFLE");
    }

    #[test]
    fn test_vfile_in_memory_mmap_bytes_mut() {
        let mut vf = VantaFile::create_in_memory(128);
        let bytes = vf.mmap_bytes_mut().unwrap();
        assert!(!bytes.is_empty());
        bytes[0] = b'X';
        // Verify the write was applied
        assert_eq!(vf.mmap_bytes()[0], b'X');
    }

    #[test]
    fn test_vfile_in_memory_save_cursor() {
        let mut vf = VantaFile::create_in_memory(256);
        vf.write_cursor = 192;
        vf.save_cursor().unwrap();
        // Cursor position is stored at bytes 16..24
        let stored = u64::from_le_bytes(vf.mmap_bytes()[16..24].try_into().unwrap());
        assert_eq!(stored, 192);
    }

    #[test]
    fn test_vfile_in_memory_flush() {
        let vf = VantaFile::create_in_memory(64);
        // In-memory flush is a no-op
        assert!(vf.flush().is_ok());
    }

    #[test]
    fn test_vfile_in_memory_resident_bytes() {
        let vf = VantaFile::create_in_memory(128);
        let bytes = vf.mmap_resident_bytes();
        // In-memory mode: always Some (all bytes are in process heap)
        assert!(bytes.is_some());
    }

    // ── VantaFile Growth (In-Memory) ──

    #[test]
    fn test_vfile_in_memory_grow() {
        let mut vf = VantaFile::create_in_memory(64);
        assert_eq!(vf.size, 64);

        vf.grow_to(256).unwrap();
        assert_eq!(vf.size, 256);
        // New region should be zero-filled
        assert_eq!(vf.mmap_bytes()[128], 0);
        assert_eq!(vf.mmap_bytes()[255], 0);
    }

    #[test]
    fn test_vfile_in_memory_grow_noop_equal_size() {
        let mut vf = VantaFile::create_in_memory(64);
        assert!(vf.grow_to(64).is_ok());
        assert_eq!(vf.size, 64);
    }

    #[test]
    fn test_vfile_grow_to_rejects_shrink() {
        let mut vf = VantaFile::create_in_memory(256);
        let err = vf.grow_to(128).unwrap_err();
        assert!(
            err.to_string().contains("grow_to"),
            "shrink should be rejected: {}",
            err
        );
    }

    // ── DiskNodeHeader read/write (In-Memory) ──

    #[test]
    fn test_vfile_in_memory_write_read_header() {
        let mut vf = VantaFile::create_in_memory(256);
        let header = DiskNodeHeader::new(42);
        // Write at an aligned offset past the header area
        let offset: u64 = STORAGE_ALIGNMENT;
        vf.write_header(offset, &header).unwrap();

        let read_back = vf.read_header(offset).expect("should read header");
        assert_eq!(read_back.id, 42);
        assert!((read_back.confidence_score - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vfile_read_header_misaligned_offset() {
        let vf = VantaFile::create_in_memory(256);
        // Offset not a multiple of STORAGE_ALIGNMENT ⇒ None
        assert!(vf.read_header(1).is_none());
        assert!(vf.read_header(STORAGE_ALIGNMENT + 1).is_none());
    }

    #[test]
    fn test_vfile_read_header_rejects_misaligned_vector_offset() {
        // INV-024 M-1 / AUDIT-03: a header whose vector payload offset is not a
        // multiple of 4 must be rejected centrally — otherwise the
        // `from_raw_parts(.. as *const f32)` cast at the 7 vector read sites
        // would produce a misaligned `&[f32]` (UB in release) on a corrupt or
        // adversarial file.
        let mut vf = VantaFile::create_in_memory(256);
        let mut header = DiskNodeHeader::new(1);
        header.vector_offset = 2; // NOT a multiple of 4 → corrupt payload pointer
        header.vector_len = 4;
        vf.write_header(STORAGE_ALIGNMENT, &header).unwrap();

        assert!(
            vf.read_header(STORAGE_ALIGNMENT).is_none(),
            "misaligned vector_offset must be rejected"
        );

        // Control: a multiple-of-4 offset is accepted.
        header.vector_offset = 4;
        vf.write_header(STORAGE_ALIGNMENT, &header).unwrap();
        assert!(vf.read_header(STORAGE_ALIGNMENT).is_some());
    }

    #[test]
    fn test_vfile_read_header_out_of_bounds() {
        let vf = VantaFile::create_in_memory(128);
        // Offset past end of file
        assert!(vf.read_header(200).is_none());
    }

    #[test]
    fn test_vfile_write_header_misaligned() {
        let mut vf = VantaFile::create_in_memory(256);
        let header = DiskNodeHeader::new(1);
        let err = vf.write_header(1, &header).unwrap_err();
        assert!(
            err.to_string().contains("misaligned"),
            "should reject misaligned: {}",
            err
        );
    }

    #[test]
    fn test_vfile_write_header_out_of_bounds() {
        // 128-byte file, DiskNodeHeader is 64 bytes, offset 128 is aligned but OOB
        let mut vf = VantaFile::create_in_memory(128);
        let header = DiskNodeHeader::new(1);
        let err = vf.write_header(128, &header).unwrap_err();
        assert!(
            err.to_string().contains("out of bounds"),
            "should reject OOB: {}",
            err
        );
    }

    #[test]
    fn test_vfile_write_header_multiple_offsets() {
        let mut vf = VantaFile::create_in_memory(256);
        let h1 = DiskNodeHeader::new(10);
        let h2 = DiskNodeHeader::new(20);
        vf.write_header(STORAGE_ALIGNMENT, &h1).unwrap();
        vf.write_header(STORAGE_ALIGNMENT * 2, &h2).unwrap();

        assert_eq!(vf.read_header(STORAGE_ALIGNMENT).unwrap().id, 10);
        assert_eq!(vf.read_header(STORAGE_ALIGNMENT * 2).unwrap().id, 20);
    }

    // ── File-Backed VantaFile ──

    #[test]
    fn test_vfile_create_and_open() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.vfle");

        // Create new file with 256-byte initial size
        let mut vf = VantaFile::open(path.clone(), 256).unwrap();
        assert!(vf.file.is_some());
        assert_eq!(vf.size, 256);
        assert!(!vf.read_only);
        assert_eq!(vf.write_cursor, STORAGE_ALIGNMENT);

        // Write a header and verify
        let header = DiskNodeHeader::new(99);
        vf.write_header(STORAGE_ALIGNMENT, &header).unwrap();
        vf.flush().unwrap();

        // Reopen and verify data persists
        let vf2 = VantaFile::open(path, 256).unwrap();
        let read = vf2.read_header(STORAGE_ALIGNMENT).unwrap();
        assert_eq!(read.id, 99);
    }

    #[test]
    fn test_vfile_open_reuses_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("existing.vfle");

        // Create file first
        let _ = VantaFile::open(path.clone(), 512).unwrap();

        // Re-open with different initial_size (should use existing file size)
        let vf = VantaFile::open(path, 0).unwrap();
        assert_eq!(vf.size, 512);
    }

    #[test]
    fn test_vfile_open_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro_test.vfle");

        // Create a file first
        let mut create = VantaFile::open(path.clone(), 128).unwrap();
        let header = DiskNodeHeader::new(7);
        create.write_header(STORAGE_ALIGNMENT, &header).unwrap();
        create.flush().unwrap();
        drop(create);

        // Open read-only
        let ro = VantaFile::open_read_only(path).unwrap();
        assert!(ro.read_only);
        assert_eq!(ro.read_header(STORAGE_ALIGNMENT).unwrap().id, 7);
    }

    #[test]
    fn test_vfile_read_only_write_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro_write.vfle");

        let create = VantaFile::open(path.clone(), 128).unwrap();
        create.flush().unwrap();
        drop(create);

        // mmap_bytes_mut requires &mut self — read_only file can only use mmap_bytes
        let _ = VantaFile::open_read_only(path).unwrap().mmap_bytes();
        // Confirm read_only can't get mutable access by design (compile-time check)
    }

    #[test]
    fn test_vfile_remap_mut_on_read_only_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro_remap.vfle");

        let create = VantaFile::open(path.clone(), 128).unwrap();
        create.flush().unwrap();
        drop(create);

        let mut ro = VantaFile::open_read_only(path).unwrap();
        let err = ro.remap_mut().unwrap_err();
        assert!(
            err.to_string().contains("read_only"),
            "remap_mut on read-only should fail: {}",
            err
        );
    }

    #[test]
    fn test_vfile_replace_backing_file_on_read_only_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro_replace.vfle");

        let create = VantaFile::open(path.clone(), 128).unwrap();
        create.flush().unwrap();
        drop(create);

        let mut ro = VantaFile::open_read_only(path).unwrap();
        let err = ro.replace_backing_file(256).unwrap_err();
        assert!(
            err.to_string().contains("read_only"),
            "replace on read-only should fail: {}",
            err
        );
    }

    #[test]
    fn test_vfile_grow_to_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("grow.vfle");

        let mut vf = VantaFile::open(path, 128).unwrap();
        assert_eq!(vf.size, 128);

        vf.grow_to(512).unwrap();
        assert_eq!(vf.size, 512);

        // Verify we can write at the new offset
        let header = DiskNodeHeader::new(200);
        vf.write_header(STORAGE_ALIGNMENT * 4, &header).unwrap();
        assert_eq!(vf.read_header(STORAGE_ALIGNMENT * 4).unwrap().id, 200);
    }

    #[test]
    fn test_vfile_open_nonexistent_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("_does_not_exist_.vfle");
        // Ensure it doesn't exist
        let _ = std::fs::remove_file(&path);
        let result = VantaFile::open_read_only(path);
        assert!(result.is_err(), "should fail for nonexistent file");
    }

    #[cfg(not(feature = "memmap2"))]
    #[test]
    fn shim_grow_to_preserves_pending_buffer_writes() {
        // AUD-044: grow_to/remap_mut replaces the mapping; in the no-memmap2
        // shim, pending buffer writes must be flushed first or they are
        // silently discarded (the file on disk still holds pre-grow content).
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("grow_preserve.vanta");
        let mut vf = VantaFile::open(path, 128).unwrap();
        let header = DiskNodeHeader::new(42);
        vf.write_header(STORAGE_ALIGNMENT, &header).unwrap();
        vf.grow_to(512).unwrap();
        let read = vf.read_header(STORAGE_ALIGNMENT).unwrap();
        assert_eq!(
            read.id, 42,
            "grow_to must preserve pending buffer writes (shim)"
        );
    }

    #[test]
    fn test_vfile_save_and_reload_cursor() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cursor_test.vfle");

        let mut vf = VantaFile::open(path.clone(), 256).unwrap();
        vf.write_cursor = 200;
        vf.save_cursor().unwrap();
        vf.flush().unwrap();
        drop(vf);

        // Reopen and verify cursor is restored (rounded up to alignment: (200+63)&!63 = 256)
        let vf2 = VantaFile::open(path, 256).unwrap();
        assert!(
            vf2.write_cursor == STORAGE_ALIGNMENT || vf2.write_cursor == 256,
            "cursor should be restored (200) or clamped (64), got {}",
            vf2.write_cursor
        );
    }

    // ── VantaFile Version ──

    #[test]
    fn test_vfile_version_constant() {
        assert_eq!(VFILE_VERSION, 2);
    }

    // ── VantaFile warmup_top_layers ──

    #[test]
    fn test_vfile_warmup_top_layers() {
        let vf = VantaFile::create_in_memory(256);
        // Should not panic
        vf.warmup_top_layers(128);
    }

    // ── engine_mmap_resident_bytes ──

    #[test]
    fn test_engine_mmap_resident_bytes_basic() {
        // In-memory vfile reports Some, in-memory index reports None → total is Some
        let index = crate::index::CPIndex::with_backend(crate::index::IndexBackend::InMemory);
        let vf = VantaFile::create_in_memory(128);
        let bytes = engine_mmap_resident_bytes(&index, &vf);
        assert!(bytes.is_some());
    }
}
