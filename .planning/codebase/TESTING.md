# Testing Strategy

This document describes the testing patterns and coverage within the `phantom` codebase.

## Unit Testing
- **Co-located Tests**: Unit tests are located within the source files they test, inside `#[cfg(test)] mod tests` blocks.
- **Core Modules**: Comprehensive tests exist for:
    - `src/ftle/ftle.rs`: Math validation (embedding, exclusion).
    - `src/ftle/echo_state.rs`: Reservoir computing validation (spectral radius, state updates).
    - `src/ftle/embedding.rs`: Time-series transformation checks.

## Property-Based Testing
- **Proptest**: The `proptest` crate is used to validate mathematical invariants against a wide range of inputs, particularly in the `ftle` modules.

## Benchmarking
- **Criterion**: High-performance components (like FTLE calculation) use `criterion` for precise performance measurement. Benchmarks should reside in a `benches/` directory (to be implemented).

## Integration Testing
- **Current State**: Dedicated integration tests in a `tests/` directory are currently **missing**.
- **Requirement**: Future signals and detector integrations must include E2E tests simulating data flow from `src/signals/` to `src/detectors/`.

## Examples
- **Current State**: The `examples/` directory is defined in `Cargo.toml` but is currently **empty**.
- **Requirement**: Practical usage examples for the FTLE engine and signal detectors should be added to demonstrate system capabilities.
