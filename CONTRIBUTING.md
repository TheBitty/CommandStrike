# Contributing to CommandStrike

Thank you for considering contributing to CommandStrike! This document outlines the guidelines for contributing to this project.

## Development Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests to ensure they pass (`cargo test`)
5. Run lints to ensure code quality (`cargo clippy`)
6. Format your code (`cargo fmt`)
7. Commit your changes (see commit message guidelines below)
8. Push to your branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## Code Style

This project follows the Rust style guidelines:

- Use `cargo fmt` to format your code
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use meaningful variable and function names
- Keep functions small and focused on a single responsibility
- Add documentation comments to public APIs

## Testing

- Write tests for all new functionality
- Ensure all tests pass before submitting a PR
- Include integration tests for new features where appropriate

## Commit Message Guidelines

Follow the conventional commits specification:

```
<type>(<scope>): <short summary>

<body>

<footer>
```

Types:
- feat: A new feature
- fix: A bug fix
- docs: Documentation changes
- style: Code style changes (formatting, etc.)
- refactor: Code refactoring without functionality changes
- perf: Performance improvements
- test: Tests additions or corrections
- chore: Build process or tooling changes

Example:
```
feat(cli): add support for configuration files

Add ability to read configuration from YAML files.
Configuration values override command-line options.

Closes #123
```

## Pull Request Process

1. Update the README.md with details of changes if appropriate
2. Update the CHANGELOG.md with details of changes
3. The PR requires approval from at least one maintainer
4. Once approved, a maintainer will merge the PR

## Code of Conduct

- Be respectful and inclusive
- Value constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

## Questions?

If you have any questions about contributing, please open an issue with your question. 