use std::collections::HashMap;
use std::env;

use vantadb::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions};

fn usage() {
    eprintln!("Usage:");
    eprintln!("  vanta-cli put --db <path> --namespace <ns> --key <key> --payload <text>");
    eprintln!("  vanta-cli get --db <path> --namespace <ns> --key <key>");
    eprintln!("  vanta-cli list --db <path> --namespace <ns>");
    eprintln!("  vanta-cli rebuild-index --db <path>");
    eprintln!("  vanta-cli export --db <path> [--namespace <ns>] --out <file>");
    eprintln!("  vanta-cli import --db <path> --in <file>");
}

fn parse_flags(args: &[String]) -> HashMap<String, String> {
    let mut flags = HashMap::new();
    let mut index = 0usize;
    while index < args.len() {
        if let Some(name) = args[index].strip_prefix("--") {
            if let Some(value) = args.get(index + 1) {
                flags.insert(name.to_string(), value.clone());
                index += 2;
                continue;
            }
        }
        index += 1;
    }
    flags
}

fn required<'a>(flags: &'a HashMap<String, String>, name: &str) -> Result<&'a str, String> {
    flags
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("missing --{}", name))
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        usage();
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        usage();
        return Ok(());
    }

    let command = args.remove(0);
    let flags = parse_flags(&args);
    let db_path = required(&flags, "db")?;
    let db = VantaEmbedded::open(db_path)?;

    match command.as_str() {
        "put" => {
            let namespace = required(&flags, "namespace")?;
            let key = required(&flags, "key")?;
            let payload = required(&flags, "payload")?;
            let record = db.put(VantaMemoryInput::new(namespace, key, payload))?;
            db.flush()?;
            println!(
                "stored namespace={} key={} version={} node_id={}",
                record.namespace, record.key, record.version, record.node_id
            );
        }
        "get" => {
            let namespace = required(&flags, "namespace")?;
            let key = required(&flags, "key")?;
            match db.get(namespace, key)? {
                Some(record) => println!("{}", record.payload),
                None => {
                    println!("not found");
                    std::process::exit(2);
                }
            }
        }
        "list" => {
            let namespace = required(&flags, "namespace")?;
            let page = db.list(namespace, VantaMemoryListOptions::default())?;
            for record in page.records {
                println!("{}\t{}", record.key, record.payload);
            }
        }
        "rebuild-index" => {
            let report = db.rebuild_index()?;
            println!(
                "rebuild success={} scanned_nodes={} indexed_vectors={} skipped_tombstones={} duration_ms={} index_path={}",
                report.success,
                report.scanned_nodes,
                report.indexed_vectors,
                report.skipped_tombstones,
                report.duration_ms,
                report.index_path
            );
        }
        "export" => {
            let out = required(&flags, "out")?;
            let report = if let Some(namespace) = flags.get("namespace") {
                db.export_namespace(out, namespace)?
            } else {
                db.export_all(out)?
            };
            println!(
                "exported records={} namespaces={} duration_ms={} path={}",
                report.records_exported,
                report.namespaces.join(","),
                report.duration_ms,
                report.path
            );
        }
        "import" => {
            let input = required(&flags, "in")?;
            let report = db.import_file(input)?;
            db.flush()?;
            println!(
                "imported inserted={} updated={} skipped={} errors={} duration_ms={}",
                report.inserted, report.updated, report.skipped, report.errors, report.duration_ms
            );
        }
        _ => return Err(format!("unknown command '{}'", command).into()),
    }

    Ok(())
}
