use rocksdb::{Options, DB};
use crate::error::{IadbmsError, Result};
use crate::node::UnifiedNode;

pub struct StorageEngine {
    db: DB,
}

impl StorageEngine {
    pub fn open(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_background_jobs(4);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        // Disable WAL inside rocksdb, we have our own WAL (or we combine them).
        // For zero-copy, we use pinned gets.
        let db = DB::open(&opts, path).map_err(|e| IadbmsError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        Ok(Self { db })
    }

    pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
        let key = node.id.to_le_bytes();
        let val = bincode::serialize(node).map_err(|e| IadbmsError::SerializationError(e.to_string()))?;
        self.db.put(&key, &val).map_err(|e| IadbmsError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        Ok(())
    }

    pub fn get(&self, id: u64) -> Result<Option<UnifiedNode>> {
        let key = id.to_le_bytes();
        match self.db.get_pinned(&key) {
            Ok(Some(slice)) => {
                // Zero-copy deserialization reference via bincode if using borrowed struct,
                // otherwise it dynamically allocates the struct content but copies pinned data directly.
                let node: UnifiedNode = bincode::deserialize(&slice).map_err(|e| IadbmsError::SerializationError(e.to_string()))?;
                Ok(Some(node))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(IadbmsError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))),
        }
    }

    pub fn delete(&self, id: u64) -> Result<()> {
        let key = id.to_le_bytes();
        self.db.delete(&key)
            .map_err(|e| IadbmsError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        Ok(())
    }
}
