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

/**
 * Register auto-save handlers for a VantaDB instance.
 *
 * Attaches listeners for `visibilitychange` (with debounce) and `pagehide`
 * events that call `db.try_auto_save()` when the document becomes hidden or
 * is about to unload.
 *
 * @param {Object} db - VantaDB instance (returned from connect_persistent/connect_idb/connect_worker)
 * @param {Object} [options] - Configuration options
 * @param {number} [options.debounceMs=2000] - Debounce time for visibilitychange in milliseconds
 * @returns {Function} Unregister function to clean up event listeners
 *
 * @example
 * import { registerAutoSave } from "vantadb-wasm/src/opfs_bridge.js";
 * const db = await VantaDB.connect_persistent("my-db");
 * db.enable_auto_save();
 * const unregister = registerAutoSave(db);
 * // Later, to stop auto-save:
 * unregister();
 */
export function registerAutoSave(db, options = {}) {
  const debounceMs = options.debounceMs ?? 2000;
  let debounceTimer = null;
  let isSaving = false;

  const attemptSave = async () => {
    if (isSaving) return;
    isSaving = true;
    try {
      // Call the WASM try_auto_save method
      await db.try_auto_save();
    } catch (e) {
      console.warn('[vantadb] auto-save failed:', e);
    } finally {
      isSaving = false;
    }
  };

  const handleVisibilityChange = () => {
    if (document.visibilityState === 'hidden') {
      // Debounce: wait for debounceMs before attempting save
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }
      debounceTimer = setTimeout(attemptSave, debounceMs);
    }
  };

  const handlePageHide = async () => {
    // On pagehide, we can't rely on setTimeout completing.
    // Use navigator.sendBeacon or fetch with keepalive as a best-effort.
    if (isSaving) return;
    isSaving = true;

    // Try to call try_auto_save synchronously first (may complete if fast)
    const savePromise = db.try_auto_save();

    // Use sendBeacon as fallback for the actual persist data if needed
    // Note: try_auto_save handles the actual save internally, so we just
    // need to ensure the promise has a chance to run.
    // We await with a short timeout to give it a chance.
    const timeoutPromise = new Promise(resolve => setTimeout(resolve, 100));
    await Promise.race([savePromise, timeoutPromise]);

    // If save didn't complete, we can't do much more in pagehide
    // The differential persist cache will retry on next load
  };

  document.addEventListener('visibilitychange', handleVisibilityChange);
  window.addEventListener('pagehide', handlePageHide);

  // Return unregister function
  return () => {
    document.removeEventListener('visibilitychange', handleVisibilityChange);
    window.removeEventListener('pagehide', handlePageHide);
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
  };
}

/**
 * Unregister auto-save handlers.
 *
 * @param {Function} unregister - The function returned by registerAutoSave
 */
export function unregisterAutoSave(unregister) {
  if (typeof unregister === 'function') {
    unregister();
  }
}

// Side-effect: auto-register the worker helper on `globalThis` so consumers
// using `connect_worker(path)` do not need to manually inject it. This mirrors
// the DuckDB-WASM glue pattern (`@duckdb/duckdb-wasm` registers its worker
// creator as a module side effect). Guarded to browser contexts to avoid
// polluting Node / jsdom test environments.
if (typeof window !== 'undefined' && typeof globalThis !== 'undefined') {
  globalThis.spawnOpfsWorker = spawnOpfsWorker;
}
