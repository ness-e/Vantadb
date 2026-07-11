;(function () {
  if (typeof window === "undefined" || window.vantaIdbStorage) return;

  var DB_NAME = "VantaDB";
  var STORE_NAME = "state";
  var listeners = [];
  var channel = null;

  function notify(key) {
    for (var i = 0; i < listeners.length; i++) {
      try { listeners[i](key); } catch (e) { /* ignore */ }
    }
  }

  function openDB() {
    return new Promise(function (resolve, reject) {
      var req = indexedDB.open(DB_NAME, 1);
      req.onupgradeneeded = function () {
        req.result.createObjectStore(STORE_NAME);
      };
      req.onsuccess = function () { resolve(req.result); };
      req.onerror = function () { reject(req.error); };
    });
  }

  try { channel = new BroadcastChannel("vantadb-sync"); } catch (e) { /* no-op */ }
  if (channel) {
    channel.onmessage = function (ev) {
      if (ev.data && ev.data.type === "data-changed") {
        notify(ev.data.key || "db_state.json");
      }
    };
  }

  window.vantaIdbStorage = {
    read: function (key) {
      return openDB().then(function (db) {
        return new Promise(function (resolve, reject) {
          var tx = db.transaction(STORE_NAME, "readonly");
          var req = tx.objectStore(STORE_NAME).get(key);
          req.onsuccess = function () { resolve(req.result || null); };
          req.onerror = function () {
            if (req.error && req.error.name === "NotFoundError") {
              resolve(null);
            } else {
              reject(req.error);
            }
          };
        });
      });
    },
    write: function (key, data) {
      return openDB().then(function (db) {
        return new Promise(function (resolve, reject) {
          var tx = db.transaction(STORE_NAME, "readwrite");
          tx.objectStore(STORE_NAME).put(data, key);
          tx.oncomplete = function () {
            if (channel) {
              channel.postMessage({ type: "data-changed", key: key });
            }
            resolve();
          };
          tx.onerror = function () { reject(tx.error); };
        });
      });
    },
    del: function (key) {
      return openDB().then(function (db) {
        return new Promise(function (resolve, reject) {
          var tx = db.transaction(STORE_NAME, "readwrite");
          tx.objectStore(STORE_NAME).delete(key);
          tx.oncomplete = function () {
            if (channel) {
              channel.postMessage({ type: "data-changed", key: key });
            }
            resolve();
          };
          tx.onerror = function () { reject(tx.error); };
        });
      });
    },
    subscribe: function (fn) {
      listeners.push(fn);
      return function () {
        listeners = listeners.filter(function (f) { return f !== fn; });
      };
    },
    getBroadcastChannel: function () {
      return channel ? "vantadb-sync" : null;
    },
  };
})();
