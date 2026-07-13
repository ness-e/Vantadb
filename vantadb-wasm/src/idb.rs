use js_sys::{Function, Promise, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;

// Inline IndexedDB persistence bridge — no external JS import needed.
#[wasm_bindgen(inline_js = r#"
(function() {
    if (typeof globalThis !== "undefined" && globalThis.vantaIdbStorage) return;
    const DB_NAME = "VantaDB";
    const STORE_NAME = "state";
    const listeners = [];
    let channel = null;
    function notify(key) { for (let i = 0; i < listeners.length; i++) { try { listeners[i](key); } catch (e) { } } }
    function openDB() {
        return new Promise((resolve, reject) => {
            const req = indexedDB.open(DB_NAME, 1);
            req.onupgradeneeded = () => req.result.createObjectStore(STORE_NAME);
            req.onsuccess = () => resolve(req.result);
            req.onerror = () => reject(req.error);
        });
    }
    try { channel = new BroadcastChannel("vantadb-sync"); } catch (e) { }
    if (channel) { channel.onmessage = (ev) => { if (ev.data && ev.data.type === "data-changed") notify(ev.data.key || "db_state.json"); }; }
    const storage = {
        read(key) {
            return openDB().then((db) => new Promise((resolve, reject) => {
                const tx = db.transaction(STORE_NAME, "readonly");
                const req = tx.objectStore(STORE_NAME).get(key);
                req.onsuccess = () => resolve(req.result || null);
                req.onerror = () => { if (req.error && req.error.name === "NotFoundError") resolve(null); else reject(req.error); };
            }));
        },
        write(key, data) {
            return openDB().then((db) => new Promise((resolve, reject) => {
                const tx = db.transaction(STORE_NAME, "readwrite");
                tx.objectStore(STORE_NAME).put(data, key);
                tx.oncomplete = () => { if (channel) channel.postMessage({ type: "data-changed", key }); resolve(); };
                tx.onerror = () => reject(tx.error);
            }));
        },
        del(key) {
            return openDB().then((db) => new Promise((resolve, reject) => {
                const tx = db.transaction(STORE_NAME, "readwrite");
                tx.objectStore(STORE_NAME).delete(key);
                tx.oncomplete = () => { if (channel) channel.postMessage({ type: "data-changed", key }); resolve(); };
                tx.onerror = () => reject(tx.error);
            }));
        },
        subscribe(fn) { listeners.push(fn); return () => { listeners.splice(listeners.indexOf(fn), 1); }; },
        getBroadcastChannel() { return channel ? "vantadb-sync" : null; },
    };
    const g = typeof globalThis !== "undefined" ? globalThis : window;
    g.vantaIdbStorage = storage;
})();
"#)]
extern "C" {
    #[allow(dead_code)]
    fn __vanta_ensure_idb_bridge();
}

fn storage() -> Result<JsValue, JsValue> {
    let val = Reflect::get(&js_sys::global(), &"vantaIdbStorage".into())?;
    if val.is_undefined() {
        return Err(JsValue::from_str(
            "vantaIdbStorage not available — inline bridge failed to register",
        ));
    }
    Ok(val)
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
        .map_err(|_| JsValue::from_str("expected Promise from IndexedDB API"))?;
    wasm_bindgen_futures::JsFuture::from(promise).await
}

/// IndexedDB-based storage for persisting VantaDB state in the browser.
pub struct IdbStorage;

impl IdbStorage {
    /// Check if IndexedDB is available in the current environment.
    pub fn is_available() -> bool {
        let global = js_sys::global();
        Reflect::get(&global, &"indexedDB".into())
            .ok()
            .is_some_and(|v| !v.is_undefined())
    }

    /// Check if the BroadcastChannel API is available for cross-tab sync.
    pub fn has_broadcast_channel() -> bool {
        let global = js_sys::global();
        Reflect::get(&global, &"BroadcastChannel".into())
            .ok()
            .is_some_and(|v| !v.is_undefined())
    }

    /// Read a file from IndexedDB by key. Returns `None` if the key does not exist.
    pub async fn read_file(key: &str) -> Result<Option<Vec<u8>>, JsValue> {
        let s = storage()?;
        let args = js_sys::Array::new();
        args.push(&key.into());
        let result = js_call(&s, "read", &args).await?;
        if result.is_null() || result.is_undefined() {
            return Ok(None);
        }
        let buf = result
            .dyn_into::<js_sys::ArrayBuffer>()
            .map_err(|_| JsValue::from_str("expected ArrayBuffer from IndexedDB read"))?;
        let uint8 = Uint8Array::new(&buf);
        let mut vec = vec![0u8; uint8.length() as usize];
        uint8.copy_to(&mut vec);
        Ok(Some(vec))
    }

    /// Write a file to IndexedDB. Replaces any existing value for the same key.
    pub async fn write_file(key: &str, data: &[u8]) -> Result<(), JsValue> {
        let s = storage()?;
        let buf = Uint8Array::new_with_length(data.len() as u32);
        buf.copy_from(data);
        let args = js_sys::Array::new();
        args.push(&key.into());
        args.push(&buf.buffer());
        js_call(&s, "write", &args).await?;
        Ok(())
    }

    /// Delete a persisted key-value entry from IndexedDB.
    pub async fn delete_file(key: &str) -> Result<(), JsValue> {
        let s = storage()?;
        let args = js_sys::Array::new();
        args.push(&key.into());
        js_call(&s, "del", &args).await?;
        Ok(())
    }

    /// Subscribe to cross-tab data change notifications via BroadcastChannel.
    /// Returns an unsubscribe closure. The callback receives a JsValue (the changed key).
    pub fn subscribe(cb: &js_sys::Function) -> Result<js_sys::Function, JsValue> {
        let s = storage()?;
        let args = js_sys::Array::new();
        args.push(cb);
        let result = get_fn(&s, "subscribe")?.apply(&s, &args)?;
        result
            .dyn_into::<js_sys::Function>()
            .map_err(|_| JsValue::from_str("expected unsubscribe function from subscribe"))
    }

    /// Return the BroadcastChannel name used for cross-tab sync, or null if unavailable.
    pub fn channel_name() -> Result<Option<String>, JsValue> {
        let s = storage()?;
        let result = get_fn(&s, "getBroadcastChannel")?.call0(&s)?;
        if result.is_null() || result.is_undefined() {
            Ok(None)
        } else {
            result.as_string().map(Some).ok_or_else(|| {
                JsValue::from_str("expected string or null from getBroadcastChannel")
            })
        }
    }
}
