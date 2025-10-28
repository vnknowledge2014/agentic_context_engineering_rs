// ACE System - Main Entry Point
mod ace;
mod functional_core;
mod imperative_shell;
mod tools;
mod types;

use ace::ACEFramework;
use tools::{SearchTool, ThinkingTool, DeepResearchTool};
use futures::StreamExt;
use imperative_shell::{log_error, log_info, log_success};
use std::io::{self, Write};
use types::OllamaConfig;

async fn demo_mode(ace: &mut ACEFramework) {
    log_info("ACE Demo Mode - Testing All Features");
    println!("\n{}", "=".repeat(60));

    // 1. Basic ACE Query
    println!("\n🧪 Test 1: Basic ACE Query");
    println!("{}", "-".repeat(60));
    let query = "What is Agentic Context Engineering?";
    println!("Query: {}", query);
    print!("\n🤖 Response:\n");
    io::stdout().flush().unwrap();
    
    match ace.process_query_stream(query).await {
        Ok(mut stream) => {
            let mut full_response = String::new();
            while let Some(result) = stream.next().await {
                if let Ok(chunk) = result {
                    print!("{}", chunk);
                    full_response.push_str(&chunk);
                    io::stdout().flush().unwrap();
                }
            }
            println!();
            ace.learn_from_interaction(query, &full_response).await;
        }
        Err(e) => log_error(&format!("Error: {}", e)),
    }
    let stats = ace.get_context_stats();
    println!("📈 Context: {} bullets learned", stats.total_bullets);

    // 2. Context Learning
    println!("\n{}", "=".repeat(60));
    println!("\n🧪 Test 2: Context Learning");
    println!("{}", "-".repeat(60));
    let query = "Write a Rust function to calculate factorial";
    println!("Query: {}", query);
    print!("\n🤖 Response:\n");
    io::stdout().flush().unwrap();
    
    match ace.process_query_stream(query).await {
        Ok(mut stream) => {
            let mut full_response = String::new();
            while let Some(result) = stream.next().await {
                if let Ok(chunk) = result {
                    print!("{}", chunk);
                    full_response.push_str(&chunk);
                    io::stdout().flush().unwrap();
                }
            }
            println!();
            ace.learn_from_interaction(query, &full_response).await;
        }
        Err(e) => log_error(&format!("Error: {}", e)),
    }
    let stats = ace.get_context_stats();
    println!("📈 Context: {} bullets learned", stats.total_bullets);

    // 3. Search in Context
    println!("\n{}", "=".repeat(60));
    println!("\n🧪 Test 3: Search in Context");
    println!("{}", "-".repeat(60));
    let search_tool = SearchTool::new(false);
    let context = ace.curator.get_context();
    let results = search_tool.search_context("Rust", &context.bullets);
    println!("🔍 Search 'Rust': Found {} results", results.len());
    for (i, r) in results.iter().take(2).enumerate() {
        let preview: String = r.content.chars().take(60).collect();
        println!("  {}. {}...", i + 1, preview);
    }

    // 4. Thinking Mode
    println!("\n{}", "=".repeat(60));
    println!("\n🧪 Test 4: Deep Thinking");
    println!("{}", "-".repeat(60));
    let query = "Compare functional vs OOP";
    println!("Query: {}", query);
    println!("\n🧠 Thinking:");
    match ace.think(query).await {
        Ok(response) => {
            let preview: String = response.chars().take(200).collect();
            println!("{}...", preview);
        }
        Err(e) => log_error(&format!("Error: {}", e)),
    }

    // 5. Web Search
    println!("\n{}", "=".repeat(60));
    println!("\n🧪 Test 5: Web Search");
    println!("{}", "-".repeat(60));
    let search_tool_web = SearchTool::new(true);
    println!("🔍 Searching 'Rust programming'...");
    let web_results = search_tool_web.search("Rust programming", &context.bullets).await;
    println!("Found {} results (context + web)", web_results.len());
    for (i, r) in web_results.iter().take(2).enumerate() {
        let source = if r.source == "web" { "🌐" } else { "📚" };
        let preview: String = r.content.chars().take(60).collect();
        println!("  {}. {} {}...", i + 1, source, preview);
    }

    // 6. Deep Research
    println!("\n{}", "=".repeat(60));
    println!("\n🧪 Test 6: Deep Research");
    println!("{}", "-".repeat(60));
    let topic = "Functional Programming";
    println!("Topic: {}", topic);
    println!("\n🔬 Researching...");
    match ace.research(topic).await {
        Ok(report) => {
            let lines: Vec<&str> = report.lines().take(15).collect();
            println!("{}", lines.join("\n"));
            println!("...");
        }
        Err(e) => log_error(&format!("Error: {}", e)),
    }

    // Final Stats
    println!("\n{}", "=".repeat(60));
    println!("\n📊 Final Statistics");
    println!("{}", "-".repeat(60));
    let stats = ace.get_context_stats();
    println!("  Total bullets: {}", stats.total_bullets);
    println!("  Helpful bullets: {}", stats.helpful_bullets);
    println!("  Context version: {}", stats.version);
    println!("  Avg helpfulness: {:.2}", stats.avg_helpfulness);
    println!("\n✅ All tests completed!");
    println!("{}", "=".repeat(60));
}

async fn interactive_mode(ace: &mut ACEFramework) {
    log_info("ACE Interactive Mode");
    println!("\nCommands: 'stats', 'help', 'exit', '/think', '/search', '/research', '/thinking on|off', '/web on|off'");
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
                println!("  - '/search <query>' - Search in context/web");
                println!("  - '/research <topic>' - Deep research mode");
                println!("  - '/thinking on|off' - Toggle native thinking mode");
                println!("  - '/web on|off' - Toggle web search (like OpenAI)");
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
            _ if input.starts_with("/web ") => {
                let mode = &input[5..].trim().to_lowercase();
                match mode.as_str() {
                    "on" => {
                        ace.web_search_enabled = true;
                        log_success("🌐 Web search enabled (like OpenAI)");
                    }
                    "off" => {
                        ace.web_search_enabled = false;
                        log_success("Web search disabled");
                    }
                    _ => log_error("Use: /web on or /web off"),
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
                print!("\n🔍 Searching...\n");
                let result = ace.search_query(query).await;
                println!("{}", result);
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

                let stream_result = ace.process_query_stream(input).await;

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
