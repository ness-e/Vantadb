// IDB Bridge — IndexedDB persistence for VantaDB WASM builds.
// Auto-imported as ES module via wasm_bindgen(module = "/src/idb_bridge.js").
// Also available as globalThis.vantaIdbStorage for legacy manual-import use.

const DB_NAME = "VantaDB";
const STORE_NAME = "state";
const listeners = [];
let channel = null;

function notify(key) {
  for (let i = 0; i < listeners.length; i++) {
    try { listeners[i](key); } catch (e) { /* ignore */ }
  }
}

function openDB() {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1);
    req.onupgradeneeded = () => req.result.createObjectStore(STORE_NAME);
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

try { channel = new BroadcastChannel("vantadb-sync"); } catch (e) { /* no-op */ }
if (channel) {
  channel.onmessage = (ev) => {
    if (ev.data && ev.data.type === "data-changed") {
      notify(ev.data.key || "db_state.json");
    }
  };
}

const storage = {
  read(key) {
    return openDB().then((db) => new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, "readonly");
      const req = tx.objectStore(STORE_NAME).get(key);
      req.onsuccess = () => resolve(req.result || null);
      req.onerror = () => {
        if (req.error && req.error.name === "NotFoundError") {
          resolve(null);
        } else {
          reject(req.error);
        }
      };
    }));
  },
  write(key, data) {
    return openDB().then((db) => new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, "readwrite");
      tx.objectStore(STORE_NAME).put(data, key);
      tx.oncomplete = () => {
        if (channel) channel.postMessage({ type: "data-changed", key });
        resolve();
      };
      tx.onerror = () => reject(tx.error);
    }));
  },
  del(key) {
    return openDB().then((db) => new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, "readwrite");
      tx.objectStore(STORE_NAME).delete(key);
      tx.oncomplete = () => {
        if (channel) channel.postMessage({ type: "data-changed", key });
        resolve();
      };
      tx.onerror = () => reject(tx.error);
    }));
  },
  subscribe(fn) {
    listeners.push(fn);
    return () => { listeners.splice(listeners.indexOf(fn), 1); };
  },
  getBroadcastChannel() {
    return channel ? "vantadb-sync" : null;
  },
};

// Register on globalThis for both ES-module and legacy IIFE consumers
const g = typeof globalThis !== "undefined" ? globalThis : window;
g.vantaIdbStorage = storage;

// Named exports for wasm_bindgen(module = ...) auto-import
export const read = storage.read.bind(storage);
export const write = storage.write.bind(storage);
export const del = storage.del.bind(storage);
export const subscribe = storage.subscribe.bind(storage);
export const getBroadcastChannel = storage.getBroadcastChannel.bind(storage);
