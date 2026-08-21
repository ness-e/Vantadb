//! Pipeline factory (port of TDAM `MC/utils/pipeline-factory.ts`, minimal —
//! MEM-16).
//!
//! Builds the local orchestration trio (backend + manager + worker) from one
//! config with an injected clock. TDAM's 1200-line factory selects remote
//! backends, Redis hash-tag layouts and multi-instance routing — none of that
//! exists here (Principio 7); it reappears only when a second backend does.

use crate::services::pipeline_worker::PipelineWorker;
use crate::utils::local_backend::LocalStateBackend;
use crate::utils::managed_timer::{Clock, SystemClock};
use crate::utils::pipeline_manager::{MemoryPipelineManager, PipelineConfig};

/// Assembled pipeline components sharing one backend.
pub struct PipelineComponents<'a, C: Clock> {
    /// Shared state backend.
    pub backend: &'a LocalStateBackend<C>,
    /// In-process capture manager.
    pub manager: MemoryPipelineManager<'a, C>,
    /// Task worker consuming the shared queue.
    pub worker: PipelineWorker<'a, C>,
}

/// Factory entry point.
pub struct PipelineFactory;

impl PipelineFactory {
    /// Build the shared backend with the given clock. Managers and workers
    /// borrow-assemble over it via [`PipelineComponents::assemble`] (which
    /// takes the [`PipelineConfig`]).
    pub fn build<C: Clock>(clock: C) -> LocalStateBackend<C> {
        LocalStateBackend::new(clock)
    }

    /// Real-clock convenience constructor.
    pub fn build_system() -> LocalStateBackend<SystemClock> {
        Self::build(SystemClock)
    }
}

impl<'a, C: Clock> PipelineComponents<'a, C> {
    /// Borrow-assemble manager + worker over an existing backend.
    pub fn assemble(backend: &'a LocalStateBackend<C>, config: PipelineConfig) -> Self {
        Self {
            backend,
            manager: MemoryPipelineManager::new(backend, config.clone()),
            worker: PipelineWorker::new(
                backend,
                crate::services::pipeline_worker::WorkerConfig::default(),
            ),
        }
    }
}
