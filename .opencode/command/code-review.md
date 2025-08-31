---
description: AI/ML focused code review for neural network implementations
agent: build

---

Perform a comprehensive code review focused on AI/ML best practices and neural network implementations.

Review areas specific to this codebase:

1. **Neural Network Implementation**:
   - Algorithm correctness in FANN implementations
   - Training loop efficiency and convergence
   - Memory management in neural computations
   - Numerical stability and precision handling

2. **Performance Considerations**:
   - SIMD utilization in core algorithms
   - GPU acceleration opportunities
   - Memory access patterns and cache efficiency
   - WebAssembly optimization potential

3. **Code Quality**:
   - Error handling in mathematical operations
   - Documentation of complex algorithms
   - Test coverage for neural components
   - Type safety in numerical computations

4. **Architecture Patterns**:
   - Component separation and modularity
   - Dependency injection patterns
   - Configuration management
   - Extensibility for new neural architectures

5. **Security Considerations**:
   - Input validation for neural network parameters
   - Safe memory handling in WASM context
   - Protection against numerical instabilities

Current files to review:
`!find src -name "*.rs" | head -10`

Focus on identifying:

- Performance bottlenecks in neural computations
- Memory safety issues in numerical code
- Algorithm correctness problems
- Code maintainability issues
- Documentation gaps for complex algorithms
