/// Testing utilities for VantaDB.
///
/// This module provides reusable test harnesses and utilities
/// that are only available when the `failpoints` feature is enabled.
#[cfg(feature = "failpoints")]
pub mod chaos;
