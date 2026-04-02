use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::BTreeMap;
use crate::storage::StorageEngine;

pub struct GcWorker<'a> {
    storage: &'a StorageEngine,
    // Maps expiration timestamp (seconds) to a list of Node IDs
    index_ttl: BTreeMap<u64, Vec<u64>>,
}

impl<'a> GcWorker<'a> {
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self {
            storage,
            index_ttl: BTreeMap::new(),
        }
    }

    /// Registers a node to be automatically expired and cleared at `expiry_secs`
    pub fn register_ttl(&mut self, id: u64, expiry_secs: u64) {
        self.index_ttl.entry(expiry_secs).or_insert_with(Vec::new).push(id);
    }

    /// Triggers a sweep that clears old items. In production this runs in a `tokio::spawn` loop.
    pub fn sweep(&mut self) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Split the BTreeMap, taking all nodes where expiration <= now
        let mut expired_count = 0;
        
        let mut keys_to_remove = Vec::new();
        for (expiry, ids) in self.index_ttl.iter() {
            if *expiry <= now {
                for &id in ids {
                    // Attempt deletion via StorageEngine (Mocked delete here)
                    // Normally: self.storage.delete(id);
                    expired_count += 1;
                }
                keys_to_remove.push(*expiry);
            } else {
                break; // Because it's a BTreeMap, subsequent keys are > now
            }
        }

        for key in keys_to_remove {
            self.index_ttl.remove(&key);
        }

        expired_count
    }
}
