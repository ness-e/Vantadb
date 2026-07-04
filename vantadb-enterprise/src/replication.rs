//! Async WAL shipping for read replicas.

/// Replication role for a VantaDB node.
#[derive(Debug, Clone)]
pub enum ReplicationRole {
    Primary,
    Replica {
        primary_endpoint: String,
        sync_mode: SyncMode,
    },
}

#[derive(Debug, Clone)]
pub enum SyncMode {
    Sync,  // Wait for ACK before acknowledging write
    Async, // Fire-and-forget
}

pub struct ReplicationManager {
    // TODO: WAL shipping, offset tracking, health checks
}

impl ReplicationManager {
    pub fn new(_role: ReplicationRole) -> Self {
        Self {}
    }

    pub fn ship_wal_segment(&self, _data: &[u8]) -> std::io::Result<()> {
        // TODO: HTTP/gRPC push to replica
        Ok(())
    }

    pub fn health_check(&self) -> bool {
        // TODO: ping replicas, check lag
        true
    }
}
