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
