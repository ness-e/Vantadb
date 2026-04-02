use rocksdb::{Options, DB};
use crate::error::{IadbmsError, Result};
use crate::node::UnifiedNode;
use crate::index::CPIndex;
use std::sync::RwLock;

pub struct StorageEngine {
    db: DB,
    pub hnsw: RwLock<CPIndex>,
}

impl StorageEngine {
    pub fn open(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_background_jobs(4);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        let db = DB::open(&opts, path).map_err(|e| IadbmsError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        
        Ok(Self { 
            db,
            hnsw: RwLock::new(CPIndex::new())
        })
    }

    pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
        // RocksDB Disk Persistence (Durability)
        let key = node.id.to_le_bytes();
        let val = bincode::serialize(node).map_err(|e| IadbmsError::SerializationError(e.to_string()))?;
        self.db.put(&key, &val).map_err(|e| IadbmsError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        // In-Memory Index Tracking (HNSW)
        if node.flags.contains(crate::node::NodeFlags::HAS_VECTOR) {
            if let crate::node::VectorData::F32(vec) = &node.vector {
                let mut index = self.hnsw.write().unwrap();
                index.add(node.id, 0, Some(vec.clone())); // MVP mask 0
            }
        }

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
