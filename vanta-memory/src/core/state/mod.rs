//! Pipeline state contracts shared by managers, backends and the worker
//! (MEM-16).

mod types;

pub use types::{
    CaptureAtomicParams, CaptureAtomicResult, PipelineSessionState, TaskKind, TaskPayload,
    TimerEntry,
};
