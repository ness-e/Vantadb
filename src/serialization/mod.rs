//! Reserved namespace — intentionally empty.
//!
//! FIND-608 (SE-01): the previous doc promised "zero-copy and portable
//! archive formats" that were never implemented here. The actual node and
//! index serialization lives in `vantadb::sdk` (`sdk::serialization`,
//! memory-record payloads) and in the storage layer (`storage::ops`).
//! This module is kept only as a stable public-path placeholder — it is
//! part of the published API surface (`tests/api/public-api.txt`) and must
//! not acquire items without an API review.
