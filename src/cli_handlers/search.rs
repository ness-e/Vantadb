//! Search command handler — semantic/hybrid search.

use console::Term;

use crate::cli_handlers::fmt::{header_style, info_style, warning_style};
use crate::cli_handlers::{create_spinner, open_embedded, print_warning};
use crate::error::{ChainedError, Result};

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
    let db = open_embedded(db_path, true)?;
    spinner.set_message("Searching...");

    let query_vector = if let Some(qv) = query_vector_str {
        qv.split(',')
            .map(|s| {
                s.trim().parse::<f32>().map_err(|e| {
                    crate::error::VantaError::InvalidInput(format!(
                        "Invalid vector component '{s}': {e}"
                    ))
                })
            })
            .collect::<std::result::Result<Vec<f32>, _>>()?
    } else {
        vec![]
    };

    let request = crate::sdk::VantaMemorySearchRequest {
        namespace: namespace.to_string(),
        query_vector,
        query_sparse: None,
        filters: crate::sdk::VantaMemoryMetadata::new(),
        text_query: Some(query.to_string()),
        top_k: limit,
        distance_metric: crate::node::DistanceMetric::Cosine,
        explain: false,
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
                crate::error::VantaError::CliError(ChainedError::msg(format!(
                    "JSON serialization error: {e}"
                )))
            })?
        );
        return Ok(());
    }

    let term = Term::stdout();
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
                    &hit.record.payload[..hit.record.payload.len().min(80)]
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
    top_k: usize,
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
    let db = crate::cli_handlers::open_embedded(db_path, true)?;
    spinner.set_message("Searching similar records...");

    let hits = db.similar_to_key(namespace, key, top_k)?;
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
                crate::error::VantaError::CliError(ChainedError::msg(format!(
                    "JSON serialization error: {e}"
                )))
            })?
        );
        return Ok(());
    }

    let term = Term::stdout();
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
            let payload_preview = &hit.record.payload[..hit.record.payload.len().min(80)];
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
                    crate::error::VantaError::InvalidInput(format!(
                        "Invalid vector component '{tok}': {e}"
                    ))
                })
            })
            .collect(),
    }
}

/// Render search hits to stdout (shared by search_multi and search_all).
fn print_hits(
    hits: &[crate::sdk::VantaMemorySearchHit],
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
                crate::error::VantaError::CliError(crate::error::ChainedError::msg(format!(
                    "JSON serialization error: {e}"
                )))
            })?
        );
        return Ok(());
    }

    let term = Term::stdout();
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
        let preview = &hit.record.payload[..hit.record.payload.len().min(80)];
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
    top_k: usize,
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
    let db = open_embedded(db_path, true)?;
    spinner.set_message("Searching across namespaces...");

    let query_vector = parse_query_vector(query_vector_str)?;

    let request = crate::sdk::VantaMemorySearchRequest {
        // namespace is overridden per-namespace inside search_multi
        namespace: String::new(),
        query_vector,
        query_sparse: None,
        filters: crate::sdk::VantaMemoryMetadata::new(),
        text_query: query.map(str::to_string),
        top_k,
        distance_metric: crate::node::DistanceMetric::Cosine,
        explain: false,
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
    top_k: usize,
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
    let db = open_embedded(db_path, true)?;
    spinner.set_message("Discovering namespaces and searching...");

    let query_vector = parse_query_vector(query_vector_str)?;

    let request = crate::sdk::VantaMemorySearchRequest {
        namespace: String::new(),
        query_vector,
        query_sparse: None,
        filters: crate::sdk::VantaMemoryMetadata::new(),
        text_query: query.map(str::to_string),
        top_k,
        distance_metric: crate::node::DistanceMetric::Cosine,
        explain: false,
    };

    let hits = db.search_all(request)?;
    spinner.finish_and_clear();

    print_hits(&hits, json_output, "Search results across all namespaces:")
}
