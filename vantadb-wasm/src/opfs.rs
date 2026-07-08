use js_sys::{Function, Promise, Reflect, Uint8Array};
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
                wasm_bindgen_futures::JsFuture::from(promise).await?
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
    pub async fn append(&self, data: &[u8]) -> Result<(), JsValue> {
        let opts = js_sys::Object::new();
        Reflect::set(&opts, &"keepExistingData".into(), &true.into())?;
        let args = js_sys::Array::new();
        args.push(&opts);
        let writable = js_call(&self.handle, "createWritable", &args).await?;
        let buf = Uint8Array::new_with_length(data.len() as u32);
        buf.copy_from(data);
        let write_args = js_sys::Array::new();
        write_args.push(&buf);
        js_call(&writable, "write", &write_args).await?;
        js_call(&writable, "close", &js_sys::Array::new()).await?;
        Ok(())
    }

    /// Delete the file from OPFS. Returns `Ok(true)` if deleted, `Ok(false)` if not found.
    pub async fn delete(&self) -> Result<bool, JsValue> {
        Err(JsValue::from_str(
            "OpfsFile::delete requires the parent directory handle — use OpfsStorage::delete_file instead",
        ))
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
    pub async fn write_file(&self, path: &str, data: &[u8]) -> Result<(), JsValue> {
        let file = OpfsFile::open(&self.dir_handle, path, true)
            .await?
            .expect("OpfsFile::open with create=true should succeed");
        file.write(data).await
    }

    /// Read a file from OPFS, returning None if it does not exist.
    pub async fn read_file(&self, path: &str) -> Result<Option<Vec<u8>>, JsValue> {
        let file = match OpfsFile::open(&self.dir_handle, path, false).await? {
            Some(f) => f,
            None => return Ok(None),
        };
        file.read().await.map(Some)
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

    /// Append data to an existing file. Creates the file if it doesn't exist.
    pub async fn append_file(&self, path: &str, data: &[u8]) -> Result<(), JsValue> {
        let file = OpfsFile::open(&self.dir_handle, path, true)
            .await?
            .expect("OpfsFile::open with create=true should succeed");
        file.append(data).await
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
