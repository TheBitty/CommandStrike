# Cursor AI Rules for CommandStrike Rust Project

This document provides guidelines on how to effectively use Cursor's AI features when working on the CommandStrike Rust project. Following these practices will help maintain code quality and consistency.

## 1. Code Generation and Modification

- **Idiomatic Rust**: Request code that follows the [Rust API guidelines](https://rust-lang.github.io/api-guidelines/). Specify "idiomatic Rust" in your prompts.
- **Error Handling**: Ask for proper Rust error handling using `Result<T, E>` and `Option<T>` types rather than exceptions or null values.
- **Memory Safety**: Request code that respects Rust's ownership model. Ask the AI to avoid `unsafe` code unless absolutely necessary and with clear justification.
- **Type Definitions**: When creating new types or structs, specify which traits to derive (e.g., `Debug`, `Clone`, `PartialEq`).
- **Lifetimes**: Ask for explicit lifetime annotations when appropriate, especially in struct and function definitions that use references.

## 2. Project Structure

- **Modular Architecture**: Request code organized into appropriate modules following Rust conventions (e.g., `mod utils;` in `lib.rs`).
- **Cargo Structure**: Ensure responses follow Cargo's workspace structure when adding packages or dependencies.
- **Documentation**: Ask for rustdoc-style documentation (`///`) for public APIs with examples.
- **Unit Tests**: Request inline tests using Rust's `#[test]` attribute where appropriate.
- **Benchmarks**: For performance-critical code, ask for benchmark tests using `#[bench]` attribute.

## 3. AI Query Patterns

- **Specify Dependencies**: When asking to add functionality, name which crates you'd prefer to use (e.g., "using clap for CLI parsing").
- **Request Testing**: Ask the agent to include unit tests for any new functionality.
- **Error Message Help**: When sharing compiler errors, ask for explanations of ownership/borrowing concepts.
- **Code Reviews**: Use prompts like "Review this Rust code for idiomatic patterns and potential improvements".
- **Specific Function Signatures**: Provide the expected function signature when asking for implementations.

## 4. Agent Reference Guidelines

- **Standard Library First**: The agent should prioritize solutions from the Rust standard library before suggesting external crates.
- **Performance Considerations**: The agent should highlight when code might have performance implications.
- **Cross-Platform Compatibility**: The agent should note when code might behave differently on different platforms.
- **Future Compatibility**: Code should avoid deprecated features and prepare for future Rust releases where reasonable.

## 5. Common Prompts

When working with this project, these are useful prompts for the agent:

- "Explain this Rust code with focus on ownership and borrowing"
- "Refactor this code to use more idiomatic Rust patterns"
- "Add appropriate error handling to this function using anyhow/thiserror"
- "Show me how to implement [feature] using the clap crate"
- "Write tests for this module following Rust testing conventions"
- "Add documentation to this struct following rustdoc conventions"
- "Optimize this function for performance"
- "Make this code more readable while maintaining functionality"

## 6. Commit Conventions

When having the AI generate commit messages, use the following format:

```
<type>(<scope>): <short summary>

<body>

<footer>
```

Common types:
- feat: A new feature
- fix: A bug fix
- docs: Documentation changes
- style: Code style changes (formatting, etc.)
- refactor: Code refactoring without functionality changes
- perf: Performance improvements
- test: Tests additions or corrections
- chore: Build process or tooling changes 