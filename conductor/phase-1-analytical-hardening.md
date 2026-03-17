# Phase 1: Analytical Hardening Plan

## Goal
Achieve numerical stability and mathematical robustness across the codebase's analytical components (specifically FTLE and ESN).

## Requirements
- MATH-01: Numerical stability in ESN training.
- MATH-02: Robust linear algebra operations (SVD).
- MATH-03: Lorenz attractor validation.
- MATH-04: Functional examples for key detectors.

## Success Criteria
1. `cargo build` and `cargo test --lib` pass cleanly without warnings or errors.
2. ESN linear solver correctly handles ill-conditioned matrices using SVD.
3. FTLE/ESN predictions on the Lorenz attractor match ground truth within a 5% error margin.
4. Functional examples for `argus_anomaly.rs` and `kalshi_edge.rs` exist and run without issue.

## Tasks

### 1. Robust Linear Solver for ESN
- **Target**: `src/ftle/echo_state.rs`
- **Current State**: The `solve_linear_system` method uses `nalgebra::DMatrix::lu()`, which is unstable for ill-conditioned or singular matrices.
- **Action**: Replace the LU decomposition with an SVD-based pseudo-inverse (using `nalgebra::DMatrix::svd(true, true)`). This ensures robust calculation of output weights (`w_out`) during ridge regression.
- **Verification**: Add a unit test with a nearly singular matrix to ensure the solver converges.

### 2. Lorenz Attractor Validation
- **Target**: `src/ftle/` and tests.
- **Action**: 
  - Implement a numerical integrator (e.g., Runge-Kutta 4th order) for the Lorenz equations (`dx/dt = sigma*(y - x)`, `dy/dt = x*(rho - z) - y`, `dz/dt = x*y - beta*z`).
  - Generate a ground-truth dataset from the Lorenz system.
  - Create a dedicated test in `src/ftle/echo_state.rs` (or a new integration test) that trains the ESN on the Lorenz time-series and asserts that the closed-loop prediction error is `< 5%`.
- **Verification**: Assert prediction error `< 0.05` across a 100-step horizon.

### 3. Implementation of Functional Examples
- **Target**: `examples/argus_anomaly.rs`
  - **Action**: Write an example demonstrating anomaly detection using Argus detectors and the `EchoStateNetwork`. It should generate or load synthetic data, feed it into the detector, and print anomaly scores.
- **Target**: `examples/kalshi_edge.rs`
  - **Action**: Write an example demonstrating prediction market analysis using the `kalshi` signals module. Fetch/mock prediction market data and evaluate the prediction edge using FTLE/ESN.
- **Verification**: Ensure `cargo run --example argus_anomaly` and `cargo run --example kalshi_edge` execute successfully.

### 4. Code Quality & Precision Assurance
- **Action**: Ensure numerical thresholds are set appropriately in tests using epsilon checks (`abs(a - b) < 1e-6`).
- **Action**: Run `cargo check`, `cargo clippy`, and `cargo test --lib` continuously during development to guarantee strict compliance.
- **Verification**: Zero warnings/errors from the toolchain.

## Execution Strategy
1. Modify `src/ftle/echo_state.rs` solver logic to use SVD.
2. Implement `lorenz` generator for ground-truth data.
3. Add the Lorenz ESN test and calibrate parameters (spectral radius, ridge param) to reach < 5% error.
4. Flesh out the two required examples.
5. Final verification via the Rust toolchain commands.
