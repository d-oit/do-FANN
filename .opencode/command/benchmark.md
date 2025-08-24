---
description: Run performance benchmarks and analyze results
agent: build

---

Run comprehensive performance benchmarks across all projects and analyze results.

Benchmark categories:

1. **Neural Network Performance**:
   - Training speed benchmarks: `!cargo bench --bench neural_benchmarks`
   - Inference latency: `!cargo bench --bench inference_benchmarks`
   - Memory usage patterns: `!cargo bench --bench memory_benchmarks`

2. **WebAssembly Performance**:
   - WASM compilation time: `!time wasm-pack build cuda-wasm --release`
   - Runtime performance: `!node benchmarks/wasm-performance.js`
   - Bundle size analysis: `!npx bundlesize cuda-wasm/pkg/*.js`

3. **System Integration**:
   - Cross-project communication: `!cargo bench --bench integration_benchmarks`
   - Swarm coordination performance: `!cargo bench --bench swarm_benchmarks`
   - GPU acceleration benchmarks: `!cargo bench --bench gpu_benchmarks`

4. **Memory and CPU Usage**:
   - Peak memory consumption: `!cargo build --release && /usr/bin/time -v ./target/release/ruv-fann`
   - CPU utilization patterns: `!perf stat cargo build --release`
   - Cache performance: `!valgrind --tool=cachegrind ./target/release/ruv-fann`

Analyze results and identify:

- Performance regressions
- Memory leaks
- CPU bottlenecks
- Optimization opportunities

Provide specific recommendations for performance improvements.
