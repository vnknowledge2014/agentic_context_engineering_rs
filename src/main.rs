// ACE System - Main Entry Point
mod ace;
mod functional_core;
mod imperative_shell;
mod types;

use ace::ACEFramework;
use futures::StreamExt;
use imperative_shell::{log_error, log_info, log_success};
use std::io::{self, Write};
use types::OllamaConfig;

async fn demo_mode(ace: &mut ACEFramework) {
    log_info("ACE Demo Mode - Agentic Context Engineering");

    let queries = vec![
        "Agentic Context Engineering là gì?",
        "Viết Rust function tính fibonacci",
        "Phân tích ưu nhược điểm của ACE framework",
    ];

    for (i, query) in queries.iter().enumerate() {
        println!("\n{}", "=".repeat(60));
        println!("Query {}: {}", i + 1, query);
        println!("{}", "=".repeat(60));

        print!("\n🤖 Response:\n");
        io::stdout().flush().unwrap();

        match ace.process_query_stream(query).await {
            Ok(mut stream) => {
                let mut full_response = String::new();
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => {
                            print!("{}", chunk);
                            full_response.push_str(&chunk);
                            io::stdout().flush().unwrap();
                        }
                        Err(e) => {
                            log_error(&format!("Stream error: {}", e));
                            break;
                        }
                    }
                }
                println!();
                
                // Learn from interaction
                ace.learn_from_interaction(query, &full_response).await;
            }
            Err(e) => log_error(&format!("Query failed: {}", e)),
        }

        let stats = ace.get_context_stats();
        println!(
            "\n📈 Context: {} bullets, version {}\n",
            stats.total_bullets, stats.version
        );
    }
}

async fn interactive_mode(ace: &mut ACEFramework) {
    log_info("ACE Interactive Mode");
    println!("\nCommands: 'stats', 'help', 'exit'");
    println!("{}", "-".repeat(60));

    let stdin = io::stdin();
    loop {
        print!("\n👤 You: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match input {
            "exit" | "quit" => {
                log_info("Goodbye!");
                break;
            }
            "stats" => {
                let stats = ace.get_context_stats();
                println!("\n📊 Context Statistics:");
                println!("  Total bullets: {}", stats.total_bullets);
                println!("  Helpful bullets: {}", stats.helpful_bullets);
                println!("  Version: {}", stats.version);
                println!("  Avg helpfulness: {:.2}", stats.avg_helpfulness);
            }
            "help" => {
                println!("\n📖 ACE Framework Help");
                println!("  - Ask any question naturally");
                println!("  - 'stats' - Show context statistics");
                println!("  - 'exit' - Exit system");
            }
            _ => {
                print!("\n🤖 ACE:\n");
                io::stdout().flush().unwrap();

                match ace.process_query_stream(input).await {
                    Ok(mut stream) => {
                        let mut full_response = String::new();
                        while let Some(result) = stream.next().await {
                            match result {
                                Ok(chunk) => {
                                    print!("{}", chunk);
                                    full_response.push_str(&chunk);
                                    io::stdout().flush().unwrap();
                                }
                                Err(e) => {
                                    log_error(&format!("Stream error: {}", e));
                                    break;
                                }
                            }
                        }
                        println!();

                        // Learn from this interaction
                        ace.learn_from_interaction(input, &full_response).await;

                        let stats = ace.get_context_stats();
                        if stats.total_bullets > 0 {
                            println!("💡 Context: {} bullets learned", stats.total_bullets);
                        }
                    }
                    Err(e) => log_error(&format!("Error: {}", e)),
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = if args.len() > 1 && args[1] == "demo" {
        "demo"
    } else {
        "interactive"
    };

    let config = OllamaConfig::default();
    let mut ace = ACEFramework::new(config);

    match ace.initialize().await {
        Ok(_) => {}
        Err(e) => {
            log_error(&format!("Failed to initialize: {}", e));
            return;
        }
    }

    if mode == "demo" {
        demo_mode(&mut ace).await;
    } else {
        interactive_mode(&mut ace).await;
    }

    log_success("ACE Framework shutdown complete");
}
