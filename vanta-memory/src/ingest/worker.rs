//! Ingest worker (MEM-30): orchestrates one full wiki build against the core
//! [`vantadb::wiki::WikiStore`] state machine:
//!
//! `begin_processing` → scan/chunk (`scan_local_sources` + `chunk_text`) →
//! extract candidates (optional LLM) → serial merge ([`crate::ingest::merge::commit`])
//! → `put_page` per written page → `complete` / `fail`.
//!
//! The worker never touches core internals: it drives the public SDK surface
//! of `vantadb::wiki` and degrades LLM-free (P4) when no runner is provided.

use std::path::Path;

use crate::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use crate::ingest::{
    callback::{IngestPhase, IngestProgress, ProgressTracker},
    merge::{aggregate_by_rel_path, commit, parse_file_blocks, CandidatePage},
    prompts, IngestConfig, IngestError,
};

/// Result of one worker run.
#[derive(Debug, Clone, PartialEq)]
pub struct IngestReport {
    pub run_id: String,
    /// Sources that produced candidates.
    pub sources_processed: Vec<String>,
    /// Sources skipped (e.g. no extraction available LLM-free).
    pub sources_skipped: Vec<String>,
    /// Serial merge report (written pages + non-fatal skips/errors).
    pub commit_report: crate::ingest::merge::CommitReport,
}

/// Run a full ingest build for `namespace:slug` over local markdown under
/// `root`. The wiki must already exist (`create`). A busy wiki is rejected by
/// the store's own guards; a finished one gets `request_ingest` first.
pub fn run<R: LlmRunner>(
    store: &vantadb::wiki::WikiStore<'_>,
    namespace: &str,
    slug: &str,
    root: &Path,
    runner: Option<&R>,
    config: &IngestConfig,
) -> Result<IngestReport, IngestError> {
    run_with_progress(store, namespace, slug, root, runner, config, None)
}

/// [`run`] with live progress reporting (MEM-31): snapshots are pushed to
/// `progress` during extracting|merging|indexing. The channel is best-effort
/// P4 — it never blocks and never fails the build.
pub fn run_with_progress<R: LlmRunner>(
    store: &vantadb::wiki::WikiStore<'_>,
    namespace: &str,
    slug: &str,
    root: &Path,
    runner: Option<&R>,
    config: &IngestConfig,
    progress: Option<&ProgressTracker>,
) -> Result<IngestReport, IngestError> {
    let current = store
        .get(namespace, slug)?
        .ok_or_else(|| IngestError::NotFound {
            namespace: namespace.to_string(),
            slug: slug.to_string(),
        })?;
    if !current.state.is_busy() {
        store.request_ingest(namespace, slug)?;
    }
    let wiki = store.begin_processing(namespace, slug)?;
    let run_id = match &wiki.run_id {
        Some(id) => id.clone(),
        None => {
            return Err(IngestError::Invalid(
                "begin_processing returned no run_id".into(),
            ));
        }
    };
    // MEM-31: from here on the tracker only accepts this run_id — packets
    // from an older build are discarded (late-packet guard).
    if let Some(t) = progress {
        t.begin_run(&run_id);
    }

    match ingest_body(
        store, namespace, slug, root, runner, config, progress, &run_id,
    ) {
        Ok(mut report) => {
            emit(
                progress,
                IngestProgress::new(
                    &run_id,
                    IngestPhase::Indexing,
                    report.commit_report.written.len(),
                    report.commit_report.written.len(),
                    0,
                    0,
                ),
            );
            match store.complete(namespace, slug, &run_id) {
                Ok(_) => {
                    emit(
                        progress,
                        IngestProgress::new(&run_id, IngestPhase::Done, 1, 1, 0, 0),
                    );
                    report.run_id = run_id;
                    Ok(report)
                }
                Err(e) => {
                    let _ = store.fail(namespace, slug, &run_id, &e.to_string());
                    emit(
                        progress,
                        IngestProgress::new(&run_id, IngestPhase::Failed, 0, 0, 1, 0),
                    );
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            // Best-effort failure marking; surface the original error either way.
            let _ = store.fail(namespace, slug, &run_id, &e.to_string());
            emit(
                progress,
                IngestProgress::new(&run_id, IngestPhase::Failed, 0, 0, 1, 0),
            );
            Err(e)
        }
    }
}

/// Best-effort progress emission (P4): `update_progress` uses try_lock, so
/// this can never block or fail the ingest.
fn emit(progress: Option<&ProgressTracker>, snapshot: IngestProgress) {
    if let Some(t) = progress {
        let _ = t.update_progress(snapshot);
    }
}

/// Scan → chunk → extract → serial merge → persist. Storage-free except the
/// final write pass, which binds to `store`. `progress` (MEM-31) receives
/// Extracting snapshots per source and Merging snapshots per page write.
#[allow(clippy::too_many_arguments)]
fn ingest_body<R: LlmRunner>(
    store: &vantadb::wiki::WikiStore<'_>,
    namespace: &str,
    slug: &str,
    root: &Path,
    runner: Option<&R>,
    config: &IngestConfig,
    progress: Option<&ProgressTracker>,
    run_id: &str,
) -> Result<IngestReport, IngestError> {
    let files = vantadb::wiki::scan_local_sources(root)?;
    emit(
        progress,
        IngestProgress::new(run_id, IngestPhase::Extracting, files.len(), 0, 0, 0),
    );

    let mut sources_processed = Vec::new();
    let mut sources_skipped = Vec::new();
    let mut extracted: Vec<(String, Vec<CandidatePage>)> = Vec::new();

    for file in files {
        let chunks = vantadb::wiki::chunk_text(
            &file.content,
            config.chunk_target_chars,
            config.chunk_overlap_chars,
        );
        if chunks.is_empty() {
            continue;
        }
        let mut candidates: Vec<CandidatePage> = Vec::new();
        let mut degraded = false;
        for chunk in &chunks {
            match extract_chunk(&file.rel_path, chunk, runner) {
                Ok(pages) => candidates.extend(pages),
                Err(LlmError::NotConfigured) => {
                    // P4: LLM-free mode — source recorded as skipped, never a hard error.
                    degraded = true;
                    break;
                }
                Err(e) => {
                    tracing::warn!(source = %file.rel_path, error = %e, "wiki ingest: extraction failed");
                    degraded = true;
                    break;
                }
            }
        }
        if candidates.is_empty() {
            sources_skipped.push(file.rel_path.clone());
            if !degraded {
                continue;
            }
        } else {
            sources_processed.push(file.rel_path.clone());
            extracted.push((file.rel_path.clone(), candidates));
        }
        // Per-source snapshot; the tracker's throttle collapses these to
        // ~2/sec while preserving phase changes.
        emit(
            progress,
            IngestProgress::new(
                run_id,
                IngestPhase::Extracting,
                sources_processed.len() + sources_skipped.len(),
                sources_processed.len(),
                0,
                sources_skipped.len(),
            ),
        );
    }

    let by_page = aggregate_by_rel_path(extracted);
    let merge_total = by_page.len();
    emit(
        progress,
        IngestProgress::new(run_id, IngestPhase::Merging, merge_total, 0, 0, 0),
    );

    // Store-backed page IO: relPath (`wiki/<dir>/<file>.md`) maps to
    // (page_type = dir, title = stem); canonicalization/dedup stay in WikiStore.
    // Merged/written/failed counters feed per-page Merging progress.
    let merged = std::cell::Cell::new(0usize);
    let write_failed = std::cell::Cell::new(0usize);
    let commit_report = commit(
        by_page,
        runner,
        config,
        |rel| match split_rel_path(rel) {
            Some((page_type, title)) => {
                let path = vantadb::wiki::canonical_path(&page_type, &title);
                Ok(store
                    .get_page(namespace, slug, &path)?
                    .map(|page| page.content))
            }
            None => Ok(None),
        },
        |rel, content| {
            let Some((page_type, title)) = split_rel_path(rel) else {
                return Err(IngestError::Invalid(format!(
                    "candidate path `{rel}` is not storable"
                )));
            };
            match store.put_page(namespace, slug, &page_type, &title, content) {
                Ok(_) => {
                    merged.set(merged.get() + 1);
                    emit(
                        progress,
                        IngestProgress::new(
                            run_id,
                            IngestPhase::Merging,
                            merge_total,
                            merged.get(),
                            write_failed.get(),
                            0,
                        ),
                    );
                    Ok(())
                }
                Err(e) => {
                    write_failed.set(write_failed.get() + 1);
                    emit(
                        progress,
                        IngestProgress::new(
                            run_id,
                            IngestPhase::Merging,
                            merge_total,
                            merged.get(),
                            write_failed.get(),
                            0,
                        ),
                    );
                    Err(IngestError::from(e))
                }
            }
        },
    )?;

    Ok(IngestReport {
        run_id: String::new(),
        sources_processed,
        sources_skipped,
        commit_report,
    })
}

/// Extract candidate pages for one source chunk via the optional LLM runner.
fn extract_chunk<R: LlmRunner>(
    source_name: &str,
    chunk: &str,
    runner: Option<&R>,
) -> Result<Vec<CandidatePage>, LlmError> {
    let Some(runner) = runner else {
        return Err(LlmError::NotConfigured);
    };
    let mut params = LlmRunParams::new(String::new(), "ingest-extract");
    params.system_prompt = Some(prompts::extraction_system_prompt(
        "Capture the durable knowledge of these documents as an interconnected wiki.",
    ));
    params.prompt = prompts::extraction_user_prompt(source_name, chunk, &[]);
    let output = runner.run(&params)?;
    Ok(parse_file_blocks(&output))
}

/// Map a candidate relPath (`wiki/<dir>/<file>.md`) to its storage components:
/// page_type = dir component, title = filename stem.
pub(crate) fn split_rel_path(rel: &str) -> Option<(String, String)> {
    let rest = rel.strip_prefix("wiki/")?;
    let (dir, file) = rest.split_once('/')?;
    let title = file.strip_suffix(".md").unwrap_or(file);
    if dir.is_empty() || title.is_empty() || file.contains('/') {
        return None;
    }
    Some((dir.to_string(), title.to_string()))
}
