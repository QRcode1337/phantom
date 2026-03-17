# Codebase Conventions

This document outlines the coding standards, idioms, and patterns used in the `phantom` codebase.

## Language and Edition
- **Rust 2021 Edition**: The project strictly adheres to the latest stable Rust features.
- **Strict Linting**: Expected use of `clippy` for maintaining idiomatic code quality.

## Error Handling
- **Application Level**: `anyhow::Result` is used for high-level application logic and CLI entry points (e.g., `src/main.rs`).
- **Library Level**: `thiserror` is preferred for defining structured, domain-specific error types in library modules (e.g., `src/ftle/mod.rs`).

## Math and Parallelism
- **Linear Algebra**: `ndarray` and `nalgebra` are the primary engines for matrix operations and chaos math.
- **Parallelism**: `rayon` is used for data-parallel tasks, particularly in FTLE calculations and ensemble processing.
- **Async Runtime**: `tokio` is used for handling concurrent signals and I/O-bound tasks (e.g., `reqwest`).

## Logging and Instrumentation
- **Tracing**: The `tracing` crate is used for structured logging and performance profiling.
- **Subscribers**: `tracing-subscriber` is configured in `main.rs` for output formatting.

## Code Style
- **Naming**: Standard Rust `snake_case` for functions/modules/variables and `PascalCase` for types/traits.
- **Documentation**: Public APIs must be documented using `///` doc comments. Modules should include `//!` overview comments.
- **Formatting**: `rustfmt` is the authoritative source for code layout.
