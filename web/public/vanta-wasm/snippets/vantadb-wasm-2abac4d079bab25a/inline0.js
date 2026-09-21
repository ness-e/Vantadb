
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
                function doWrite() {
                    const tx = db.transaction(STORE_NAME, "readwrite");
                    tx.objectStore(STORE_NAME).put(data, key);
                    tx.oncomplete = () => { if (channel) channel.postMessage({ type: "data-changed", key }); resolve(); };
                    tx.onerror = () => reject(tx.error);
                }
                if (typeof navigator !== "undefined" && navigator.locks) {
                    navigator.locks.request("vantadb-write", () => new Promise((resolveTx, rejectTx) => {
                        const tx = db.transaction(STORE_NAME, "readwrite");
                        tx.objectStore(STORE_NAME).put(data, key);
                        tx.oncomplete = () => { if (channel) channel.postMessage({ type: "data-changed", key }); resolve(); resolveTx(); };
                        tx.onerror = () => rejectTx(tx.error);
                    })).catch((err) => reject(err));
                } else {
                    doWrite();
                }
            }));
        },
        del(key) {
            return openDB().then((db) => new Promise((resolve, reject) => {
                function doDel() {
                    const tx = db.transaction(STORE_NAME, "readwrite");
                    tx.objectStore(STORE_NAME).delete(key);
                    tx.oncomplete = () => { if (channel) channel.postMessage({ type: "data-changed", key }); resolve(); };
                    tx.onerror = () => reject(tx.error);
                }
                if (typeof navigator !== "undefined" && navigator.locks) {
                    navigator.locks.request("vantadb-write", () => new Promise((resolveTx, rejectTx) => {
                        const tx = db.transaction(STORE_NAME, "readwrite");
                        tx.objectStore(STORE_NAME).delete(key);
                        tx.oncomplete = () => { if (channel) channel.postMessage({ type: "data-changed", key }); resolve(); resolveTx(); };
                        tx.onerror = () => rejectTx(tx.error);
                    })).catch((err) => reject(err));
                } else {
                    doDel();
                }
            }));
        },
        subscribe(fn) { listeners.push(fn); return () => { listeners.splice(listeners.indexOf(fn), 1); }; },
        getBroadcastChannel() { return channel ? "vantadb-sync" : null; },
    };
    const g = typeof globalThis !== "undefined" ? globalThis : window;
    g.vantaIdbStorage = storage;
})();
