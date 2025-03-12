# CommandStrike

CommandStrike is a Rust-based CTF assistant that translates natural language security requests into shell commands using a local LLM (Gemma 3 12B running on Ollama), executes these commands, and provides explanations of the results.

## Features

- Natural language to shell command translation
- Command explanation with security context
- Conversation history tracking
- Streaming responses for real-time feedback
- Interactive CLI interface with colored output

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2021 edition or later)
- [Ollama](https://ollama.ai/download) for running local LLMs
- Gemma 3 12B model (install using `ollama pull gemma3:12b`)

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

3. Enter natural language security requests at the prompt:
   ```
   CommandStrike> scan open ports on localhost
   ```

4. CommandStrike will generate a command, which you can:
   - Execute (simulated in this version)
   - Get a detailed explanation
   - Skip and try another request

## Example Commands

- "Find all files containing the word 'password' in the current directory"
- "Scan the network for vulnerable web servers"
- "Show all running processes and their ports"
- "Check for privilege escalation vulnerabilities"

## Architecture

CommandStrike consists of the following components:

- **LLM Integration**: API client for Ollama (Gemma 3 12B)
- **Command Executor**: Simulated shell command execution
- **Context Manager**: Maintains history between commands
- **CLI Interface**: Interactive terminal UI

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 