// OPFS Bridge — JS helper functions callable from Rust via wasm-bindgen.
//
// These functions are exposed as top-level exports so Rust code can import
// them through `#[wasm_bindgen(module = "/src/opfs_bridge.js")]`.

/**
 * Open or create a file inside an OPFS directory handle.
 * @param {FileSystemDirectoryHandle} dirHandle
 * @param {string} path
 * @param {boolean} create
 * @returns {Promise<FileSystemFileHandle|null>}
 */
export async function openFile(dirHandle, path, create) {
  try {
    return await dirHandle.getFileHandle(path, { create });
  } catch (e) {
    if (!create && e.name === 'NotFoundError') {
      return null;
    }
    throw e;
  }
}

/**
 * Read the full contents of a FileSystemFileHandle as a Uint8Array.
 * @param {FileSystemFileHandle} handle
 * @returns {Promise<Uint8Array>}
 */
export async function readFile(handle) {
  const file = await handle.getFile();
  const buffer = await file.arrayBuffer();
  return new Uint8Array(buffer);
}

/**
 * Write data to a FileSystemFileHandle (replaces contents).
 * @param {FileSystemFileHandle} handle
 * @param {Uint8Array} data
 * @returns {Promise<void>}
 */
export async function writeFile(handle, data) {
  const writable = await handle.createWritable();
  await writable.write(data);
  await writable.close();
}

/**
 * Append data to a FileSystemFileHandle.
 * @param {FileSystemFileHandle} handle
 * @param {Uint8Array} data
 * @returns {Promise<void>}
 */
export async function appendFile(handle, data) {
  const writable = await handle.createWritable({ keepExistingData: true });
  await writable.write({ type: 'write', data, position: (await handle.getFile()).size });
  await writable.close();
}

/**
 * Create a dedicated Web Worker from a dynamically built blob URL.
 * The worker script includes a minimal onmessage handler that posts
 * back the received data (the Rust WASM worker module takes over
 * once initialised).
 * @returns {Worker}
 */
export function spawnOpfsWorker() {
  const blob = new Blob(
    [
      `self.onmessage = function (e) {
        // Forward messages to the WASM module once it registers its handler.
        if (self.__opfsWorkerHandler) {
          self.__opfsWorkerHandler(e);
        } else {
          self.postMessage({ type: 'error', payload: { message: 'worker handler not registered' } });
        }
      };
      self.__opfsWorkerHandler = null;
      self.__registerOpfsHandler = function (handler) {
        self.__opfsWorkerHandler = handler;
      };`,
    ],
    { type: 'application/javascript' },
  );
  return new Worker(URL.createObjectURL(blob));
}
