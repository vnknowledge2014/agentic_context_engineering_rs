// ACE System - Main Entry Point
mod ace;
mod functional_core;
mod imperative_shell;
mod tools;
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
    println!("\nCommands: 'stats', 'help', 'exit', '/think', '/search', '/research', '/thinking on|off'");
    println!("{}", "-".repeat(60));

    let mut thinking_mode = false;
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
                println!("  - '/think <query>' - Deep thinking mode");
                println!("  - '/search <query>' - Search in context");
                println!("  - '/research <topic>' - Deep research mode");
                println!("  - '/thinking on|off' - Toggle native thinking mode");
                println!("  - 'exit' - Exit system");
            }
            _ if input.starts_with("/thinking ") => {
                let mode = &input[10..].trim().to_lowercase();
                match mode.as_str() {
                    "on" => {
                        thinking_mode = true;
                        log_success("Native thinking mode enabled");
                    }
                    "off" => {
                        thinking_mode = false;
                        log_success("Native thinking mode disabled");
                    }
                    _ => log_error("Use: /thinking on or /thinking off"),
                }
            }
            _ if input.starts_with("/think ") => {
                let query = &input[7..];
                print!("\n🧠 Thinking:\n");
                match ace.think(query).await {
                    Ok(result) => println!("{}", result),
                    Err(e) => log_error(&format!("Error: {}", e)),
                }
            }
            _ if input.starts_with("/search ") => {
                let query = &input[8..];
                let result = ace.search_query(query);
                println!("\n🔍 {}", result);
            }
            _ if input.starts_with("/research ") => {
                let topic = &input[10..];
                print!("\n🔬 Researching:\n");
                match ace.research(topic).await {
                    Ok(result) => println!("{}", result),
                    Err(e) => log_error(&format!("Error: {}", e)),
                }
            }
            _ => {
                print!("\n🤖 ACE:\n");
                io::stdout().flush().unwrap();

                let stream_result = if thinking_mode {
                    ace.generator.client.generate_stream_with_thinking(input, true).await
                } else {
                    ace.process_query_stream(input).await
                };

                match stream_result {
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
                        if !thinking_mode {
                            ace.learn_from_interaction(input, &full_response).await;
                        }

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
