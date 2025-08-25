---
description: Build and optimize WebAssembly components
agent: orchestrator

---

Build and optimize all WebAssembly components in the workspace.

Build targets:

1. **CUDA WASM**: `!wasm-pack build cuda-wasm --target web --out-dir pkg --dev`
2. **OpenCV WASM**: `!wasm-pack build opencv-rust/opencv-wasm --target web`
3. **Neural WASM**: `!wasm-pack build neuro-divergent --target web --features wasm`

Optimization steps:

1. **Size optimization**: `!wasm-opt -Oz cuda-wasm/pkg/*.wasm -o cuda-wasm/pkg/*.optimized.wasm`
2. **Bundle analysis**: `!npx webpack-bundle-analyzer cuda-wasm/pkg/package.json`
3. **Performance profiling**: `!cargo build --target wasm32-unknown-emscripten --release`

Check for:

- Bundle size and loading performance
- WebAssembly feature compatibility
- JavaScript bindings quality
- Browser compatibility issues

Suggest optimizations for smaller bundle sizes and better performance.
