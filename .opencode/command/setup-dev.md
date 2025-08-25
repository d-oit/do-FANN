---
description: Set up development environment for ruv-FANN
agent: orchestrator
---

Set up a complete development environment for the ruv-FANN workspace.

Current environment status:
`!which rustc && rustc --version`
`!which node && node --version`
`!which npm && npm --version`

Required setup steps:

1. **Rust Development Setup**:
   - Install WASM targets: `!rustup target add wasm32-unknown-unknown wasm32-wasi`
   - Install development tools: `!cargo install cargo-watch cargo-audit cargo-outdated`
   - Set up rust-analyzer: `!rustup component add rust-analyzer`

2. **WebAssembly Development**:
   - Install wasm-pack: `!curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
   - Install WASM test tools: `!cargo install wasm-pack`

3. **Node.js/JavaScript Setup**:
   - Install workspace dependencies: `!npm install`
   - Install development tools: `!npm install -g typescript eslint prettier`

4. **Testing Framework Setup**:
   - Install test tools: `!cargo install cargo-nextest cargo-tarpaulin`
   - Set up test coverage tools

5. **Development Tools**:
   - Install performance tools: `!cargo install flamegraph`
   - Install documentation tools: `!cargo install mdbook`

Verify the setup by checking:
- `!cargo check --workspace`
- `!cargo test --workspace --lib`
- `!npm run build`

Provide specific troubleshooting for any setup issues found.
