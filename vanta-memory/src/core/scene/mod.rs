//! L2 scene contracts + LLM-free scene index (MEM-12) + sandboxed scene
//! tools (MEM-13) + strategy extractor (MEM-14, F4).
//!
//! The scene is the L2 memory unit: the block format ([`scene_format`]) and
//! the per-session index ([`scene_index`]) that anchors scene navigation
//! without an LLM call (Principio 4). The core graph node
//! (`vantadb::entity::scene::SceneNodeStore`) is the persisted anchor; the
//! index here is the SDK-side companion used by the L2 strategy (MEM-14).
//! [`scene_tools`] is the sandboxed tool layer (read/write/edit) the L2
//! strategy drives the store through (MEM-13), and [`scene_extractor`] is the
//! UPDATE>MERGE>CREATE strategy with heat + soft-delete (MEM-14).
//! [`filename_normalizer`] canonicalizes LLM-emitted scene names before they
//! hit the store (MEM-14).

pub mod filename_normalizer;
pub mod scene_extractor;
pub mod scene_format;
pub mod scene_index;
pub mod scene_navigation;
pub mod scene_tools;

pub use filename_normalizer::{is_normalized_scene_name, normalize_scene_name};
pub use scene_extractor::{
    apply_strategy, decide_strategy, extract_scenes, extract_scenes_with_llm, SceneAction,
    SceneApplyResult, SceneExtraction, SceneExtractionResult, SceneExtractorError,
    SceneMemoryInput, SceneStrategy,
};
pub use scene_format::SceneBlock;
pub use scene_index::{
    current_scene, get_scene, list_scenes, scene_namespace, soft_delete_scene, upsert_scene,
    write_scene_block, SceneError,
};
pub use scene_navigation::{
    generate_scene_navigation, heat_emoji, strip_scene_navigation, NAV_HEADER,
};
pub use scene_tools::{
    edit_scene_tool, execute_scene_tool, read_scene_tool, write_scene_tool, SceneToolCall,
    SceneToolError, SceneToolResult,
};
