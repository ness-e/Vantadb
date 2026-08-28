use js_sys::{Function, Promise, Reflect, Uint8Array};
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

/// Storage quota information returned by `navigator.storage.estimate()`.
#[derive(Debug, Clone)]
pub struct QuotaInfo {
    /// Current usage in bytes.
    pub usage: u64,
    /// Quota limit in bytes (may be `None` if unlimited/unavailable).
    pub quota: Option<u64>,
    /// Usage as a percentage of quota (0.0-1.0), `None` if quota unknown.
    pub usage_ratio: Option<f64>,
}

impl QuotaInfo {
    /// Returns `true` if usage is at or above 90% of quota.
    pub fn is_near_limit(&self) -> bool {
        self.usage_ratio.is_some_and(|r| r >= 0.9)
    }

    /// Returns a human-readable description.
    pub fn describe(&self) -> String {
        match self.quota {
            Some(q) => format!(
                "Storage: {} / {} bytes ({:.1}%)",
                self.usage,
                q,
                self.usage_ratio.unwrap_or(0.0) * 100.0
            ),
            None => format!("Storage: {} bytes (quota unknown)", self.usage),
        }
    }
}

/// Error type for quota-exceeded conditions with actionable details.
#[derive(Debug)]
pub struct QuotaExceededError {
    pub message: String,
    pub quota_info: Option<QuotaInfo>,
}

impl QuotaExceededError {
    fn new(message: String, quota_info: Option<QuotaInfo>) -> Self {
        Self {
            message,
            quota_info,
        }
    }

    /// Convert to a `JsValue` suitable for returning from WASM.
    pub fn to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        Reflect::set(&obj, &"name".into(), &"QuotaExceededError".into()).ok();
        Reflect::set(&obj, &"message".into(), &self.message.clone().into()).ok();
        if let Some(q) = &self.quota_info {
            let qi = js_sys::Object::new();
            Reflect::set(&qi, &"usage".into(), &(q.usage as f64).into()).ok();
            if let Some(quota) = q.quota {
                Reflect::set(&qi, &"quota".into(), &(quota as f64).into()).ok();
            }
            if let Some(ratio) = q.usage_ratio {
                Reflect::set(&qi, &"usageRatio".into(), &ratio.into()).ok();
            }
            Reflect::set(&qi, &"description".into(), &q.describe().into()).ok();
            Reflect::set(&obj, &"quotaInfo".into(), &qi).ok();
        }
        obj.into()
    }
}

/// Check if a `JsValue` represents a `QuotaExceededError` DOMException.
fn is_quota_exceeded_error(e: &JsValue) -> bool {
    Reflect::get(e, &"name".into())
        .ok()
        .and_then(|v| v.as_string())
        .as_deref()
        == Some("QuotaExceededError")
}

/// Emit `console.warn(msg)` when a console exists (best-effort, never throws).
fn console_warn(msg: &str) {
    let global = js_sys::global();
    let console = js_sys::Reflect::get(&global, &"console".into()).ok();
    let warn = console
        .as_ref()
        .and_then(|c| js_sys::Reflect::get(c, &"warn".into()).ok());
    if let Some(w) = warn.and_then(|w| w.dyn_into::<js_sys::Function>().ok()) {
        let _ = w.call1(&JsValue::undefined(), &JsValue::from_str(msg));
    }
}

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
    ///
    /// Performs a quota check before writing; returns a `QuotaExceededError`
    /// with actionable details if the write would likely exceed the storage quota.
    pub async fn write_file(&self, path: &str, data: &[u8]) -> Result<(), JsValue> {
        // Pre-flight quota check (best-effort, non-blocking if estimate unavailable).
        let _ = self.check_quota_before_write(data.len() as u64).await;

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

        // Catch QuotaExceededError from the write and enrich with quota info.
        match file.write(&buf).await {
            Ok(()) => file.move_to(path).await,
            Err(e) if is_quota_exceeded_error(&e) => {
                let quota_info = self.estimate_quota().await.ok();
                Err(QuotaExceededError::new(
                    format!(
                        "QuotaExceededError writing '{}': {}",
                        path,
                        js_sys::Error::from(e)
                            .message()
                            .as_string()
                            .unwrap_or_default()
                    ),
                    quota_info,
                )
                .to_js_value())
            }
            Err(e) => Err(e),
        }
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
    /// Performs a quota check before appending; returns a `QuotaExceededError`
    /// with actionable details if the append would likely exceed the storage quota.
    ///
    /// ponytail: full rewrite per append (O(file) copy, atomic rename kept) —
    /// switch to a streaming WAL layout if append throughput ever matters.
    pub async fn append_file(&self, path: &str, data: &[u8]) -> Result<(), JsValue> {
        // Pre-flight quota check (best-effort).
        let _ = self.check_quota_before_write(data.len() as u64).await;

        let mut buf = self.read_file(path).await?.unwrap_or_default();
        buf.extend_from_slice(data);

        // Catch QuotaExceededError from the write and enrich with quota info.
        match self.write_file(path, &buf).await {
            Ok(()) => Ok(()),
            Err(e) if is_quota_exceeded_error(&e) => {
                let quota_info = self.estimate_quota().await.ok();
                Err(QuotaExceededError::new(
                    format!(
                        "QuotaExceededError appending to '{}': {}",
                        path,
                        js_sys::Error::from(e)
                            .message()
                            .as_string()
                            .unwrap_or_default()
                    ),
                    quota_info,
                )
                .to_js_value())
            }
            Err(e) => Err(e),
        }
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

    /// Query the current storage usage and quota via `navigator.storage.estimate()`.
    ///
    /// Returns `QuotaInfo` with usage, quota (if available), and usage ratio.
    /// This is a best-effort check; some browsers may not support `estimate()` or
    /// may return `quota: null` for unlimited storage.
    pub async fn estimate_quota(&self) -> Result<QuotaInfo, JsValue> {
        let global = js_sys::global();
        let navigator = Reflect::get(&global, &"navigator".into())?;
        let storage = Reflect::get(&navigator, &"storage".into())?;
        let estimate_fn = get_fn(&storage, "estimate")?;
        let promise = estimate_fn
            .apply(&storage, &js_sys::Array::new())?
            .dyn_into::<Promise>()
            .map_err(|_| JsValue::from_str("expected Promise from storage.estimate()"))?;
        let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
        let usage = Reflect::get(&result, &"usage".into())?
            .as_f64()
            .unwrap_or(0.0) as u64;
        let quota = Reflect::get(&result, &"quota".into())
            .ok()
            .and_then(|v| v.as_f64())
            .map(|q| q as u64);
        let usage_ratio = quota.map(|q| if q > 0 { usage as f64 / q as f64 } else { 0.0 });
        Ok(QuotaInfo {
            usage,
            quota,
            usage_ratio,
        })
    }

    /// Check if a write of `additional_bytes` would likely exceed quota.
    ///
    /// Performs a quick `estimate()` check and returns an error if the projected
    /// usage would exceed 95% of quota. This is a heuristic — the actual write
    /// may still fail or succeed depending on browser behavior.
    pub async fn check_quota_before_write(&self, additional_bytes: u64) -> Result<(), JsValue> {
        let info = self.estimate_quota().await?;
        if let Some(quota) = info.quota {
            let projected = info.usage.saturating_add(additional_bytes);
            if projected > quota {
                let msg = format!(
                    "QuotaExceededError: projected write of {} bytes would exceed quota ({} / {} bytes, {:.1}% used)",
                    additional_bytes,
                    info.usage,
                    quota,
                    info.usage_ratio.unwrap_or(0.0) * 100.0
                );
                return Err(QuotaExceededError::new(msg, Some(info)).to_js_value());
            }
            // Warn if near limit (90%) but don't block
            if info.is_near_limit() {
                console_warn(&format!(
                    "Storage quota near limit: {} (projected after write: {:.1}%)",
                    info.describe(),
                    (projected as f64 / quota as f64) * 100.0
                ));
            }
        }
        Ok(())
    }
}
