//! CRUD command handlers — put, get, list, delete.

use console::Term;
use web_time::{SystemTime, UNIX_EPOCH};

use crate::cli_handlers::{
    create_spinner, memory_node_id, open_database, open_embedded, print_error, print_info,
    print_success, print_warning, FIELD_CREATED_AT_MS, FIELD_KEY, FIELD_NAMESPACE, FIELD_PAYLOAD,
    FIELD_UPDATED_AT_MS, FIELD_VERSION,
};
use crate::error::Result;
use crate::node::{FieldValue, NodeFlags, VectorRepresentations};

#[tracing::instrument]
/// Store a key-value record with optional vector embedding
pub fn cmd_put(
    db_path: &str,
    namespace: &str,
    key: &str,
    payload: &str,
    vector: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let spinner = create_spinner("Opening database...");

    let engine = open_database(db_path, false)?;
    spinner.set_message("Preparing record...");

    // Parse optional vector
    let vector_data = if let Some(vec_str) = vector {
        let parsed: std::result::Result<Vec<f32>, _> = vec_str
            .split(',')
            .map(|s| s.trim().parse::<f32>().map_err(|e| e.to_string()))
            .collect();
        match parsed {
            Ok(v) => Some(v),
            Err(e) => {
                spinner.finish_and_clear();
                print_error(&format!("Invalid vector format: {}", e));
                return Err(crate::error::VantaError::CliError(format!(
                    "Vector must be comma-separated f32 values: {}",
                    e
                )));
            }
        }
    } else {
        None
    };

    spinner.set_message("Inserting record...");

    // Build the node with memory record fields
    let node_id = memory_node_id(namespace, key);
    let mut node = crate::node::UnifiedNode::new(node_id);

    // Set memory fields
    node.relational.insert(
        FIELD_NAMESPACE.to_string(),
        FieldValue::String(namespace.to_string()),
    );
    node.relational
        .insert(FIELD_KEY.to_string(), FieldValue::String(key.to_string()));
    node.relational.insert(
        FIELD_PAYLOAD.to_string(),
        FieldValue::String(payload.to_string()),
    );

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    node.relational.insert(
        FIELD_CREATED_AT_MS.to_string(),
        FieldValue::Int(now_ms as i64),
    );
    node.relational.insert(
        FIELD_UPDATED_AT_MS.to_string(),
        FieldValue::Int(now_ms as i64),
    );
    node.relational
        .insert(FIELD_VERSION.to_string(), FieldValue::Int(1));

    if let Some(vec) = vector_data {
        node.vector = VectorRepresentations::Full(vec);
        node.flags.set(NodeFlags::HAS_VECTOR);
    }

    node.flags.set(NodeFlags::ACTIVE);

    engine.insert(&node)?;
    engine.flush()?;

    spinner.finish_and_clear();

    if verbose {
        print_info(&format!("Node ID: {}", node_id));
        if let Some(v) = vector {
            print_info(&format!("Vector dimensions: {}", v.split(',').count()));
        }
    }

    print_success(&format!(
        "Record stored: {}:{} ({} bytes)",
        namespace,
        key,
        payload.len()
    ));

    Ok(())
}

#[tracing::instrument]
/// Retrieve and display a record by namespace and key
pub fn cmd_get(db_path: &str, namespace: &str, key: &str, verbose: bool) -> Result<()> {
    use crate::cli_handlers::fmt::{header_style, info_style};

    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let engine = open_database(db_path, true)?;
    spinner.set_message("Searching record...");

    let node_id = memory_node_id(namespace, key);

    match engine.get(node_id)? {
        Some(node) => {
            spinner.finish_and_clear();

            let term = Term::stdout();
            let _ = term.write_line("");
            let _ = term.write_line(&format!(
                "{}",
                header_style().apply_to("╭─────────────────────────────────────────╮")
            ));
            let _ = term.write_line(&format!(
                "{}",
                header_style().apply_to(format!("│  Record: {}:{}", namespace, key))
            ));
            let _ = term.write_line(&format!(
                "{}",
                header_style().apply_to("├─────────────────────────────────────────┤")
            ));

            // Display payload
            if let Some(FieldValue::String(payload)) = node.relational.get(FIELD_PAYLOAD) {
                let _ = term.write_line(&format!(
                    "{} {}",
                    info_style().apply_to("│  Payload:"),
                    payload
                ));
            }

            // Display vector info
            match &node.vector {
                VectorRepresentations::Full(v) => {
                    let _ = term.write_line(&format!(
                        "{} {} dimensions",
                        info_style().apply_to("│  Vector:"),
                        v.len()
                    ));
                }
                _ => {
                    let _ =
                        term.write_line(&format!("{} None", info_style().apply_to("│  Vector:")));
                }
            }

            // Display metadata
            if let Some(FieldValue::Int(created)) = node.relational.get(FIELD_CREATED_AT_MS) {
                let _ = term.write_line(&format!(
                    "{} {}",
                    info_style().apply_to("│  Created:"),
                    created
                ));
            }

            if let Some(FieldValue::Int(version)) = node.relational.get(FIELD_VERSION) {
                let _ = term.write_line(&format!(
                    "{} {}",
                    info_style().apply_to("│  Version:"),
                    version
                ));
            }

            let _ = term.write_line(&format!(
                "{}",
                header_style().apply_to("╰─────────────────────────────────────────╯")
            ));

            if verbose {
                print_info(&format!("Node ID: {}", node_id));
                print_info(&format!("Tier: {:?}", node.tier));
                print_info(&format!("Hits: {}", node.hits));
            }

            Ok(())
        }
        None => {
            spinner.finish_and_clear();
            print_error(&format!("Record not found: {}:{}", namespace, key));
            Err(crate::error::VantaError::NodeNotFound(node_id))
        }
    }
}

#[tracing::instrument]
/// List records in a namespace with an optional limit
pub fn cmd_list(db_path: &str, namespace: &str, limit: usize, verbose: bool) -> Result<()> {
    use crate::cli_handlers::fmt::header_style;

    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");

    let engine = open_database(db_path, true)?;
    spinner.set_message("Scanning namespace...");

    let nodes = engine.scan_nodes()?;

    // Filter by namespace
    let filtered: Vec<_> = nodes
        .into_iter()
        .filter(|n| {
            n.relational
                .get(FIELD_NAMESPACE)
                .map(|v| matches!(v, FieldValue::String(s) if s == namespace))
                .unwrap_or(false)
        })
        .take(limit)
        .collect();

    spinner.finish_and_clear();

    if filtered.is_empty() {
        print_warning(&format!("No records found in namespace '{}'", namespace));
        return Ok(());
    }

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(format!(
            "Records in '{}' (showing {})",
            namespace,
            filtered.len()
        ))
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("┌────────────────────┬────────────────────────────────────────┐")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("│ Key                │ Payload Preview                        │")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("├────────────────────┼────────────────────────────────────────┤")
    ));

    for node in &filtered {
        let key = node
            .relational
            .get(FIELD_KEY)
            .and_then(|v| match v {
                FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "?".to_string());

        let payload = node
            .relational
            .get(FIELD_PAYLOAD)
            .and_then(|v| match v {
                FieldValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "".to_string());

        let preview = if payload.len() > 38 {
            format!("{}...", &payload[..35])
        } else {
            payload
        };

        let _ = term.write_line(&format!("│ {:<18} │ {:<38} │", key, preview));
    }

    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("└────────────────────┴────────────────────────────────────────┘")
    ));

    if verbose {
        print_info(&format!("Total nodes scanned: {}", filtered.len()));
    }

    Ok(())
}

#[tracing::instrument]
/// Delete a record by namespace and key
pub fn cmd_delete(db_path: &str, namespace: &str, key: &str, verbose: bool) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let db = open_embedded(db_path, false)?;
    spinner.set_message("Deleting record...");

    let deleted = db.delete(namespace, key)?;
    spinner.finish_and_clear();

    if deleted {
        print_success(&format!("Record deleted: {}:{}", namespace, key));
        if verbose {
            let node_id = memory_node_id(namespace, key);
            print_info(&format!("Node ID: {}", node_id));
        }
    } else {
        print_warning(&format!("Record not found: {}:{}", namespace, key));
    }

    Ok(())
}
