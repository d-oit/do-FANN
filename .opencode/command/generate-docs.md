---
description: Generate comprehensive documentation for all projects
agent: build

---

Generate comprehensive documentation for the entire ruv-FANN workspace.

Documentation to generate:

1. **API Documentation**:
   - Rust API docs: `!cargo doc --workspace --open`
   - JavaScript/TypeScript bindings: `!npx typedoc cuda-wasm/src/lib.rs`
   - Python bindings: `!pdoc neuro-divergent/python/`

2. **Architecture Documentation**:
   - System architecture diagrams
   - Component interaction flows
   - Data flow diagrams
   - Deployment architectures

3. **Usage Examples**:
   - Basic neural network training
   - WebAssembly integration
   - Swarm coordination examples
   - Performance optimization guides

4. **Development Guides**:
   - Contributing guidelines
   - Development setup
   - Testing procedures
   - Release process

Analyze the current documentation state:
`!find . -name "*.md" -o -name "*.rst" -o -name "*.txt" | head -20`

Identify gaps and create comprehensive documentation that covers:

- Installation and setup
- API usage patterns
- Architecture decisions
- Performance characteristics
- Troubleshooting guides
