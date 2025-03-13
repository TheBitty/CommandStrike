# CommandStrike

CommandStrike is a Rust-based CTF assistant that translates natural language security requests into shell commands using a local LLM running on Ollama, executes these commands, and provides explanations of the results.

## Features

- Natural language to shell command translation
- Command explanation with security context
- **Multiple LLM model support with runtime switching**
- **Pre-defined security command templates library**
- Conversation history tracking
- Streaming responses for real-time feedback
- Interactive CLI interface with colored output

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2021 edition or later)
- [Ollama](https://ollama.ai/download) for running local LLMs
- At least one of the following models installed in Ollama:
  - Gemma 3 12B model (`ollama pull gemma3:12b`)
  - DeepSeek R1 8B model (`ollama pull deepseek-r1:8b`)
  - DeepSeek Coder 6.7B (`ollama pull deepseek-coder:6.7b`)
  - Any other compatible model of your choice

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/command_strike.git
   cd command_strike
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Run CommandStrike:
   ```
   cargo run --bin command_strike
   ```

## Usage

1. Start Ollama in the background:
   ```
   ollama serve
   ```

2. Launch CommandStrike:
   ```
   cargo run --bin command_strike
   ```

3. Select your preferred LLM model from the menu or enter a custom model name

4. Enter natural language security requests at the prompt:
   ```
   CommandStrike> scan open ports on localhost
   ```

5. CommandStrike will generate a command, which you can:
   - Execute (simulated in this version)
   - Get a detailed explanation
   - Skip and try another request

## Advanced Commands

CommandStrike provides several special commands:

- `switch` or `model` - Switch to a different LLM model during runtime
- `models` - View available and recommended models
- `templates` - Browse pre-defined security command templates by category
- `help` - Display help information and example requests
- `exit` or `quit` - Exit CommandStrike

## Security Command Templates

CommandStrike includes a comprehensive library of pre-defined security command templates organized by category:

- **Network Reconnaissance**: Host discovery, port scanning, service detection
- **Web Application**: Directory enumeration, vulnerability scanning, SQL injection testing
- **Password Attacks**: Brute force tools, hash cracking, wordlist generation
- **Exploitation**: Reverse shells, payload generation
- **Post-Exploitation**: Privilege escalation checks, system information gathering
- **File Analysis**: Sensitive data search, binary analysis

Access these templates at any time by typing `templates` at the prompt.

## Example Security Requests

- "Find all files containing the word 'password' in the current directory"
- "Scan the network for vulnerable web servers"
- "Show all running processes and their ports"
- "Check for privilege escalation vulnerabilities"
- "Perform a directory traversal test on a web server at 192.168.1.10"
- "Generate a reverse shell payload for a Linux target"
- "Brute force SSH login for user admin on 10.0.0.1"

## Supported Models

CommandStrike recommends these models for security tasks:

| Model | Size | Description |
|-------|------|-------------|
| gemma3:12b | 12B | Google's Gemma 3 12B model, good general performance for security tasks |
| deepseek-coder:6.7b | 6.7B | Model focused on code analysis and generation, useful for exploit development |
| deepseek-r1:8b | 8B | Lightweight yet powerful reasoning model for security analysis |
| llama3:8b | 8B | Meta's Llama 3 8B model, good balance of performance and resource usage |
| phi3:14b | 14B | Microsoft's Phi-3 large model, excellent for complex security reasoning |
| mixtral:8x7b | 8x7B | Mistral AI's mixture of experts model, very strong on complex security tasks |

You can also use any other model available in Ollama.

## Architecture

CommandStrike consists of the following components:

- **LLM Integration**: API client for Ollama with multi-model support
- **Command Executor**: Simulated shell command execution
- **Context Manager**: Maintains history between commands
- **CLI Interface**: Interactive terminal UI with model selection
- **Templates Library**: Pre-defined security commands organized by category

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 