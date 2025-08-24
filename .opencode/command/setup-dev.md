---
description: Set up development environment for ruv-FANN
agent: build
---
Set up a complete development environment for the ruv-FANN workspace.

Development Environment Setup:

1. **Rust Development**:
   - Install required targets: `!rustup target add wasm32-unknown-emscripten wasm32-wasi`
   - Install development tools: `!cargo install cargo-watch cargo-audit cargo-outdated`
   - Set up rust-analyzer: `!rustup component add rust-analyzer`

2. **WebAssembly Development**:
   - Install wasm-pack: `!curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
   - Install emscripten: `!git clone https://github.com/emscripten-core/emsdk.git && cd emsdk && ./emsdk install latest && ./emsdk activate latest`
   - Set up WASM testing: `!npm install -g @webassemblyjs/cli`

3. **Node.js/JavaScript Development**:
   - Install dependencies: `!npm install --workspace=**`
   - Set up TypeScript: `!npx tsc --init --target es2020 --moduleResolution node`
   - Install development tools: `!npm install -g typescript eslint prettier`

4. **Testing Framework**:
   - Set up test runners: `!cargo install cargo-nextest`
   - Install WASM test runner: `!npm install -g wasm-pack`
   - Configure test coverage: `!cargo install cargo-tarpaulin`

5. **Development Tools**:
   - Install performance tools: `!cargo install flamegraph`
   - Set up benchmarking: `!cargo install criterion`
   - Install documentation tools: `!cargo install mdbook`

6. **IDE Configuration**:
   - Generate VS Code settings: Create `.vscode/settings.json` with Rust and WASM support
   - Set up launch configurations for debugging
   - Configure workspace-specific settings

Verify setup by running:

- `!cargo check --workspace`
- `!npm run build --workspace=**`
- `!cargo test --workspace --lib`

Provide troubleshooting for common setup issues.
