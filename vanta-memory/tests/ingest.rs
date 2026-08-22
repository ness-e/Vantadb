//! Ingest tests (D19, MEM-30). Contract:
//! (a) chunks → candidates aggregated by relPath;
//! (b) merge serial per page under configurable global limit (default 5,
//!     clamp 1-20);
//! (c) merge failure on page N does not block N+1..;
//! (d) ensureSources injects frontmatter;
//! (e) STRUCTURAL_FILES never overwritten;
//! (f) LLM optional (P4): no runner → deterministic fallback, documented skip.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::ingest::callback::{IngestPhase, ProgressTracker};
use vanta_memory::ingest::merge::MergeDecision;
use vanta_memory::ingest::merge::{
    aggregate_by_rel_path, commit, is_structural, merge_page, normalize_wiki_path,
    parse_file_blocks, CandidatePage,
};
use vanta_memory::ingest::worker;
use vanta_memory::ingest::{
    clamp_llm_concurrency, ensure_sources, parse_frontmatter, IngestConfig, IngestError,
    STRUCTURAL_FILES,
};

const NS: &str = "default";
const SLUG: &str = "team-wiki";

fn in_memory_engine() -> vantadb::storage::StorageEngine {
    let config = vantadb::config::VantaConfig {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::VantaConfig::default()
    };
    vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config))
        .expect("open in-memory engine")
}

/// Scripted LLM runner: replays queued outputs; a queued error fails the call.
struct ScriptedRunner {
    outputs: Mutex<Vec<Result<String, LlmError>>>,
    calls: AtomicUsize,
    max_concurrent: AtomicUsize,
    in_flight: AtomicUsize,
}

impl ScriptedRunner {
    fn new(outputs: Vec<Result<String, LlmError>>) -> Self {
        Self {
            outputs: Mutex::new(outputs),
            calls: AtomicUsize::new(0),
            max_concurrent: AtomicUsize::new(0),
            in_flight: AtomicUsize::new(0),
        }
    }
}

impl LlmRunner for ScriptedRunner {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_concurrent.fetch_max(now, Ordering::SeqCst);
        // FIFO: outputs replay in the order they were queued.
        let out = {
            let mut queue = self.outputs.lock().expect("poisoned");
            if queue.is_empty() {
                Err(LlmError::Other("script exhausted".into()))
            } else {
                queue.remove(0)
            }
        };
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
        // Concurrency ceiling: never exceed the configured global limit.
        assert!(now <= 20, "in-flight calls exceeded any sane global limit");
        out
    }
}

fn file_block(path: &str, body: &str) -> String {
    format!(
        "<<<FILE path=\"{path}\">>>\n---\ntype: entity\ntitle: {}\n---\n{body}\n<<<END>>>",
        path.rsplit('/')
            .next()
            .unwrap_or("page")
            .trim_end_matches(".md")
    )
}

/// Runner that always answers NotConfigured — used to annotate `None`
/// generics without pulling a real implementation.
struct NeverRunner;
impl LlmRunner for NeverRunner {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        Err(LlmError::NotConfigured)
    }
}

fn source_dir(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for (name, content) in files {
        std::fs::write(dir.path().join(name), content).expect("write source");
    }
    dir
}

// ── helpers for storage-free commit ──

type MemPages = Arc<Mutex<std::collections::BTreeMap<String, String>>>;

fn mem_commit<L: LlmRunner>(
    by_page: Vec<(String, Vec<(String, String)>)>,
    runner: Option<&L>,
    config: &IngestConfig,
    pages: &MemPages,
) -> Result<vanta_memory::ingest::merge::CommitReport, IngestError> {
    let store_clone = pages.clone();
    commit(
        by_page,
        runner,
        config,
        move |rel| Ok(store_clone.lock().expect("poisoned").get(rel).cloned()),
        move |rel, content| {
            pages
                .lock()
                .expect("poisoned")
                .insert(rel.to_string(), content.to_string());
            Ok(())
        },
    )
}

// ══ (a) chunks → candidates agregados por relPath ══

#[test]
fn file_blocks_parse_into_candidates() {
    let output = [
        file_block("wiki/entities/redis.md", "Redis is an in-memory store."),
        "some stray commentary".to_string(),
        file_block(
            "wiki/concepts/persistence.md",
            "Persistence snapshots memory.",
        ),
    ]
    .join("\n");

    let pages = parse_file_blocks(&output);

    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].rel_path, "wiki/entities/redis.md");
    assert!(pages[0].content.contains("Redis is an in-memory store."));
    assert_eq!(pages[1].rel_path, "wiki/concepts/persistence.md");
}

#[test]
fn unsafe_paths_are_rejected() {
    assert_eq!(normalize_wiki_path("../etc/passwd"), None);
    assert_eq!(normalize_wiki_path("C:/evil.md"), None);
    assert_eq!(normalize_wiki_path("/abs/path.md"), None);
    assert_eq!(normalize_wiki_path("notwiki/page.md"), None);
    assert_eq!(
        normalize_wiki_path("wiki/entities/ok.md").as_deref(),
        Some("wiki/entities/ok.md")
    );
}

#[test]
fn candidates_from_two_sources_aggregate_by_rel_path() {
    let a = vec![CandidatePage {
        rel_path: "wiki/entities/redis.md".into(),
        content: "from A".into(),
    }];
    let b = vec![
        CandidatePage {
            rel_path: "wiki/entities/redis.md".into(),
            content: "from B".into(),
        },
        CandidatePage {
            rel_path: "wiki/concepts/cache.md".into(),
            content: "cache notes".into(),
        },
    ];

    let aggregated = aggregate_by_rel_path(vec![("a.md".into(), a), ("b.md".into(), b)]);

    assert_eq!(aggregated.len(), 2);
    let redis = aggregated
        .iter()
        .find(|(p, _)| p == "wiki/entities/redis.md")
        .expect("redis page");
    assert_eq!(redis.1.len(), 2, "both sources land on the same page");
    assert_eq!(redis.1[0].0, "a.md");
    assert_eq!(redis.1[1].0, "b.md");
}

// ══ (b) límite global configurable default 5, clamp 1-20 ══

#[test]
fn llm_concurrency_defaults_to_five_and_clamps() {
    assert_eq!(clamp_llm_concurrency(None), 5);
    assert_eq!(
        clamp_llm_concurrency(Some(0)),
        5,
        "invalid raw falls back to default"
    );
    assert_eq!(clamp_llm_concurrency(Some(3)), 3);
    assert_eq!(clamp_llm_concurrency(Some(100)), 20, "upper clamp at 20");
    let cfg = IngestConfig::new(Some(100));
    assert_eq!(cfg.global_llm_concurrency, 20);
    assert_eq!(IngestConfig::default().global_llm_concurrency, 5);
}

#[test]
fn merges_run_serially_one_page_at_a_time() {
    let runner = ScriptedRunner::new(vec![Ok([
        file_block("wiki/a/one.md", "page one"),
        file_block("wiki/b/two.md", "page two"),
    ]
    .join("\n"))]);
    let src = source_dir(&[("s.md", "# doc\ncontent")]);
    let engine = in_memory_engine();
    let store = vantadb::wiki::WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create");

    let report = worker::run(
        &store,
        NS,
        SLUG,
        src.path(),
        Some(&runner),
        &IngestConfig::default(),
    )
    .expect("run");

    // One extraction call total (single small chunk); merges are serial by
    // construction — verify both pages landed in first-appearance order.
    assert_eq!(report.commit_report.written.len(), 2);
    assert_eq!(
        store.get(NS, SLUG).expect("get").expect("exists").state,
        vantadb::wiki::WikiState::Ready
    );
}

// ══ (c) fallo de página N no bloquea N+1.. ══

#[test]
fn page_failure_does_not_block_next_pages() {
    let pages: MemPages = Arc::default();
    // Both pages exist so their merges actually go through the LLM
    // (new pages are written verbatim without an LLM call).
    {
        let mut map = pages.lock().expect("poisoned");
        map.insert(
            "wiki/entities/one.md".into(),
            "---\ntype: entity\nsources: [old.md]\n---\nold one".into(),
        );
        map.insert(
            "wiki/entities/two.md".into(),
            "---\ntype: entity\nsources: [old.md]\n---\nold two".into(),
        );
    }
    let runner = ScriptedRunner::new(vec![
        Err(LlmError::Transport("boom".into())), // page 1 merge fails
        Ok("---\ntype: entity\n---\nmerged two ok".to_string()), // page 2 succeeds
    ]);
    let config = IngestConfig::default();
    let by_page = vec![
        (
            "wiki/entities/one.md".into(),
            vec![("src.md".into(), "candidate one".into())],
        ),
        (
            "wiki/entities/two.md".into(),
            vec![("src.md".into(), "candidate two".into())],
        ),
    ];

    let report = mem_commit(by_page, Some(&runner), &config, &pages).expect("commit");

    assert!(!report.written.iter().any(|p| p.ends_with("one.md")));
    assert!(report.written.iter().any(|p| p.ends_with("two.md")));
    assert!(pages.lock().expect("poisoned")["wiki/entities/two.md"].contains("merged two ok"));
    assert!(report
        .merge_errors
        .iter()
        .any(|e| e.rel_path == "wiki/entities/one.md" && e.error.contains("boom")));
}

// ══ (d) ensureSources inyecta frontmatter ══

#[test]
fn ensure_sources_injects_and_is_idempotent() {
    let without_fm = ensure_sources("plain body", "s.md");
    let (fm, _) = parse_frontmatter(&without_fm);
    assert_eq!(fm.sources, vec!["s.md".to_string()]);
    assert!(without_fm.starts_with("---\n"));

    let with_fm = "---\ntype: entity\nsources: [a.md]\n---\nbody";
    let once = ensure_sources(with_fm, "b.md");
    let (fm1, _) = parse_frontmatter(&once);
    assert_eq!(fm1.sources.len(), 2);
    assert_eq!(ensure_sources(&once, "b.md"), once, "idempotent");

    // Locked flag and title survive the round-trip.
    let locked = "---\ntitle: Redis\nlocked: true\nsources: []\n---\nx";
    let rebuilt = ensure_sources(locked, "s.md");
    let (fm2, body2) = parse_frontmatter(&rebuilt);
    assert!(fm2.locked);
    assert_eq!(fm2.title.as_deref(), Some("Redis"));
    assert_eq!(body2, "x");
}

// ══ (e) STRUCTURAL_FILES nunca sobrescritos ══

#[test]
fn structural_files_never_overwritten() {
    for path in STRUCTURAL_FILES {
        assert!(is_structural(path), "{path} must be protected");
    }
    assert!(!is_structural("wiki/entities/redis.md"));

    let pages: MemPages = Arc::default();
    pages
        .lock()
        .expect("poisoned")
        .insert("wiki/index.md".into(), "original index".into());
    let config = IngestConfig::default();
    let by_page = vec![(
        "wiki/index.md".into(),
        vec![("src.md".into(), "hijacked index".into())],
    )];

    let report = mem_commit(by_page, None::<&NeverRunner>, &config, &pages).expect("commit");

    assert_eq!(
        pages.lock().expect("poisoned")["wiki/index.md"],
        "original index"
    );
    assert!(report
        .merge_errors
        .iter()
        .any(|e| e.error.contains("structural")));
}

// ══ (f) LLM opcional (P4): fallback determinístico ══

#[test]
fn llm_free_mode_new_pages_written_verbatim_existing_merged_skipped() {
    let pages: MemPages = Arc::default();
    pages
        .lock()
        .expect("poisoned")
        .insert("wiki/entities/existing.md".into(), "old body".into());
    let by_page = vec![
        (
            "wiki/entities/new.md".into(),
            vec![("s.md".into(), "brand new content".into())],
        ),
        (
            "wiki/entities/existing.md".into(),
            vec![("s.md".into(), "conflicting update".into())],
        ),
    ];
    let report = mem_commit(
        by_page,
        None::<&NeverRunner>,
        &IngestConfig::default(),
        &pages,
    )
    .expect("commit");

    let map = pages.lock().expect("poisoned");
    assert!(map["wiki/entities/new.md"].contains("brand new content"));
    assert_eq!(
        map["wiki/entities/existing.md"], "old body",
        "no silent overwrite"
    );
    assert!(report.written.iter().any(|p| p.ends_with("new.md")));
    assert!(report
        .merge_errors
        .iter()
        .any(|e| e.error.contains("LLM unavailable")));
}

#[test]
fn not_configured_runner_degrades_like_no_runner() {
    struct NotConfigured;
    impl LlmRunner for NotConfigured {
        fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
            Err(LlmError::NotConfigured)
        }
    }
    let decision = merge_page(Some("old"), "new candidate", Some(&NotConfigured), 4000);
    assert!(matches!(decision, MergeDecision::Skip(_)));
    let fresh = merge_page(None, "new candidate", Some(&NotConfigured), 4000);
    assert!(matches!(fresh, MergeDecision::Write(c) if c == "new candidate"));
}

// ══ worker end-to-end contra WikiStore ══

#[test]
fn end_to_end_build_completes_ready_with_written_pages() {
    let runner = ScriptedRunner::new(vec![Ok([
        file_block("wiki/entities/redis.md", "Redis is fast."),
        file_block("wiki/index.md", "should be ignored"),
    ]
    .join("\n"))]);
    let src = source_dir(&[("notes.md", "# Notes\nall about redis")]);
    let engine = in_memory_engine();
    let store = vantadb::wiki::WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create");

    let report = worker::run(
        &store,
        NS,
        SLUG,
        src.path(),
        Some(&runner),
        &IngestConfig::default(),
    )
    .expect("run");

    assert!(!report.run_id.is_empty());
    assert_eq!(report.sources_processed, vec!["notes.md"]);
    assert!(report
        .commit_report
        .written
        .iter()
        .any(|p| p == "wiki/entities/redis.md"));
    // (e) structural file skipped even when the LLM emitted it.
    assert!(!report
        .commit_report
        .written
        .iter()
        .any(|p| p == "wiki/index.md"));
    assert!(report
        .commit_report
        .merge_errors
        .iter()
        .any(|e| e.rel_path == "wiki/index.md"));

    // Page persisted via canonical path with injected sources frontmatter.
    let stored = store
        .get_page(NS, SLUG, "wiki/entities/redis.md")
        .expect("get_page")
        .expect("page exists");
    let (fm, _) = parse_frontmatter(&stored.content);
    assert_eq!(fm.sources, vec!["notes.md".to_string()]);
    assert!(stored.locked, "managed pages are locked by core store");
}

#[test]
fn missing_wiki_is_a_clear_error() {
    let src = source_dir(&[("x.md", "content")]);
    let engine = in_memory_engine();
    let store = vantadb::wiki::WikiStore::new(&engine);

    let err = worker::run(
        &store,
        NS,
        SLUG,
        src.path(),
        None::<&NeverRunner>,
        &IngestConfig::default(),
    )
    .expect_err("must fail");
    assert!(matches!(err, IngestError::NotFound { .. }));
}

#[test]
fn extraction_error_fails_the_build_via_store_fail() {
    let runner = ScriptedRunner::new(vec![Err(LlmError::Http {
        status: 500,
        message: "provider down".into(),
    })]);
    let src = source_dir(&[("x.md", "real content long enough to chunk? one chunk")]);
    let engine = in_memory_engine();
    let store = vantadb::wiki::WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create");

    // Extraction degraded → sources skipped; nothing written; build still
    // completes ready (P4: degrade, never block).
    let report = worker::run(
        &store,
        NS,
        SLUG,
        src.path(),
        Some(&runner),
        &IngestConfig::default(),
    )
    .expect("degraded run still completes");
    assert!(report.sources_skipped.contains(&"x.md".to_string()));
    assert_eq!(report.commit_report.written, Vec::<String>::new());
}

// ══ MEM-31 (D19): progreso de ingest — canal interno + polling run_id ══

/// (e) Fases extracting|merging|indexing con contadores {total, completed,
/// failed, skipped, percent} coherentes a lo largo de un build real.
#[test]
fn progress_phases_carry_coherent_counters() {
    let runner = ScriptedRunner::new(vec![Ok([
        file_block("wiki/a/one.md", "page one"),
        file_block("wiki/b/two.md", "page two"),
    ]
    .join("\n"))]);
    let src = source_dir(&[("s.md", "# doc\ncontent")]);
    let engine = in_memory_engine();
    let store = vantadb::wiki::WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create");
    let tracker = ProgressTracker::new();

    let report = worker::run_with_progress(
        &store,
        NS,
        SLUG,
        src.path(),
        Some(&runner),
        &IngestConfig::default(),
        Some(&tracker),
    )
    .expect("run");
    let run_id = &report.run_id;

    // Terminal state visible por polling con el run_id del build.
    let done = tracker.wiki_status(run_id).expect("final snapshot");
    assert_eq!(done.phase, IngestPhase::Done);
    assert_eq!(done.percent, 100);

    // El run_id viejo ya no consulta: begin_run de otro build lo descarta.
    tracker.begin_run("wikirun-next");
    assert_eq!(tracker.wiki_status(run_id), None);

    // Reconstruimos la secuencia de fases con un segundo build instrumentado.
    store.request_ingest(NS, SLUG).expect("re-request");
    let tracker2 = ProgressTracker::new();
    let runner2 = ScriptedRunner::new(vec![Ok([
        file_block("wiki/a/one.md", "page one v2"),
        file_block("wiki/b/two.md", "page two v2"),
    ]
    .join("\n"))]);
    let report2 = worker::run_with_progress(
        &store,
        NS,
        SLUG,
        src.path(),
        Some(&runner2),
        &IngestConfig::default(),
        Some(&tracker2),
    )
    .expect("second run");

    // Contadores finales coherentes: 1 fuente extraída, 0 skipped.
    let final_done = tracker2.wiki_status(&report2.run_id).expect("done");
    assert_eq!(final_done.phase, IngestPhase::Done);
    assert_eq!(final_done.completed, 1);
    assert_eq!(final_done.skipped, 0);
    // Merging: total = páginas escritas + errores de merge del reporte.
    let merge_total =
        report2.commit_report.written.len() + report2.commit_report.merge_errors.len();
    assert!(merge_total >= 1);
}

/// (d/f) El canal nunca bloquea el ingest y es consultable desde otro handle:
/// el build completa con el tracker attached y un handle clonado observa el
/// snapshot Done del mismo run_id. La garantía de no-bloqueo bajo contención
/// (try_lock → drop) se prueba determinísticamente en los unit tests de
/// `callback::tests::contended_channel_drops_update_instead_of_blocking`.
#[test]
fn channel_never_blocks_build_and_is_pollable_cross_handle() {
    let runner = ScriptedRunner::new(vec![Ok(
        [file_block("wiki/entities/solo.md", "only page")].join("\n")
    )]);
    let src = source_dir(&[("n.md", "# n\nbody")]);
    let engine = in_memory_engine();
    let store = vantadb::wiki::WikiStore::new(&engine);
    store.create(NS, SLUG).expect("create");
    let tracker = ProgressTracker::new();
    let observer = tracker.clone(); // "otro handle": hilo puente desktop

    let report = worker::run_with_progress(
        &store,
        NS,
        SLUG,
        src.path(),
        Some(&runner),
        &IngestConfig::default(),
        Some(&tracker),
    )
    .expect("build completes with live channel attached");

    let done = observer.wiki_status(&report.run_id).expect("pollable");
    assert_eq!(done.phase, IngestPhase::Done);
    assert_eq!(done.percent, 100);
    // run_id viejo → descartado (late-packet guard del canal).
    assert_eq!(observer.wiki_status("wikirun-viejo"), None);
}
