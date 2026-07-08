//! Web Worker bridge for offloading OPFS I/O to a dedicated thread.
//!
//! The worker communicates with the main thread via `postMessage` / `onmessage`.
//! Each message is a JSON-encoded `WorkerRequest`, and the worker replies
//! with a `WorkerResponse` containing the result.
//!
//! # Message Protocol
//!
//! ```ignore
//! Main thread                    Worker
//!     │                           │
//!     ├── Read { path } ──────────┤
//!     │                           ├── open file, read, return data
//!     │── Write { path, data } ───┤
//!     │                           ├── open (create), write, confirm
//!     │── Delete { path } ────────┤
//!     │                           ├── remove entry or skip if absent
//!     │── Init { name } ──────────┤
//!     │                           ├── open storage directory
//! ```

use js_sys::{Array, Promise, Reflect};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::opfs::OpfsStorage;

/// Requests that can be sent to the worker.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum WorkerRequest {
    /// Initialize the worker with a storage directory name.
    Init { name: String },
    /// Read a file and return its contents.
    Read { path: String },
    /// Write data to a file (creates or overwrites).
    Write { path: String, data: Vec<u8> },
    /// Append data to the end of a file.
    Append { path: String, data: Vec<u8> },
    /// Delete a file.
    Delete { path: String },
}

/// Responses sent back from the worker.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum WorkerResponse {
    /// Worker has been initialized successfully.
    Initialized,
    /// Read completed — contains the file bytes, or None if file not found.
    ReadResult { data: Option<Vec<u8>> },
    /// Write completed successfully.
    Written,
    /// Append completed successfully.
    Appended,
    /// Delete completed successfully.
    Deleted,
    /// An error occurred.
    Error { message: String },
}

/// The worker-side state holding the OPFS storage handle.
///
/// This struct lives inside the dedicated Worker global scope and is
/// not accessible from the main thread.
pub struct OpfsWorker {
    storage: Option<OpfsStorage>,
}

impl OpfsWorker {
    /// Create a new uninitialised worker.
    pub fn new() -> Self {
        Self { storage: None }
    }

    /// Handle an incoming request and return a response.
    pub async fn handle(&mut self, req: WorkerRequest) -> WorkerResponse {
        match req {
            WorkerRequest::Init { name } => match OpfsStorage::open(&name).await {
                Ok(storage) => {
                    self.storage = Some(storage);
                    WorkerResponse::Initialized
                }
                Err(e) => {
                    let msg = js_sys::Error::from(e)
                        .to_string()
                        .as_string()
                        .unwrap_or_else(|| "unknown init error".into());
                    WorkerResponse::Error { message: msg }
                }
            },
            WorkerRequest::Read { path } => {
                let storage = match &self.storage {
                    Some(s) => s,
                    None => {
                        return WorkerResponse::Error {
                            message: "worker not initialized".into(),
                        }
                    }
                };
                match storage.read_file(&path).await {
                    Ok(data) => WorkerResponse::ReadResult { data },
                    Err(e) => WorkerResponse::Error {
                        message: format!("read error: {:?}", e),
                    },
                }
            }
            WorkerRequest::Write { path, data } => {
                let storage = match &self.storage {
                    Some(s) => s,
                    None => {
                        return WorkerResponse::Error {
                            message: "worker not initialized".into(),
                        }
                    }
                };
                match storage.write_file(&path, &data).await {
                    Ok(()) => WorkerResponse::Written,
                    Err(e) => WorkerResponse::Error {
                        message: format!("write error: {:?}", e),
                    },
                }
            }
            WorkerRequest::Append { path, data } => {
                let storage = match &self.storage {
                    Some(s) => s,
                    None => {
                        return WorkerResponse::Error {
                            message: "worker not initialized".into(),
                        }
                    }
                };
                match storage.append_file(&path, &data).await {
                    Ok(()) => WorkerResponse::Appended,
                    Err(e) => WorkerResponse::Error {
                        message: format!("append error: {:?}", e),
                    },
                }
            }
            WorkerRequest::Delete { path } => {
                let storage = match &self.storage {
                    Some(s) => s,
                    None => {
                        return WorkerResponse::Error {
                            message: "worker not initialized".into(),
                        }
                    }
                };
                match storage.delete_file(&path).await {
                    Ok(()) => WorkerResponse::Deleted,
                    Err(e) => WorkerResponse::Error {
                        message: format!("delete error: {:?}", e),
                    },
                }
            }
        }
    }
}

// ─── Main-thread proxy ──────────────────────────────────────────────

/// A proxy that communicates with an OPFS Web Worker.
///
/// Sends `WorkerRequest` messages and awaits `WorkerResponse` replies.
/// The worker is created by the JS-side `opfs_bridge.js` module; this
/// struct holds a reference to the `MessagePort`-like handle.
pub struct OpfsWorkerProxy {
    worker: JsValue,
}

impl OpfsWorkerProxy {
    /// Create a new proxy wrapping an existing worker JS object.
    ///
    /// The worker must already have a `postMessage` / `onmessage` channel.
    pub fn new(worker: JsValue) -> Self {
        Self { worker }
    }

    /// Send a request and await the response.
    async fn send(&self, req: &WorkerRequest) -> Result<WorkerResponse, JsValue> {
        let msg = serde_json::to_value(req).map_err(|e| js_sys::Error::new(&e.to_string()))?;

        // Create a MessageChannel for this request/response pair.
        let global = js_sys::global();
        let message_channel = Reflect::get(&global, &"MessageChannel".into())?
            .dyn_into::<js_sys::Function>()
            .map_err(|_| JsValue::from_str("MessageChannel not available"))?;
        let channel = message_channel
            .new(&Array::new())
            .map_err(|_| JsValue::from_str("failed to create MessageChannel"))?;
        let port1 = Reflect::get(&channel, &"port1".into())?;
        let port2 = Reflect::get(&channel, &"port2".into())?;

        // Set up the response listener on port1.
        let promise = js_sys::Promise::new(&mut {
            let port1 = port1.clone();
            move |resolve: js_sys::Function, _reject: js_sys::Function| {
                let onmessage = js_sys::Function::new_no_args(&format!(
                    r#"
                        const msg = arguments[0];
                        this.onmessage = null;
                        arguments[1](msg.data);
                        "#,
                ));
                Reflect::set(&port1, &"onmessage".into(), &onmessage).ok();
            }
        });

        // Send the message through the worker, transferring port2.
        let transfer = Array::new();
        transfer.push(&port2);
        let post_args = Array::new();
        post_args.push(&msg);
        post_args.push(&transfer);
        let post_fn = get_fn(&self.worker, "postMessage")?;
        post_fn.apply(&self.worker, &post_args)?;

        // Await the response.
        let resp_val = JsFuture::from(promise).await?;
        let resp_str = resp_val
            .as_string()
            .ok_or_else(|| JsValue::from_str("expected string response"))?;
        serde_json::from_str(&resp_str).map_err(|e| js_sys::Error::new(&e.to_string()).into())
    }

    /// Initialise the worker with a storage directory name.
    pub async fn init(&self, name: &str) -> Result<(), JsValue> {
        let req = WorkerRequest::Init {
            name: name.to_string(),
        };
        match self.send(&req).await? {
            WorkerResponse::Initialized => Ok(()),
            WorkerResponse::Error { message } => Err(js_sys::Error::new(&message).into()),
            _ => Err(js_sys::Error::new("unexpected worker response").into()),
        }
    }

    /// Read a file via the worker.
    pub async fn read(&self, path: &str) -> Result<Option<Vec<u8>>, JsValue> {
        let req = WorkerRequest::Read {
            path: path.to_string(),
        };
        match self.send(&req).await? {
            WorkerResponse::ReadResult { data } => Ok(data),
            WorkerResponse::Error { message } => Err(js_sys::Error::new(&message).into()),
            _ => Err(js_sys::Error::new("unexpected worker response").into()),
        }
    }

    /// Write a file via the worker.
    pub async fn write(&self, path: &str, data: &[u8]) -> Result<(), JsValue> {
        let req = WorkerRequest::Write {
            path: path.to_string(),
            data: data.to_vec(),
        };
        match self.send(&req).await? {
            WorkerResponse::Written => Ok(()),
            WorkerResponse::Error { message } => Err(js_sys::Error::new(&message).into()),
            _ => Err(js_sys::Error::new("unexpected worker response").into()),
        }
    }

    /// Append to a file via the worker.
    pub async fn append(&self, path: &str, data: &[u8]) -> Result<(), JsValue> {
        let req = WorkerRequest::Append {
            path: path.to_string(),
            data: data.to_vec(),
        };
        match self.send(&req).await? {
            WorkerResponse::Appended => Ok(()),
            WorkerResponse::Error { message } => Err(js_sys::Error::new(&message).into()),
            _ => Err(js_sys::Error::new("unexpected worker response").into()),
        }
    }

    /// Delete a file via the worker.
    pub async fn delete(&self, path: &str) -> Result<(), JsValue> {
        let req = WorkerRequest::Delete {
            path: path.to_string(),
        };
        match self.send(&req).await? {
            WorkerResponse::Deleted => Ok(()),
            WorkerResponse::Error { message } => Err(js_sys::Error::new(&message).into()),
            _ => Err(js_sys::Error::new("unexpected worker response").into()),
        }
    }
}

fn get_fn(obj: &JsValue, method: &str) -> Result<js_sys::Function, JsValue> {
    let val = Reflect::get(obj, &method.into())?;
    val.dyn_into::<js_sys::Function>()
}
