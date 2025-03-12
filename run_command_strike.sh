#!/bin/bash

# Change to the project directory
cd "$(dirname "$0")/command_strike"

# Check if Ollama is running
if ! curl -s http://localhost:11434/api/tags > /dev/null; then
    echo "Error: Ollama is not running. Please start Ollama with 'ollama serve'"
    exit 1
fi

# Set environment variables for more verbose logging
export RUST_LOG=debug

# Parse command line arguments
RELEASE=false
MODEL="gemma3:12b"

# Process command line arguments
for arg in "$@"; do
    case $arg in
        --release)
            RELEASE=true
            ;;
        --model=*)
            MODEL="${arg#*=}"
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --release         Run the release version (optimized)"
            echo "  --model=<name>    Specify the model to use (default: gemma3:12b)"
            echo "  --help            Show this help message"
            exit 0
            ;;
    esac
done

# Check if the specified model exists
if ! curl -s "http://localhost:11434/api/tags" | grep -q "\"name\":\"$MODEL\""; then
    echo "Warning: Model '$MODEL' may not be available. Available models:"
    curl -s "http://localhost:11434/api/tags" | grep "\"name\":" | sed 's/.*"name":"\([^"]*\)".*/  - \1/'
    
    echo ""
    echo "Do you want to continue anyway? (y/n)"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "Exiting. You can pull the model with: ollama pull $MODEL"
        exit 1
    fi
fi

# Choose which version to run
if [ "$RELEASE" = true ]; then
    echo "Running release version with model: $MODEL"
    RUST_BACKTRACE=1 cargo run --release --bin command_strike
else
    echo "Running debug version with model: $MODEL (use --release for optimized version)"
    RUST_BACKTRACE=1 cargo run --bin command_strike
fi 