use std::io::{self, Write};
// Mock of the CLI REPL for MVP. In reality, we would use `rustyline` for a proper term.

#[tokio::main]
async fn main() {
    println!("IADBMS Interactive Shell v0.1.0");
    println!("Type '\\help' for commands, or write your query directly.");
    println!("Connecting to tcp://127.0.0.1:8080...");

    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:8080/api/v1/query";

    loop {
        print!("iadbms> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "\\q" || input == "\\quit" || input == "exit" {
            println!("Goodbye!");
            break;
        } else if input == "\\help" {
            println!("Commands:\n\\q     Quit\n\\help  Show help\n<query> Send physical query to daemon");
            continue;
        }

        // Send query to the REST API
        match client.post(url).json(&serde_json::json!({ "query": input })).send().await {
            Ok(response) => {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    let success = json["success"].as_bool().unwrap_or(false);
                    let data = json["data"].as_str().unwrap_or("");
                    if success {
                        println!("✅ SUCCESS\n{}", data);
                    } else {
                        println!("❌ ERROR\n{}", data);
                    }
                } else {
                    println!("Error parsing daemon response");
                }
            }
            Err(e) => {
                println!("Error communicating with daemon: {}", e);
            }
        }
    }
}
