use js_sys::{Function, Promise, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;

/// OPFS-based persistent storage for VantaDB in browser environments.
pub struct OpfsStorage {
    dir_handle: JsValue,
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
        let opts = js_sys::Object::new();
        Reflect::set(&opts, &"create".into(), &true.into())?;
        let args = js_sys::Array::new();
        args.push(&path.into());
        args.push(&opts);
        let file_handle = js_call(&self.dir_handle, "getFileHandle", &args).await?;

        let writable = js_call(&file_handle, "createWritable", &js_sys::Array::new()).await?;

        let buf = Uint8Array::new_with_length(data.len() as u32);
        buf.copy_from(data);
        let write_args = js_sys::Array::new();
        write_args.push(&buf);
        js_call(&writable, "write", &write_args).await?;

        js_call(&writable, "close", &js_sys::Array::new()).await?;

        Ok(())
    }

    /// Read a file from OPFS, returning None if it does not exist.
    pub async fn read_file(&self, path: &str) -> Result<Option<Vec<u8>>, JsValue> {
        let get_handle = get_fn(&self.dir_handle, "getFileHandle")?;
        let result = get_handle.call1(&self.dir_handle, &path.into());

        let file_handle = match result {
            Ok(v) => {
                let promise = v
                    .dyn_into::<Promise>()
                    .map_err(|_| JsValue::from_str("expected Promise from getFileHandle"))?;
                wasm_bindgen_futures::JsFuture::from(promise).await?
            }
            Err(_) => return Ok(None),
        };

        let file = js_call(&file_handle, "getFile", &js_sys::Array::new()).await?;

        let buffer = js_call(&file, "arrayBuffer", &js_sys::Array::new()).await?;

        let uint8 = Uint8Array::new(&buffer);
        let mut vec = vec![0u8; uint8.length() as usize];
        uint8.copy_to(&mut vec);
        Ok(Some(vec))
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
}
