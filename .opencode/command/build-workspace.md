---
description: Build all projects in the workspace with dependency order
agent: build

---

Build all projects in the ruv-FANN workspace in the correct dependency order.

Current workspace status:
`!cargo tree --workspace`

Build the projects in this order:

1. ruv-fann (core library)
2. neuro-divergent (neural models)
3. cuda-wasm (WebAssembly bindings)
4. ruv-swarm (swarm coordination)
5. opencv-rust (computer vision)

Check for any build errors and suggest fixes. Focus on dependency resolution and compilation optimization.
