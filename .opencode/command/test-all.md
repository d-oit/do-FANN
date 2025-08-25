---
description: Run comprehensive tests across all projects
agent: orchestrator
---

Run the complete test suite across all projects in the ruv-FANN workspace.

Test execution order:

1. **Core Rust tests**: `!cargo test --workspace --lib`
2. **Integration tests**: `!cargo test --workspace --test integration`
3. **GPU/WebGPU tests**: `!cargo test --workspace --features gpu`
4. **WASM tests**: `!wasm-pack test cuda-wasm --headless --chrome`
5. **NPM package tests**: `!npm test --workspace=**`
6. **Performance benchmarks**: `!cargo bench --workspace`

Analyze test results and identify:

- Failing tests and their root causes
- Performance regressions
- Coverage gaps
- Integration issues between projects

Provide specific recommendations for fixing any failures.
