use crate::binary_header::VantaHeader;
use crate::error::{Result, VantaError};
use crate::index::CPIndex;
use crate::node::DiskNodeHeader;
use std::fs::{File, OpenOptions};
#[cfg(not(feature = "memmap2"))]
use std::io::Read;
#[cfg(not(feature = "memmap2"))]
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use tracing::warn;
use zerocopy::{FromBytes, IntoBytes};

#[cfg(feature = "memmap2")]
pub(crate) use memmap2::{Mmap, MmapMut, MmapOptions};

#[cfg(not(feature = "memmap2"))]
pub(crate) mod mmap_shim {
    use super::*;
    #[derive(Debug)]
    pub struct Mmap(Vec<u8>);
    #[derive(Debug)]
    pub struct MmapMut(Vec<u8>);
    pub struct MmapOptions;

    impl MmapOptions {
        pub fn new() -> Self {
            Self
        }
        pub fn map(&self, file: &File) -> std::io::Result<Mmap> {
            let mut v = vec![0u8; file.metadata()?.len() as usize];
            let mut f = file.try_clone()?;
            f.read_exact(&mut v)?;
            Ok(Mmap(v))
        }
        pub fn map_mut(&self, file: &File) -> std::io::Result<MmapMut> {
            let mut v = vec![0u8; file.metadata()?.len() as usize];
            let mut f = file.try_clone()?;
            f.read_exact(&mut v)?;
            Ok(MmapMut(v))
        }
    }
    impl Mmap {
        pub fn map(file: &File) -> std::io::Result<Self> {
            MmapOptions::new().map(file)
        }
        pub fn as_ptr(&self) -> *const u8 {
            self.0.as_ptr()
        }
        pub fn len(&self) -> usize {
            self.0.len()
        }
        pub fn flush(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub fn flush_async(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub fn flush_range(&self, _offset: usize, _len: usize) -> std::io::Result<()> {
            Ok(())
        }
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }
    impl Deref for Mmap {
        type Target = [u8];
        fn deref(&self) -> &[u8] {
            &self.0
        }
    }
    impl MmapMut {
        pub fn map_mut(file: &File) -> std::io::Result<Self> {
            MmapOptions::new().map_mut(file)
        }
        pub fn as_ptr(&self) -> *const u8 {
            self.0.as_ptr()
        }
        pub fn as_mut_ptr(&mut self) -> *mut u8 {
            self.0.as_mut_ptr()
        }
        pub fn len(&self) -> usize {
            self.0.len()
        }
        pub fn flush(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub fn flush_async(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub fn flush_range(&self, _offset: usize, _len: usize) -> std::io::Result<()> {
            Ok(())
        }
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }
    impl Deref for MmapMut {
        type Target = [u8];
        fn deref(&self) -> &[u8] {
            &self.0
        }
    }
    impl DerefMut for MmapMut {
        fn deref_mut(&mut self) -> &mut [u8] {
            &mut self.0
        }
    }
}
#[cfg(not(feature = "memmap2"))]
pub(crate) use mmap_shim::{Mmap, MmapMut, MmapOptions};

#[cfg(unix)]
use std::sync::atomic::AtomicBool as AtomicBoolUnix;

#[cfg(unix)]
use libc;

static SIGBUS_OCCURRED: AtomicBool = AtomicBool::new(false);
#[cfg(unix)]
static SIGBUS_FAULT_ADDR: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());

#[cfg(unix)]
fn install_sigbus_handler() -> Result<()> {
    use std::sync::Once;
    static INSTALL_ONCE: Once = Once::new();
    INSTALL_ONCE.call_once(|| unsafe {
        let mut sa: libc::sigaction = std::mem::zeroed();
        sa.sa_sigaction = sigbus_handler as *const () as usize;
        sa.sa_flags = libc::SA_SIGINFO;
        libc::sigemptyset(&mut sa.sa_mask);
        if libc::sigaction(libc::SIGBUS, &sa, std::ptr::null_mut()) != 0 {
            warn!(
                "Failed to install SIGBUS handler: {}",
                std::io::Error::last_os_error()
            );
        }
    });
    Ok(())
}

#[cfg(unix)]
unsafe extern "C" fn sigbus_handler(
    _signum: libc::c_int,
    siginfo: *mut libc::siginfo_t,
    _context: *mut libc::c_void,
) {
    SIGBUS_OCCURRED.store(true, Ordering::SeqCst);
    if !siginfo.is_null() {
        let addr = (*siginfo).si_addr() as *mut u8;
        SIGBUS_FAULT_ADDR.store(addr, Ordering::SeqCst);
        warn!("SIGBUS occurred at address: {:p}", addr);
    }
}

#[cfg(unix)]
pub fn check_sigbus() -> bool {
    SIGBUS_OCCURRED.swap(false, Ordering::SeqCst)
}

#[cfg(unix)]
pub fn get_sigbus_fault_addr() -> *mut u8 {
    SIGBUS_FAULT_ADDR.load(Ordering::SeqCst)
}

#[cfg(not(unix))]
pub fn check_sigbus() -> bool {
    false
}

#[cfg(not(unix))]
pub fn get_sigbus_fault_addr() -> *mut u8 {
    std::ptr::null_mut()
}

pub const VANTA_FILE_MAGIC: &[u8; 8] = b"VNTAFILE";
pub const VANTA_FILE_VERSION: u32 = 1;

pub fn get_resident_bytes(addr: *const u8, len: usize) -> Option<u64> {
    get_resident_bytes_impl(addr, len)
}

pub fn get_resident_bytes_impl(addr: *const u8, len: usize) -> Option<u64> {
    if len == 0 || addr.is_null() {
        return Some(0);
    }
    #[cfg(unix)]
    {
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        let page_size = if page_size <= 0 {
            4096
        } else {
            page_size as usize
        };
        let addr_val = addr as usize;
        let aligned_addr = addr_val & !(page_size - 1);
        let offset = addr_val - aligned_addr;
        let aligned_len = (len + offset + page_size - 1) & !(page_size - 1);
        let num_pages = aligned_len / page_size;
        let mut resident_pages = 0u64;
        let mut vec_buffer = vec![0u8; num_pages.min(65536)];
        for chunk_start_page in (0..num_pages).step_by(65536) {
            let pages_in_chunk = (num_pages - chunk_start_page).min(65536);
            let chunk_addr = (aligned_addr + chunk_start_page * page_size) as *mut libc::c_void;
            let chunk_len = pages_in_chunk * page_size;
            #[cfg(target_os = "macos")]
            let res = unsafe {
                libc::mincore(
                    chunk_addr,
                    chunk_len,
                    vec_buffer.as_mut_ptr() as *mut libc::c_char,
                )
            };
            #[cfg(not(target_os = "macos"))]
            let res = unsafe { libc::mincore(chunk_addr, chunk_len, vec_buffer.as_mut_ptr()) };
            if res == 0 {
                for &page_state in vec_buffer.iter().take(pages_in_chunk) {
                    if (page_state & 1) != 0 {
                        resident_pages += 1;
                    }
                }
            } else {
                return None;
            }
        }
        Some(resident_pages * page_size as u64)
    }
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::ProcessStatus::{
            QueryWorkingSetEx, PSAPI_WORKING_SET_EX_INFORMATION,
        };
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        let page_size = 4096usize;
        let addr_val = addr as usize;
        let aligned_addr = addr_val & !(page_size - 1);
        let aligned_len =
            ((len + (addr_val - aligned_addr) + page_size - 1) & !(page_size - 1)).max(page_size);
        let num_pages = aligned_len / page_size;
        let mut resident_pages = 0u64;
        let h_process = unsafe { GetCurrentProcess() };
        let mut info_buffer = vec![
            unsafe { std::mem::zeroed::<PSAPI_WORKING_SET_EX_INFORMATION>() };
            num_pages.min(65536)
        ];
        for chunk_start_page in (0..num_pages).step_by(65536) {
            let pages_in_chunk = (num_pages - chunk_start_page).min(65536);
            for (i, entry) in info_buffer.iter_mut().enumerate().take(pages_in_chunk) {
                entry.VirtualAddress =
                    (aligned_addr + (chunk_start_page + i) * page_size) as *mut _;
            }
            let cb =
                (pages_in_chunk * std::mem::size_of::<PSAPI_WORKING_SET_EX_INFORMATION>()) as u32;
            if unsafe { QueryWorkingSetEx(h_process, info_buffer.as_mut_ptr() as *mut _, cb) } != 0
            {
                for entry in info_buffer.iter().take(pages_in_chunk) {
                    if (unsafe { entry.VirtualAttributes.Flags } & 1) != 0 {
                        resident_pages += 1;
                    }
                }
            } else {
                return None;
            }
        }
        Some(resident_pages * page_size as u64)
    }
    #[cfg(not(any(unix, target_os = "windows")))]
    {
        None
    }
}

pub(crate) fn engine_mmap_resident_bytes(hnsw: &CPIndex, vector_store: &VantaFile) -> Option<u64> {
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
    InMemory(Vec<u8>),
}

impl VantaFileMap {
    fn as_slice(&self) -> &[u8] {
        match self {
            VantaFileMap::ReadOnly(m) => m,
            VantaFileMap::ReadWrite(m) => m,
            VantaFileMap::InMemory(d) => d,
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
            VantaFileMap::InMemory(d) => Ok(d),
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

pub struct VantaFile {
    pub file: Option<File>,
    mmap: VantaFileMap,
    pub path: PathBuf,
    pub size: u64,
    pub write_cursor: u64,
    read_only: bool,
}

unsafe impl Send for VantaFile {}
unsafe impl Sync for VantaFile {}

impl VantaFile {
    pub fn open(path: PathBuf, initial_size: u64) -> Result<Self> {
        Self::open_with_mode(path, initial_size, false)
    }
    pub fn open_read_only(path: PathBuf) -> Result<Self> {
        Self::open_with_mode(path, 0, true)
    }

    pub fn create_in_memory(initial_size: u64) -> Self {
        let size = initial_size.max(64);
        let mut data = vec![0u8; size as usize];
        let header = VantaHeader::new(*b"VFLE", 1, 0);
        data[0..16].copy_from_slice(&header.serialize());
        data[16..24].copy_from_slice(&64u64.to_le_bytes());
        Self {
            file: None,
            mmap: VantaFileMap::InMemory(data),
            path: PathBuf::new(),
            size,
            write_cursor: 64,
            read_only: false,
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
        let mut mmap = if read_only {
            VantaFileMap::ReadOnly(unsafe {
                MmapOptions::new().map(&file).map_err(VantaError::IoError)?
            })
        } else {
            VantaFileMap::ReadWrite(unsafe {
                MmapOptions::new()
                    .map_mut(&file)
                    .map_err(VantaError::IoError)?
            })
        };
        if !read_only && current_size >= min_header_size {
            if &mmap.as_slice()[0..4] != b"VFLE" {
                let header = VantaHeader::new(*b"VFLE", 1, 0);
                mmap.as_mut_slice()?[0..16].copy_from_slice(&header.serialize());
                mmap.as_mut_slice()?[16..24].copy_from_slice(&64u64.to_le_bytes());
                mmap.flush()?;
            }
        }
        let header = VantaHeader::deserialize(&mmap.as_slice()[0..16])?;
        header.validate(*b"VFLE", 1, "VantaFile format mismatch")?;
        let cursor = u64::from_le_bytes(mmap.as_slice()[16..24].try_into().map_err(|e| {
            VantaError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?);
        let write_cursor = if cursor < 64 || cursor > current_size {
            64
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
        })
    }

    pub fn save_cursor(&mut self) -> Result<()> {
        self.mmap.as_mut_slice()?[16..24].copy_from_slice(&self.write_cursor.to_le_bytes());
        Ok(())
    }
    pub fn mmap_bytes(&self) -> &[u8] {
        self.mmap.as_slice()
    }
    pub(crate) fn mmap_bytes_mut(&mut self) -> Result<&mut [u8]> {
        self.mmap.as_mut_slice()
    }
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
        self.mmap = VantaFileMap::ReadWrite(unsafe {
            MmapOptions::new()
                .map_mut(file)
                .map_err(VantaError::IoError)?
        });
        Ok(())
    }

    pub fn read_header(&self, offset: u64) -> Option<DiskNodeHeader> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        if offset + header_size > self.size || !offset.is_multiple_of(64) {
            return None;
        }
        let slice = &self.mmap_bytes()[offset as usize..(offset + header_size) as usize];
        DiskNodeHeader::read_from_bytes(slice).ok()
    }

    pub fn write_header(&mut self, offset: u64, header: &DiskNodeHeader) -> Result<()> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        if !offset.is_multiple_of(64) {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "misaligned",
            )));
        }
        if offset + header_size > self.size {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "out of bounds",
            )));
        }
        self.mmap_bytes_mut()?[offset as usize..(offset + header_size) as usize]
            .copy_from_slice(header.as_bytes());
        Ok(())
    }

    pub fn grow_to(&mut self, new_size: u64) -> Result<()> {
        match &mut self.mmap {
            VantaFileMap::InMemory(data) => {
                data.resize(new_size as usize, 0);
                self.size = new_size;
                Ok(())
            }
            _ => {
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

    pub fn flush(&self) -> Result<()> {
        #[cfg(feature = "failpoints")]
        {
            fail::fail_point!("mmap_flush_fail", |_| Err(VantaError::IoError(
                std::io::Error::other("injected")
            )));
        }
        self.mmap.flush()
    }

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

    pub fn mmap_resident_bytes(&self) -> Option<u64> {
        get_resident_bytes_impl(self.mmap.as_ptr(), self.mmap.len())
    }
}
