use std::env;
use std::io::Write;
use vantadb::config::{SyncMode, VantaConfig};
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: crash_helper <db_path> <count>");
        std::process::exit(1);
    }
    let db_path = &args[1];
    let count = args[2].parse::<u32>().expect("Invalid count");

    // Configurar base de datos con durabilidad fsync estricta (Always)
    let config = VantaConfig {
        sync_mode: SyncMode::Always,
        ..Default::default()
    };

    let engine = StorageEngine::open_with_config(db_path, Some(config))
        .expect("Failed to open StorageEngine");

    for i in 1..=count {
        let node = UnifiedNode::new(i as u64);
        if let Err(e) = engine.insert(&node) {
            eprintln!("CRASH_HELPER: FAILED_INSERT at {}: {}", i, e);
            let _ = std::io::stderr().flush();
            std::process::exit(2);
        }

        // Imprimir la confirmación de la transacción en caliente a stdout
        println!("WRITTEN:{}", i);
        if let Err(e) = std::io::stdout().flush() {
            eprintln!("CRASH_HELPER: FAILED_STDOUT_FLUSH at {}: {}", i, e);
            std::process::exit(4);
        }

        // flush periódico para forzar index sync y simulación realista de checkpoints
        if i % 10 == 0 {
            if let Err(e) = engine.flush() {
                eprintln!("CRASH_HELPER: FAILED_FLUSH at {}: {}", i, e);
                let _ = std::io::stderr().flush();
                std::process::exit(3);
            }
        }

        // Delay para permitir al proceso padre interceptar y matar
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    // Flush final
    let _ = engine.flush();
    println!("CRASH_HELPER: FINISHED");
    let _ = std::io::stdout().flush();
}
