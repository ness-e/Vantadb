use std::env;
use std::io::Write;
use std::thread;
use std::time::Duration;
use vantadb::storage::StorageEngine;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: lock_helper <db_path> [sleep_ms]");
        std::process::exit(1);
    }
    let db_path = &args[1];
    let sleep_ms = args.get(2)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(2000);

    // Intentamos abrir el StorageEngine.
    // Esto disparará internamente la obtención de .vanta.lock.
    match StorageEngine::open(db_path) {
        Ok(_engine) => {
            println!("LOCK_HELPER: SUCCESS_LOCK");
            let _ = std::io::stdout().flush();
            
            // Mantenemos el proceso vivo para sostener el lock
            thread::sleep(Duration::from_millis(sleep_ms));
            
            println!("LOCK_HELPER: Releasing lock...");
            let _ = std::io::stdout().flush();
            std::process::exit(0);
        }
        Err(err) => {
            println!("LOCK_HELPER: FAILED_LOCK: {}", err);
            let _ = std::io::stdout().flush();
            std::process::exit(2);
        }
    }
}
