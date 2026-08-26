use js_sys::{Function, Promise, Reflect, Uint8Array};
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

/// A handle to an open OPFS file, wrapping a JS `FileSystemFileHandle`.
///
/// Provides read, write, append, and delete operations on a single file
/// within the Origin Private File System.
pub struct OpfsFile {
    handle: JsValue,
}

impl OpfsFile {
    /// Open or create a file at `path` inside the given directory handle.
    /// Returns `None` if the file does not exist and `create` is `false`.
    pub async fn open(
        dir_handle: &JsValue,
        path: &str,
        create: bool,
    ) -> Result<Option<Self>, JsValue> {
        let opts = js_sys::Object::new();
        Reflect::set(&opts, &"create".into(), &create.into())?;
        let args = js_sys::Array::new();
        args.push(&path.into());
        args.push(&opts);
        let get_handle = get_fn(dir_handle, "getFileHandle")?;
        let result = get_handle.apply(dir_handle, &args);
        let handle = match result {
            Ok(v) => {
                let promise = v
                    .dyn_into::<Promise>()
                    .map_err(|_| JsValue::from_str("expected Promise from getFileHandle"))?;
                match wasm_bindgen_futures::JsFuture::from(promise).await {
                    Ok(v) => v,
                    Err(e) => {
                        // getFileHandle rejects with NotFoundError when the file
                        // does not exist and create=false. Treat that as "absent"
                        // (Ok(None)) instead of an error, so read_file/load can
                        // open a fresh storage directory on first run.
                        if !create {
                            let name = Reflect::get(&e, &"name".into())
                                .ok()
                                .and_then(|v| v.as_string());
                            if name.as_deref() == Some("NotFoundError") {
                                return Ok(None);
                            }
                        }
                        return Err(e);
                    }
                }
            }
            Err(_) => {
                if create {
                    return Err(JsValue::from_str("failed to create file"));
                }
                return Ok(None);
            }
        };
        Ok(Some(Self { handle }))
    }

    /// Read the entire file contents as a `Vec<u8>`.
    pub async fn read(&self) -> Result<Vec<u8>, JsValue> {
        let file = js_call(&self.handle, "getFile", &js_sys::Array::new()).await?;
        let buffer = js_call(&file, "arrayBuffer", &js_sys::Array::new()).await?;
        let uint8 = Uint8Array::new(&buffer);
        let mut vec = vec![0u8; uint8.length() as usize];
        uint8.copy_to(&mut vec);
        Ok(vec)
    }

    /// Write data to the file, replacing its current contents.
    pub async fn write(&self, data: &[u8]) -> Result<(), JsValue> {
        let writable = js_call(&self.handle, "createWritable", &js_sys::Array::new()).await?;
        let buf = Uint8Array::new_with_length(data.len() as u32);
        buf.copy_from(data);
        let write_args = js_sys::Array::new();
        write_args.push(&buf);
        js_call(&writable, "write", &write_args).await?;
        js_call(&writable, "close", &js_sys::Array::new()).await?;
        Ok(())
    }

    /// Append data to the end of the file.
    ///
    /// Computes the current file size first and writes at that offset,
    /// mirroring `opfs_bridge.js::appendFile`: `keepExistingData` alone is
    /// not enough because a bare `write(data)` starts at position 0 and
    /// overwrites the head of the file.
    pub async fn append(&self, data: &[u8]) -> Result<(), JsValue> {
        let file = js_call(&self.handle, "getFile", &js_sys::Array::new()).await?;
        let size = Reflect::get(&file, &"size".into())?;
        let opts = js_sys::Object::new();
        Reflect::set(&opts, &"keepExistingData".into(), &true.into())?;
        let args = js_sys::Array::new();
        args.push(&opts);
        let writable = js_call(&self.handle, "createWritable", &args).await?;
        let buf = Uint8Array::new_with_length(data.len() as u32);
        buf.copy_from(data);
        let write_opts = js_sys::Object::new();
        Reflect::set(&write_opts, &"type".into(), &"write".into())?;
        Reflect::set(&write_opts, &"position".into(), &size)?;
        Reflect::set(&write_opts, &"data".into(), &buf)?;
        let write_args = js_sys::Array::new();
        write_args.push(&write_opts);
        js_call(&writable, "write", &write_args).await?;
        js_call(&writable, "close", &js_sys::Array::new()).await?;
        Ok(())
    }

    /// Delete the file from OPFS. Returns `Ok(true)` if deleted.
    pub async fn delete(&self) -> Result<bool, JsValue> {
        js_call(&self.handle, "remove", &js_sys::Array::new()).await?;
        Ok(true)
    }

    /// Atomically rename this file within its OPFS directory.
    /// The handle remains valid and points to the new name.
    pub async fn move_to(&self, new_name: &str) -> Result<(), JsValue> {
        let args = js_sys::Array::new();
        args.push(&new_name.into());
        js_call(&self.handle, "move", &args).await?;
        Ok(())
    }
}

fn get_fn(obj: &JsValue, method: &str) -> Result<Function, JsValue> {
    let val = Reflect::get(obj, &method.into())?;
    val.dyn_into::<Function>()
}

async fn js_call(obj: &JsValue, method: &str, args: &js_sys::Array) -> Result<JsValue, JsValue> {
    let func = get_fn(obj, method)?;
    let result = func.apply(obj, args)?;
    let promise = result
        .dyn_into::<Promise>()
        .map_err(|_| JsValue::from_str("expected Promise from OPFS API"))?;
    wasm_bindgen_futures::JsFuture::from(promise).await
}

/// CRC-32 checksum table, computed once and cached.
fn crc32_table() -> &'static [u32; 256] {
    static TABLE: OnceLock<[u32; 256]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0u32; 256];
        for i in 0..256u32 {
            let mut crc = i;
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    crc >> 1 ^ 0xEDB88320
                } else {
                    crc >> 1
                };
            }
            table[i as usize] = crc;
        }
        table
    })
}

/// Compute CRC-32 checksum over `data` using the standard IEEE polynomial.
fn crc32(data: &[u8]) -> u32 {
    let table = crc32_table();
    let mut crc = !0u32;
    for &byte in data {
        crc = table[((crc as u8) ^ byte) as usize] ^ (crc >> 8);
    }
    !crc
}

/// OPFS-based persistent storage for VantaDB in browser environments.
///
/// Provides a simple KV-store interface over files in a dedicated OPFS directory.
/// Each file is a key; file contents are the values.
pub struct OpfsStorage {
    dir_handle: JsValue,
}

impl OpfsStorage {
    /// Open or create an OPFS storage directory with the given name.
    pub async fn open(name: &str) -> Result<Self, JsValue> {
        let global = js_sys::global();
        let navigator = Reflect::get(&global, &"navigator".into())?;
        let storage = Reflect::get(&navigator, &"storage".into())?;
        let root = js_call(&storage, "getDirectory", &js_sys::Array::new()).await?;
        let opts = js_sys::Object::new();
        Reflect::set(&opts, &"create".into(), &true.into())?;
        let args = js_sys::Array::new();
        args.push(&name.into());
        args.push(&opts);
        let dir_handle = js_call(&root, "getDirectoryHandle", &args).await?;
        Ok(Self { dir_handle })
    }

    /// Write data to a file at the given path in OPFS.
    ///
    /// Uses an atomic write strategy: writes to a temp file first, then
    /// renames to the final path. Appends a CRC-32 footer to detect
    /// corruption on read.
    pub async fn write_file(&self, path: &str, data: &[u8]) -> Result<(), JsValue> {
        // Append CRC-32 footer so read_file can detect corruption.
        let checksum = crc32(data);
        let mut buf = Vec::with_capacity(data.len() + 4);
        buf.extend_from_slice(data);
        buf.extend_from_slice(&checksum.to_le_bytes());

        // Write to a temp file, then atomically rename to the final path.
        let tmp_path = format!("{}.tmp", path);
        let file = OpfsFile::open(&self.dir_handle, &tmp_path, true)
            .await?
            .ok_or_else(|| JsValue::from_str("OpfsFile::open returned None with create=true"))?;
        file.write(&buf).await?;
        file.move_to(path).await
    }

    /// Read a file from OPFS, returning None if it does not exist.
    ///
    /// Verifies the CRC-32 footer appended by `write_file`. A footer
    /// mismatch (or a file too short to carry one) means the stored bytes
    /// are corrupt or were written by a foreign tool — this errors with a
    /// descriptive message instead of returning raw bytes that would later
    /// explode in JSON parsing far away from the actual cause.
    pub async fn read_file(&self, path: &str) -> Result<Option<Vec<u8>>, JsValue> {
        let file = match OpfsFile::open(&self.dir_handle, path, false).await? {
            Some(f) => f,
            None => return Ok(None),
        };
        let data = file.read().await?;
        // Verify the CRC-32 footer: the last 4 bytes must checksum the rest.
        // Anything shorter cannot have been written by `write_file`.
        if data.len() < 4 {
            return Err(JsValue::from_str(&format!(
                "storage corrupted: '{path}' is {} bytes, too short for a CRC-footer file",
                data.len()
            )));
        }
        let split = data.len() - 4;
        let stored = u32::from_le_bytes([
            data[split],
            data[split + 1],
            data[split + 2],
            data[split + 3],
        ]);
        let actual = crc32(&data[..split]);
        if stored != actual {
            return Err(JsValue::from_str(&format!(
                "storage corrupted: CRC-32 mismatch reading '{path}' \
                 (stored {stored:#010x}, computed {actual:#010x})"
            )));
        }
        Ok(Some(data[..split].to_vec()))
    }

    /// Delete a file at the given path from OPFS.
    pub async fn delete_file(&self, path: &str) -> Result<(), JsValue> {
        let remove = get_fn(&self.dir_handle, "removeEntry")?;
        let result = remove.call1(&self.dir_handle, &path.into());
        if let Err(e) = result {
            let name = Reflect::get(&e, &"name".into())
                .ok()
                .and_then(|v| v.as_string());
            if name.as_deref() == Some("NotFoundError") {
                return Ok(());
            }
            return Err(e);
        }
        Ok(())
    }

    /// Append data to an existing file, keeping the CRC-footer format used
    /// by [`OpfsStorage::write_file`] so the result stays readable through
    /// `read_file`. Creates the file if it doesn't exist.
    ///
    /// ponytail: full rewrite per append (O(file) copy, atomic rename kept) —
    /// switch to a streaming WAL layout if append throughput ever matters.
    pub async fn append_file(&self, path: &str, data: &[u8]) -> Result<(), JsValue> {
        let mut buf = self.read_file(path).await?.unwrap_or_default();
        buf.extend_from_slice(data);
        self.write_file(path, &buf).await
    }

    /// Return the raw JS directory handle (for advanced use).
    pub fn dir_handle(&self) -> &JsValue {
        &self.dir_handle
    }

    /// Check whether OPFS is available in the current environment.
    pub fn is_available() -> bool {
        let global = js_sys::global();
        let navigator = Reflect::get(&global, &"navigator".into()).ok();
        let navigator = match navigator {
            Some(v) => v,
            None => return false,
        };
        let storage = Reflect::get(&navigator, &"storage".into()).ok();
        storage.is_some()
    }
}
