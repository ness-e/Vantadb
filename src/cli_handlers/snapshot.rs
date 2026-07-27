//! Snapshot command handlers — create and list filesystem snapshots.

use crate::cli_handlers::{create_spinner, open_embedded, print_info, print_success};
use crate::error::Result;

#[tracing::instrument]
/// Create an instant filesystem snapshot
pub fn cmd_snapshot_create(db_path: &str, name: &str, verbose: bool) -> Result<()> {
    let spinner = create_spinner("Creating snapshot...");
    let db = open_embedded(db_path, false)?;
    db.flush()?;
    let snap = db.create_snapshot(name)?;
    spinner.finish_and_clear();
    print_success(&format!(
        "Snapshot '{}' created at: {}",
        name,
        snap.path.display()
    ));
    if verbose {
        print_info(&format!("Snapshot path: {:?}", snap.path));
    }
    Ok(())
}

#[tracing::instrument]
/// List all existing snapshots
pub fn cmd_snapshot_list(db_path: &str) -> Result<()> {
    let db = open_embedded(db_path, true)?;
    let snapshots = db.list_snapshots()?;
    if snapshots.is_empty() {
        print_info("No snapshots found.");
    } else {
        print_info(&format!("Snapshots ({}):", snapshots.len()));
        for name in &snapshots {
            print_info(&format!("  - {}", name));
        }
    }
    Ok(())
}
