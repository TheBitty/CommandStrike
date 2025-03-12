use anyhow::Result;
use colored::Colorize;
use command_strike::llm::{OllamaClient, OllamaConfig, HistoryItem, check_ollama_running, validate_model};
use std::io::{self, Write};
use tokio::time::Instant;
use env_logger::Env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    // Header
    println!("{}", "CommandStrike - CTF Assistant".green().bold());
    println!("{}", "================================".green());
    
    // Check if Ollama is running
    println!("Checking if Ollama is running...");
    if !check_ollama_running().await {
        println!("{}", "Error: Ollama is not running. Please start Ollama first.".red().bold());
        println!("You can start Ollama with: ollama serve");
        return Ok(());
    }
    println!("{}", "✓ Ollama is running".green());
    
    // Validate Gemma model
    let model = "gemma3:12b";
    println!("Checking if model '{}' is available...", model);
    if !validate_model(model).await? {
        println!("{}", format!("Error: Model '{}' is not available.", model).red().bold());
        println!("You can pull it with: ollama pull {}", model);
        return Ok(());
    }
    println!("{}", format!("✓ Model '{}' is available", model).green());
    
    // Initialize Ollama client
    let config = OllamaConfig {
        model: model.to_string(),
        temperature: 0.7,
        ..OllamaConfig::default()
    };
    
    let client = OllamaClient::with_config(config)?;
    println!("{}", "Ready to assist with CTF challenges!".green());
    
    // Store command history
    let mut history: Vec<HistoryItem> = Vec::new();
    
    // Main interaction loop
    loop {
        print!("\n{}> ", "CommandStrike".cyan().bold());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "exit" || input == "quit" {
            break;
        }
        
        // Generate command
        let start = Instant::now();
        println!("Generating command...");
        
        match client.generate_command(input, &history).await {
            Ok(command) => {
                let elapsed = start.elapsed();
                println!("\n{}: {}", "Generated Command".green().bold(), command);
                println!("Generation time: {:.2}s", elapsed.as_secs_f32());
                
                // Ask user if they want to execute this command
                println!("\nWould you like to:");
                println!("1. Execute this command (simulation only)");
                println!("2. Explain what this command does");
                println!("3. Skip and enter a new request");
                
                print!("Choice [1-3]: ");
                io::stdout().flush()?;
                
                let mut choice = String::new();
                io::stdin().read_line(&mut choice)?;
                
                match choice.trim() {
                    "1" => {
                        // Simulate command execution
                        println!("{}", "Simulating command execution...".yellow().italic());
                        let simulated_output = format!("Command '{}' executed successfully.\nThis is simulated output - in a real implementation, the command would be executed with proper safeguards.", command);
                        println!("{}", simulated_output);
                        
                        // Add to history
                        history.push(HistoryItem {
                            user_input: input.to_string(),
                            command: command.clone(),
                            result: simulated_output.to_string(),
                        });
                        
                        // Interpret results
                        println!("\nInterpreting results...");
                        match client.interpret_result(&simulated_output, &history).await {
                            Ok(interpretation) => {
                                println!("\n{}", "Interpretation:".green().bold());
                                println!("{}", interpretation);
                            },
                            Err(e) => {
                                println!("{}: {}", "Error interpreting results".red().bold(), e);
                            }
                        }
                    },
                    "2" => {
                        println!("Explaining command...");
                        let prompt = format!("Explain in detail what this command does and its security implications: {}", command);
                        let system = "You are CommandStrike, a cybersecurity assistant specializing in CTF challenges. Explain commands in detail, breaking down each part and explaining security implications.";
                        
                        let start = Instant::now();
                        match client.stream_response(&prompt, Some(system)).await {
                            Ok(mut stream) => {
                                println!("\n{}", "Explanation:".green().bold());
                                
                                // Print streaming response
                                while let Some(chunk) = stream.receiver.recv().await {
                                    print!("{}", chunk);
                                    io::stdout().flush()?;
                                }
                                println!("\n");
                                
                                let elapsed = start.elapsed();
                                println!("Explanation time: {:.2}s", elapsed.as_secs_f32());
                            },
                            Err(e) => {
                                println!("{}: {}", "Error".red().bold(), e);
                            }
                        }
                    },
                    _ => println!("Skipping to next request"),
                }
            },
            Err(e) => {
                println!("{}: {}", "Error generating command".red().bold(), e);
            }
        }
    }
    
    println!("Thank you for using CommandStrike!");
    Ok(())
}