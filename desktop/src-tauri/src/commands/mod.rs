//! Tauri IPC commands that bridge the frontend to the connection layer (DESK-03+).
//!
//! Commands are thin wrappers over the [`crate::connections`] contract:
//! they accept a `tauri::State<'_, AppState>` and never own business logic.

pub mod connection;
pub mod data;