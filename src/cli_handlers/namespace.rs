//! Namespace command handlers — list and info.

use console::Term;

use crate::cli_handlers::fmt::{header_style, info_style, warning_style};
use crate::cli_handlers::{create_spinner, open_embedded, print_warning};
use crate::error::Result;

#[tracing::instrument]
/// List all namespaces in the database
pub fn cmd_namespace_list(db_path: &str) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let db = open_embedded(db_path, true)?;
    spinner.set_message("Listing namespaces...");
    let namespaces = db.list_namespaces()?;
    spinner.finish_and_clear();

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╭─────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(format!(
            "│  Namespaces ({})                          │",
            namespaces.len()
        ))
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("├─────────────────────────────────────────┤")
    ));

    if namespaces.is_empty() {
        let _ = term.write_line(&format!(
            "{}",
            warning_style().apply_to("│  No namespaces found                     │")
        ));
    } else {
        for ns in &namespaces {
            let _ = term.write_line(&format!(
                "{}",
                info_style().apply_to(format!("│  • {}", ns))
            ));
        }
    }

    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰─────────────────────────────────────────╯")
    ));

    Ok(())
}

#[tracing::instrument]
/// Show record count and details for a specific namespace
pub fn cmd_namespace_info(db_path: &str, namespace: &str) -> Result<()> {
    let path = std::path::Path::new(db_path);
    if !path.exists() {
        print_warning(&format!(
            "Database directory does not exist at '{}'. (empty)",
            db_path
        ));
        return Ok(());
    }

    let spinner = create_spinner("Opening database...");
    let db = open_embedded(db_path, true)?;
    spinner.set_message("Scanning namespace...");

    let options = crate::sdk::VantaMemoryListOptions {
        filters: crate::sdk::VantaMemoryMetadata::new(),
        limit: usize::MAX,
        cursor: None,
    };
    let page = db.list(namespace, options)?;
    spinner.finish_and_clear();

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╭─────────────────────────────────────────╮")
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(format!("│  Namespace: {}", namespace))
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("├─────────────────────────────────────────┤")
    ));
    let _ = term.write_line(&format!(
        "{}",
        info_style().apply_to(format!("│  Records: {}", page.records.len()))
    ));

    if page.records.is_empty() {
        let _ = term.write_line(&format!(
            "{}",
            warning_style().apply_to("│  (empty)                                   │")
        ));
    } else {
        let total_payload: usize = page.records.iter().map(|r| r.payload.len()).sum();
        let _ = term.write_line(&format!(
            "{}",
            info_style().apply_to(format!("│  Total payload: {} bytes", total_payload))
        ));
    }

    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to("╰─────────────────────────────────────────╯")
    ));

    Ok(())
}
