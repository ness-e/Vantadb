//! Async WAL shipping for read replicas.

/// Replication role for a VantaDB node.
#[derive(Debug, Clone)]
pub enum ReplicationRole {
    /// Primary node that accepts writes and ships WAL segments.
    Primary,
    /// Replica node that receives replicated WAL segments.
    Replica {
        /// Endpoint URL of the primary node.
        primary_endpoint: String,
        /// Synchronisation mode for WAL shipping.
        sync_mode: SyncMode,
    },
}

/// Synchronisation mode for replication.
#[derive(Debug, Clone)]
pub enum SyncMode {
    /// Wait for ACK before acknowledging the write.
    Sync,
    /// Fire-and-forget without waiting for ACK.
    Async,
}

/// Manages WAL shipping and replica health checks.
pub struct ReplicationManager {
    // TODO: WAL shipping, offset tracking, health checks
}

impl ReplicationManager {
    /// Create a new `ReplicationManager` with the given role.
    pub fn new(_role: ReplicationRole) -> Self {
        Self {}
    }

    /// Ship a WAL segment to all configured replicas.
    pub fn ship_wal_segment(&self, _data: &[u8]) -> std::io::Result<()> {
        // TODO: HTTP/gRPC push to replica
        Ok(())
    }

    /// Check the health of all connected replicas.
    pub fn health_check(&self) -> bool {
        // TODO: ping replicas, check lag
        true
    }
}
