---
description: Neural network development and optimization assistant
agent: build
---

Assist with neural network development, optimization, and debugging.

Current neural network implementations:
`!find . -name "*.rs" -path "*/src/*" -exec grep -l "neural\\|network\\|layer\\|activation" {} \;`

Analyze the codebase for:

1. **Architecture patterns**: Layer implementations, activation functions, training loops
2. **Performance bottlenecks**: Memory usage, computation efficiency
3. **Code quality**: Error handling, documentation, testing coverage
4. **Optimization opportunities**: SIMD usage, GPU acceleration, memory layout

Specific areas to examine:

- FANN algorithm implementations in `src/`
- Neuro-divergent model architectures
- Training pipeline efficiency
- Memory management patterns

Provide recommendations for:

- Performance improvements
- Code structure enhancements
- Additional test coverage
- Documentation improvements
