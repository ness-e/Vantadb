//! Web Worker bridge for offloading OPFS I/O to a dedicated thread.
//!
//! The worker communicates with the main thread via `postMessage` / `onmessage`.
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

use js_sys::{Array, Reflect};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::opfs::OpfsStorage;

const WORKER_TIMEOUT_MS: u32 = 5000;

/// Requests that can be sent to the worker.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
#[allow(missing_docs)]
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
#[allow(missing_docs)]
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
    Error {
        /// The error message.
        message: String,
    },
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

const MAX_RETRIES: u32 = 2;
const BASE_DELAY_MS: u32 = 1000;

fn is_retryable(err: &JsValue) -> bool {
    err.as_string().is_some_and(|s| s.contains("timeout"))
}

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

    /// Send a request and await the response, with retry on timeout.
    async fn send(&self, req: &WorkerRequest) -> Result<WorkerResponse, JsValue> {
        let mut delay = BASE_DELAY_MS;
        for attempt in 0..=MAX_RETRIES {
            match self.try_send(req).await {
                Ok(resp) => return Ok(resp),
                Err(e) if attempt < MAX_RETRIES && is_retryable(&e) => {
                    let p = Promise::new(&mut {
                        let d = delay;
                        move |r: js_sys::Function, _: js_sys::Function| {
                            let code = format!("setTimeout(() => r(), {});", d);
                            js_sys::Function::new_no_args(&code)
                                .call0(&JsValue::undefined())
                                .ok();
                        }
                    });
                    JsFuture::from(p).await.ok();
                    delay *= 2;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    async fn try_send(&self, req: &WorkerRequest) -> Result<WorkerResponse, JsValue> {
        let msg = serde_wasm_bindgen::to_value(req)
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;

        // Create a MessageChannel for this request/response pair.
        let global = js_sys::global();
        let message_channel = Reflect::get(&global, &"MessageChannel".into())?
            .dyn_into::<js_sys::Function>()
            .map_err(|_| JsValue::from_str("MessageChannel not available"))?;
        let channel = Reflect::construct(&message_channel, &Array::new())
            .map_err(|_| JsValue::from_str("failed to create MessageChannel"))?;
        let port1 = Reflect::get(&channel, &"port1".into())?;
        let port2 = Reflect::get(&channel, &"port2".into())?;

        // Set up the response listener on port1.
        let promise = js_sys::Promise::new(&mut {
            let port1 = port1.clone();
            move |resolve: js_sys::Function, reject: js_sys::Function| {
                Reflect::set(&port1, &"_resolve".into(), &resolve).ok();
                Reflect::set(&port1, &"_reject".into(), &reject).ok();
                let onmessage = js_sys::Function::new_no_args(
                    r#"
                    const port = this;
                    port.onmessage = null;
                    try {
                        port._resolve(arguments[0].data);
                    } catch(e) {
                        port._reject(e);
                    }
                    "#,
                );
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

        // Add a timeout to prevent hanging if the worker never responds.
        let timeout_promise = Promise::new(&mut {
            move |resolve: js_sys::Function, reject: js_sys::Function| {
                let js_code = format!(
                    "setTimeout(function(){{ reject(new Error('Worker response timeout after {}ms')); }}, {});",
                    WORKER_TIMEOUT_MS, WORKER_TIMEOUT_MS
                );
                let wrapper = js_sys::Function::new_with_args("resolve, reject", &js_code);
                wrapper.call2(&JsValue::undefined(), &resolve, &reject).ok();
            }
        });
        let raced = Promise::race(&Array::of2(&promise, &timeout_promise));

        // Await the response.
        let resp_val = JsFuture::from(raced).await?;
        serde_wasm_bindgen::from_value(resp_val)
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))
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
