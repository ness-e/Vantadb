//! Search command handler — semantic/hybrid search.

use console::Term;

use crate::cli_handlers::fmt::{header_style, info_style, warning_style};
use crate::cli_handlers::{
    create_spinner, open_embedded, print_warning, stdout_is_term, truncate_for_term,
};
use crate::error::{ChainedError, Result};

/// Open the database read-only for a read-only command.
///
/// Returns `Ok(None)` when the directory exists but was never initialised
/// (read-only opens cannot create the lock/schema files) — callers treat that
/// exactly like a missing path (empty). Genuine errors propagate.
fn open_readonly_or_empty(db_path: &str) -> Result<Option<crate::Embedded>> {
    match open_embedded(db_path, true) {
        Ok(db) => Ok(Some(db)),
        Err(e) if crate::cli_handlers::crud::is_uninitialized_db(&e) => Ok(None),
        Err(e) => Err(e),
    }
}

#[tracing::instrument]
/// Perform semantic or hybrid search across a namespace
pub fn cmd_search(
    db_path: &str,
    namespace: &str,
    query: &str,
    query_vector_str: Option<&str>,
    limit: usize,
    json_output: bool,
) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            println!("[]");
            return Ok(());
        }
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    // API-07: reads open read-only (shared lock) — the write path keeps the
    // derived/text/sparse indexes current via the SDK (`Embedded::put`), so
    // the read paths no longer need a read-write open for reconciliation.
    let Some(db) = open_readonly_or_empty(db_path)? else {
        spinner.finish_and_clear();
        if json_output {
            println!("[]");
        } else {
            print_warning(&format!(
                "Database directory is not initialized at '{}'. (empty)",
                db_path
            ));
        }
        return Ok(());
    };
    spinner.set_message("Searching...");

    let query_vector = if let Some(qv) = query_vector_str {
        qv.split(',')
            .map(|s| {
                s.trim().parse::<f32>().map_err(|e| {
                    crate::error::Error::InvalidInput(format!(
                        "Invalid vector component '{s}': {e}"
                    ))
                })
            })
            .collect::<std::result::Result<Vec<f32>, _>>()?
    } else {
        vec![]
    };

    let request = crate::sdk::MemorySearchRequest {
        namespace: namespace.to_string(),
        query_vector,
        query_sparse: None,
        filters: crate::sdk::MemoryMetadata::new(),
        text_query: Some(query.to_string()),
        top_k: limit,
        distance_metric: crate::node::DistanceMetric::Cosine,
        explain: false,
        exclude_superseded: false,
        min_confidence: None,
        as_of_ms: None,
        valid_window: None,
        include_quarantined: false,
        search_profile: None,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
    };

    let hits = db.search(request)?;
    spinner.finish_and_clear();

    if json_output {
        let results: Vec<serde_json::Value> = hits
            .iter()
            .map(|hit| {
                serde_json::json!({
                    "key": hit.record.key,
                    "namespace": hit.record.namespace,
                    "payload": hit.record.payload,
                    "score": hit.score,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&results).map_err(|e| {
                crate::error::Error::Cli(ChainedError::msg(format!(
                    "JSON serialization error: {e}"
                )))
            })?
        );
        return Ok(());
    }

    let term = Term::stdout();
    let is_term = stdout_is_term();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style()
            .apply_to("╭──────────────────────────────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(format!(
            "│  Search results for \"{}\" in namespace \"{}\" ({}{}) │",
            query,
            namespace,
            hits.len(),
            if hits.len() < limit && !hits.is_empty() {
                " max"
            } else {
                ""
            }
        ))
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style()
            .apply_to("├──────────────────────────────────────────────────────────────────┤")
    ));

    if hits.is_empty() {
        let _ = term.write_line(&format!(
            "{}",
            warning_style().apply_to("│  No results found                                   │")
        ));
    } else {
        for (i, hit) in hits.iter().enumerate() {
            let _ = term.write_line(&format!(
                "{}",
                info_style().apply_to(format!(
                    "│  #{:<3} │ Score: {:<8} │ {}:{}",
                    i + 1,
                    format!("{:.6}", hit.score),
                    hit.record.namespace,
                    hit.record.key
                ))
            ));
            let _ = term.write_line(&format!(
                "{}",
                info_style().apply_to(format!(
                    "│       │ Payload:  {}",
                    truncate_for_term(&hit.record.payload, 80, is_term)
                ))
            ));
            if i < hits.len() - 1 {
                let _ = term.write_line(&format!(
                    "{}",
                    info_style().apply_to("│       │           │")
                ));
            }
        }
    }

    let _ = term.write_line(&format!(
        "{}",
        header_style()
            .apply_to("╰──────────────────────────────────────────────────────────────────╯")
    ));

    Ok(())
}

#[tracing::instrument]
/// Find records similar to a given key using vector similarity search
pub fn cmd_similar_to_key(
    db_path: &str,
    namespace: &str,
    key: &str,
    limit: usize,
    json_output: bool,
) -> crate::error::Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            println!("[]");
            return Ok(());
        }
        crate::cli_handlers::print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = crate::cli_handlers::create_spinner("Opening database...");
    // API-07: read-only open (shared lock); write path keeps indexes current.
    let Some(db) = open_readonly_or_empty(db_path)? else {
        spinner.finish_and_clear();
        if json_output {
            println!("[]");
        } else {
            print_warning(&format!(
                "Database directory is not initialized at '{}'. (empty)",
                db_path
            ));
        }
        return Ok(());
    };
    spinner.set_message("Searching similar records...");

    let hits = db.similar_to_key(namespace, key, limit)?;
    spinner.finish_and_clear();

    if json_output {
        let results: Vec<serde_json::Value> = hits
            .iter()
            .map(|hit| {
                serde_json::json!({
                    "key": hit.record.key,
                    "namespace": hit.record.namespace,
                    "payload": hit.record.payload,
                    "score": hit.score,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&results).map_err(|e| {
                crate::error::Error::Cli(ChainedError::msg(format!(
                    "JSON serialization error: {e}"
                )))
            })?
        );
        return Ok(());
    }

    let term = Term::stdout();
    let is_term = stdout_is_term();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style()
            .apply_to("╭──────────────────────────────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(format!(
            "│  Similar to '{}' in '{}' — {} result{}",
            key,
            namespace,
            hits.len(),
            if hits.len() == 1 { "" } else { "s" }
        ))
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style()
            .apply_to("├──────────────────────────────────────────────────────────────────┤")
    ));

    if hits.is_empty() {
        let _ = term.write_line(&format!(
            "{}",
            warning_style().apply_to("│  No similar records found                           │")
        ));
    } else {
        for (i, hit) in hits.iter().enumerate() {
            let _ = term.write_line(&format!(
                "{}",
                info_style().apply_to(format!(
                    "│  #{:<3} │ Score: {:<8} │ {}:{}",
                    i + 1,
                    format!("{:.6}", hit.score),
                    hit.record.namespace,
                    hit.record.key
                ))
            ));
            let payload_preview = truncate_for_term(&hit.record.payload, 80, is_term);
            let _ = term.write_line(&format!(
                "{}",
                info_style().apply_to(format!("│       │ Payload:  {}", payload_preview))
            ));
        }
    }

    let _ = term.write_line(&format!(
        "{}",
        header_style()
            .apply_to("╰──────────────────────────────────────────────────────────────────╯")
    ));

    Ok(())
}

// ── helpers ────────────────────────────────────────────────────────────────

/// Parse a comma-separated vector string into `Vec<f32>`.
fn parse_query_vector(s: Option<&str>) -> crate::error::Result<Vec<f32>> {
    match s {
        None => Ok(vec![]),
        Some(raw) => raw
            .split(',')
            .map(|tok| {
                tok.trim().parse::<f32>().map_err(|e| {
                    crate::error::Error::InvalidInput(format!(
                        "Invalid vector component '{tok}': {e}"
                    ))
                })
            })
            .collect(),
    }
}

/// Render search hits to stdout (shared by search_multi and search_all).
fn print_hits(
    hits: &[crate::sdk::MemorySearchHit],
    json_output: bool,
    header: &str,
) -> crate::error::Result<()> {
    if json_output {
        let results: Vec<serde_json::Value> = hits
            .iter()
            .map(|hit| {
                serde_json::json!({
                    "key":       hit.record.key,
                    "namespace": hit.record.namespace,
                    "payload":   hit.record.payload,
                    "score":     hit.score,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&results).map_err(|e| {
                crate::error::Error::Cli(crate::error::ChainedError::msg(format!(
                    "JSON serialization error: {e}"
                )))
            })?
        );
        return Ok(());
    }

    let term = Term::stdout();
    let is_term = stdout_is_term();
    let _ = term.write_line(&format!("{}", header_style().apply_to(header)));

    if hits.is_empty() {
        let _ = term.write_line(&format!(
            "{}",
            warning_style().apply_to("  No results found.")
        ));
    }

    for (i, hit) in hits.iter().enumerate() {
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to(format!(
                "  #{:<3} [score: {:.6}]  {}:{}",
                i + 1,
                hit.score,
                hit.record.namespace,
                hit.record.key
            ))
        ));
        let preview = truncate_for_term(&hit.record.payload, 80, is_term);
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to(format!("       payload: {}", preview))
        ));
    }

    Ok(())
}

// ── cmd_search_multi ───────────────────────────────────────────────────────

#[tracing::instrument]
/// Search across multiple named namespaces, merging results by score.
///
/// `namespaces_csv` is a comma-separated list, e.g. `"agent/main,agent/tools"`.
pub fn cmd_search_multi(
    db_path: &str,
    namespaces_csv: &str,
    query: Option<&str>,
    query_vector_str: Option<&str>,
    limit: usize,
    json_output: bool,
) -> crate::error::Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            println!("[]");
            return Ok(());
        }
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let namespaces: Vec<&str> = namespaces_csv
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if namespaces.is_empty() {
        if json_output {
            println!("[]");
        } else {
            print_warning("No namespaces specified.");
        }
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    // API-07: read-only open (shared lock); write path keeps indexes current.
    let Some(db) = open_readonly_or_empty(db_path)? else {
        spinner.finish_and_clear();
        if json_output {
            println!("[]");
        } else {
            print_warning(&format!(
                "Database directory is not initialized at '{}'. (empty)",
                db_path
            ));
        }
        return Ok(());
    };
    spinner.set_message("Searching across namespaces...");

    let query_vector = parse_query_vector(query_vector_str)?;

    let request = crate::sdk::MemorySearchRequest {
        // namespace is overridden per-namespace inside search_multi
        namespace: String::new(),
        query_vector,
        query_sparse: None,
        filters: crate::sdk::MemoryMetadata::new(),
        text_query: query.map(str::to_string),
        top_k: limit,
        distance_metric: crate::node::DistanceMetric::Cosine,
        explain: false,
        exclude_superseded: false,
        min_confidence: None,
        as_of_ms: None,
        valid_window: None,
        include_quarantined: false,
        search_profile: None,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
    };

    let hits = db.search_multi(&namespaces, request)?;
    spinner.finish_and_clear();

    print_hits(
        &hits,
        json_output,
        &format!(
            "Search results across namespaces [{}]:",
            namespaces.join(", ")
        ),
    )
}

// ── cmd_search_all ─────────────────────────────────────────────────────────

#[tracing::instrument]
/// Search across ALL known namespaces, merging results by score.
pub fn cmd_search_all(
    db_path: &str,
    query: Option<&str>,
    query_vector_str: Option<&str>,
    limit: usize,
    json_output: bool,
) -> crate::error::Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        if json_output {
            println!("[]");
            return Ok(());
        }
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    // API-07: read-only open (shared lock); write path keeps indexes current.
    let Some(db) = open_readonly_or_empty(db_path)? else {
        spinner.finish_and_clear();
        if json_output {
            println!("[]");
        } else {
            print_warning(&format!(
                "Database directory is not initialized at '{}'. (empty)",
                db_path
            ));
        }
        return Ok(());
    };
    spinner.set_message("Discovering namespaces and searching...");

    let query_vector = parse_query_vector(query_vector_str)?;

    let request = crate::sdk::MemorySearchRequest {
        namespace: String::new(),
        query_vector,
        query_sparse: None,
        filters: crate::sdk::MemoryMetadata::new(),
        text_query: query.map(str::to_string),
        top_k: limit,
        distance_metric: crate::node::DistanceMetric::Cosine,
        explain: false,
        exclude_superseded: false,
        min_confidence: None,
        as_of_ms: None,
        valid_window: None,
        include_quarantined: false,
        search_profile: None,
        range: None,
        group_by: None,
        mmr: None,
        cursor: None,
    };

    let hits = db.search_all(request)?;
    spinner.finish_and_clear();

    print_hits(&hits, json_output, "Search results across all namespaces:")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AUD-044/API-07 regression: `search` on a fresh DB after `put` must work
    /// without a manual `rebuild-index` step. The read paths open read-only
    /// (shared lock, `ensure_indexes_current` skipped), so this only works
    /// because `cmd_put` goes through the SDK and keeps the text index current
    /// at write time. If `put` regresses to a raw `engine.insert` bypass,
    /// text_query fails here with `NotFound { kind: "text_index", id: "bm25" }`.
    #[test]
    fn search_on_fresh_db_after_put_works_without_manual_rebuild() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().to_str().unwrap();

        crate::cli_handlers::cmd_put(
            db,
            "test",
            "k1",
            "hello world",
            Some("0.1,0.2,0.3"),
            None,
            false,
            false,
        )
        .expect("put should succeed");

        // search with a text query on the same fresh DB, JSON output (no tty)
        cmd_search(db, "test", "hello", None, 10, true).expect("search should not error");

        // same for similar-to-key (vector path, read-only open)
        cmd_similar_to_key(db, "test", "k1", 10, true).expect("similar_to_key should not error");
    }
}
