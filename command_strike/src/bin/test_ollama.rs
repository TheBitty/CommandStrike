use anyhow::Result;
use colored::Colorize;
use command_strike::llm::{OllamaClient, OllamaConfig, HistoryItem, check_ollama_running, validate_model};
use std::io::{self, Write};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the OllamaClient with default settings (gemma3:12b)
    println!("{}", "CommandStrike Ollama Integration Test".green().bold());
    println!("Checking if Ollama is running...");
    
    if !check_ollama_running().await {
        println!("{}", "Error: Ollama is not running. Please start Ollama first.".red().bold());
        println!("You can start Ollama with: ollama serve");
        return Ok(());
    }
    
    println!("{}", "✓ Ollama is running".green());
    
    // Validate that gemma3:12b model is available
    let model = "gemma3:12b";
    println!("Checking if model '{}' is available...", model);
    
    if !validate_model(model).await? {
        println!("{}", format!("Error: Model '{}' is not available.", model).red().bold());
        println!("You can pull it with: ollama pull {}", model);
        return Ok(());
    }
    
    println!("{}", format!("✓ Model '{}' is available", model).green());

    // Create client with custom configuration
    let config = OllamaConfig {
        model: model.to_string(),
        temperature: 0.5,  // Lower for more deterministic responses
        max_tokens: 2048,
        ..OllamaConfig::default()
    };
    
    let client = OllamaClient::with_config(config)?;
    println!("{}", "OllamaClient initialized successfully".green());
    
    // Store command history
    let mut history: Vec<HistoryItem> = Vec::new();
    
    // Enter interactive mode
    println!("\n{}", "=== CommandStrike Interactive Mode ===".yellow().bold());
    println!("Enter security tasks in natural language. Type 'exit' to quit.");
    
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
        
        // Time the response generation
        let start = Instant::now();
        println!("Generating command...");
        
        // Generate command
        match client.generate_command(input, &history).await {
            Ok(command) => {
                let elapsed = start.elapsed();
                println!("\n{}: {}", "Generated Command".green().bold(), command);
                println!("({:.2}s)", elapsed.as_secs_f32());
                
                // Ask user if they want to execute this command (in a real app)
                println!("\nWould you like to:");
                println!("1. Execute this command (simulation only in this demo)");
                println!("2. Explain what this command does");
                println!("3. Skip and enter a new request");
                
                print!("Choice [1-3]: ");
                io::stdout().flush()?;
                
                let mut choice = String::new();
                io::stdin().read_line(&mut choice)?;
                
                match choice.trim() {
                    "1" => {
                        // Simulate command execution (in the real app, you'd use the executor module)
                        println!("{}", "Simulating command execution...".yellow().italic());
                        let simulated_output = "Command executed successfully. This is simulated output.";
                        println!("{}", simulated_output);
                        
                        // Add to history
                        history.push(HistoryItem {
                            user_input: input.to_string(),
                            command: command.clone(),
                            result: simulated_output.to_string(),
                        });
                    },
                    "2" => {
                        // Demonstrate streaming response for command explanation
                        println!("{}", "Streaming explanation:".yellow());
                        
                        // Stream the explanation
                        let prompt = format!("Explain what this command does: {}", command);
                        let system = "You are CommandStrike, a security tool assistant. Explain what commands do in a security context, detailing each part of the command.";
                        
                        let mut stream = client.stream_response(&prompt, Some(system)).await?;
                        
                        // Print streaming response
                        while let Some(chunk) = stream.receiver.recv().await {
                            print!("{}", chunk);
                            io::stdout().flush()?;
                        }
                        println!("\n");
                    },
                    _ => println!("Skipping to next request"),
                }
            },
            Err(e) => {
                println!("{}: {}", "Error".red().bold(), e);
            }
        }
    }
    
    println!("Thanks for using CommandStrike!");
    Ok(())
} 