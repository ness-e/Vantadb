//! L2 scene contracts + LLM-free scene index (MEM-12) + sandboxed scene
//! tools (MEM-13, F4).
//!
//! The scene is the L2 memory unit: the block format ([`scene_format`]) and
//! the per-session index ([`scene_index`]) that anchors scene navigation
//! without an LLM call (Principio 4). The core graph node
//! (`vantadb::entity::scene::SceneNodeStore`) is the persisted anchor; the
//! index here is the SDK-side companion used by the L2 strategy (MEM-14).
//! [`scene_tools`] is the sandboxed tool layer (read/write/edit) the L2
//! strategy drives the store through (MEM-13).

pub mod scene_format;
pub mod scene_index;
pub mod scene_tools;

pub use scene_format::SceneBlock;
pub use scene_index::{
    current_scene, get_scene, list_scenes, scene_namespace, upsert_scene, SceneError,
};
pub use scene_tools::{
    edit_scene_tool, execute_scene_tool, read_scene_tool, write_scene_tool, SceneToolCall,
    SceneToolError, SceneToolResult,
};
