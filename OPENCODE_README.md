# OpenCode Setup for ruv-fann Multi-Project Repository

This document provides comprehensive setup and usage instructions for the OpenCode configuration in the ruv-fann multi-project repository.

## 📋 Project Overview

The ruv-fann repository is a complex multi-project ecosystem containing:

- **ruv-fann** (Main): Pure Rust implementation of Fast Artificial Neural Network (FANN) library
- **cuda-wasm**: CUDA WebAssembly integration for GPU acceleration
- **ruv-swarm**: Multi-agent swarm coordination system
- **neuro-divergent**: Advanced neural network training and models
- **opencv-rust**: OpenCV bindings for Rust with computer vision capabilities

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust (1.81+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-emscripten wasm32-wasi

# Install Node.js (18.20.8+)
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install OpenCode (if not already installed)
curl -fsSL https://opencode.ai/install | bash
```

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/ruvnet/ruv-FANN.git
cd ruv-FANN

# Configure OpenCode (if not already configured)
opencode auth login

# Start OpenCode
opencode
```

## 🏗️ Project Structure

```
ruv-FANN/
├── opencode.json             # Main OpenCode configuration (official format)
├── OPENCODE_README.md        # This file
├── src/                      # Main Rust FANN library
├── cuda-wasm/               # CUDA WebAssembly project
├── ruv-swarm/               # Swarm coordination system
├── neuro-divergent/         # Neural network training
├── opencv-rust/             # OpenCV Rust bindings
├── tests/                   # Comprehensive test suite
└── examples/                # Usage examples
```

## 🤖 Available Agents

All agents are configured in `opencode.json` with specialized capabilities:

### Core Agents

1. **orchestrator** - Main project coordinator
    - Manages multi-project dependencies
    - Coordinates build and deployment
    - Tracks project progress
    - Uses Claude Sonnet 4 for complex reasoning

2. **rust-core** - Rust development specialist
    - Neural network algorithm implementation
    - Performance optimization
    - Memory management
    - Expert in Rust best practices

3. **wasm-engineer** - WebAssembly specialist
    - CUDA WASM integration
    - Cross-compilation
    - Browser optimization
    - Performance-focused development

4. **swarm-architect** - Multi-agent system architect
    - Swarm coordination design
    - MCP protocol implementation
    - Distributed systems
    - Advanced coordination patterns

5. **ml-researcher** - Machine learning researcher
    - Algorithm research and implementation
    - Model optimization
    - Training pipeline development
    - Research methodology expert

### Specialized Agents

6. **cv-engineer** - Computer vision specialist
    - OpenCV integration
    - Image processing
    - Feature extraction
    - Real-time performance optimization

7. **test-engineer** - Testing specialist
    - Unit and integration testing
    - Performance benchmarking
    - Cross-platform testing
    - Uses faster Haiku model for efficiency

8. **docs-specialist** - Documentation expert
    - API documentation
    - Tutorial creation
    - Technical writing
    - Uses faster Haiku model for efficiency

9. **performance-optimizer** - Performance specialist
    - Algorithm optimization
    - Memory and GPU optimization
    - Benchmarking
    - Performance analysis expert

## 🔧 Common Workflows

### Build All Projects

```bash
# Using OpenCode agents
opencode

# Then ask the orchestrator:
/ask orchestrator "Build all projects in the correct dependency order"

# Manual build process
cargo build --workspace
npm run build --workspace=**
wasm-pack build cuda-wasm
```

### Run Complete Test Suite

```bash
# Using OpenCode agents
opencode

# Then ask the test engineer:
/ask test-engineer "Run comprehensive tests across all projects"

# Manual testing
cargo test --workspace
npm test --workspace=**
cargo test --features test-all
```

### Neural Network Training

```bash
# Using OpenCode agents
opencode

# Then ask the ML researcher:
/ask ml-researcher "Set up and execute neural network training with GPU acceleration"

# This will:
# 1. Prepare training data
# 2. Design network architecture
# 3. Implement training loop
# 4. Set up GPU acceleration
# 5. Execute training
# 6. Validate model performance
```

### WebAssembly Deployment

```bash
# Using OpenCode agents
opencode

# Then ask the WASM engineer:
/ask wasm-engineer "Build and deploy WebAssembly components with optimizations"

# This will:
# 1. Optimize WASM build
# 2. Generate JavaScript bindings
# 3. Create web demo
# 4. Test browser compatibility
# 5. Optimize performance
```

### Documentation Generation

```bash
# Using OpenCode agents
opencode

# Then ask the docs specialist:
/ask docs-specialist "Generate comprehensive documentation for all projects"

# This will:
# 1. Analyze codebase
# 2. Generate API docs
# 3. Create tutorials
# 4. Document architecture
# 5. Create contribution guide
```

### Run Complete Test Suite

```bash
# Using OpenCode workflow
opencode workflow run test-pipeline

# Manual testing
cargo test --workspace
npm test --workspace=**
wasm-pack test cuda-wasm
```

### Neural Network Training

```bash
# Using OpenCode workflow
opencode workflow run neural-network-training

# This will:
# 1. Prepare training data
# 2. Design network architecture
# 3. Implement training loop
# 4. Set up GPU acceleration
# 5. Execute training
# 6. Validate model performance
```

### WebAssembly Deployment

```bash
# Using OpenCode workflow
opencode workflow run wasm-deployment

# This will:
# 1. Optimize WASM build
# 2. Generate JavaScript bindings
# 3. Create web demo
# 4. Test browser compatibility
# 5. Optimize performance
```

### Documentation Generation

```bash
# Using OpenCode workflow
opencode workflow run documentation-generation

# This will:
# 1. Analyze codebase
# 2. Generate API docs
# 3. Create tutorials
# 4. Document architecture
# 5. Create contribution guide
```

## 🛠️ Agent Usage Examples

### Working with the Rust Core Agent

```bash
# Start OpenCode
opencode

# Request neural network implementation
/ask rust-core "Implement a convolutional layer with batch normalization"

# Optimize existing code
/ask rust-core "Optimize the matrix multiplication in src/layers.rs for better cache performance"

# Add comprehensive tests
/ask rust-core "Add unit tests for the activation functions in src/activation.rs"
```

### Working with the WASM Engineer

```bash
# Start OpenCode
opencode

# Create WebAssembly bindings
/ask wasm-engineer "Create JavaScript bindings for the neural network library"

# Optimize WASM performance
/ask wasm-engineer "Optimize the WASM build for smaller bundle size"

# Debug WASM issues
/ask wasm-engineer "Debug the WebAssembly memory allocation issue in cuda-wasm"
```

### Working with the ML Researcher

```bash
# Start OpenCode
opencode

# Research new algorithms
/ask ml-researcher "Research and implement the latest attention mechanisms for neural networks"

# Optimize training
/ask ml-researcher "Optimize the training loop for better convergence on GPU"

# Analyze model performance
/ask ml-researcher "Analyze the performance bottlenecks in the current model architecture"
```

## 📊 Monitoring and Analytics

### Performance Monitoring

```bash
# View project metrics
opencode monitor performance

# Check build times
opencode monitor builds

# Monitor test coverage
opencode monitor coverage
```

### Error Tracking

```bash
# View recent errors
opencode monitor errors

# Check build failures
opencode monitor failures

# Generate error report
opencode monitor report
```

## 🔒 Security and Quality

### Code Quality Checks

```bash
# Run all linting tools
opencode quality lint

# Check security vulnerabilities
opencode quality security

# Generate quality report
opencode quality report
```

### Security Scanning

```bash
# Scan for vulnerabilities
opencode security scan

# Check dependencies
opencode security dependencies

# Generate security report
opencode security report
```

## 🚀 Deployment

### Crate Deployment

```bash
# Deploy Rust crates
opencode deploy crates

# Deploy specific crate
opencode deploy crate ruv-fann
```

### NPM Package Deployment

```bash
# Deploy NPM packages
opencode deploy npm

# Deploy specific package
opencode deploy npm ruv-swarm
```

### WebAssembly Deployment

```bash
# Deploy WASM modules
opencode deploy wasm

# Deploy to CDN
opencode deploy wasm --target cdn
```

## 📚 Advanced Usage

### Custom Agents

Create custom agents by editing `opencode.json`:

```json
{
  "agent": {
    "my-custom-agent": {
      "description": "Specialized agent for specific tasks",
      "model": "anthropic/claude-sonnet-4-20250514",
      "prompt": "Custom prompt template for the agent",
      "tools": {
        "read": true,
        "write": true,
        "edit": true,
        "bash": true
      }
    }
  }
}
```

### MCP Server Integration

Configure MCP servers in `opencode.json`:

```json
{
  "mcp": {
    "my-mcp-server": {
      "type": "local",
      "command": ["npx", "my-mcp-server"],
      "args": ["--config", "config.json"],
      "env": {
        "SERVER_MODE": "production"
      }
    }
  }
}
```

### Custom Formatters

Configure code formatters in `opencode.json`:

```json
{
  "formatter": {
    "custom-formatter": {
      "command": ["my-formatter", "--write", "$FILE"],
      "extensions": [".custom"],
      "environment": {
        "FORMATTER_CONFIG": "strict"
      }
    }
  }
}
```

## 🐛 Troubleshooting

### Common Issues

1. **Build Failures**
   ```bash
   # Check build logs
   opencode monitor builds --verbose

   # Retry with clean build
   opencode workflow run build-pipeline --clean
   ```

2. **Test Failures**
   ```bash
   # Run specific test suite
   opencode agent ask test-engineer "Debug the failing tests in src/layers.rs"

   # Generate test report
   opencode monitor tests --report
   ```

3. **WASM Issues**
   ```bash
   # Debug WASM compilation
   opencode agent ask wasm-engineer "Debug the WebAssembly compilation error"

   # Check WASM compatibility
   opencode workflow run wasm-deployment --debug
   ```

### Getting Help

```bash
# Start OpenCode and use help commands
opencode

# Then use:
/help                    # Show all available commands
/help agent              # Get help for agent commands
/help config             # Get help for configuration

# View available agents
/agent list

# Get information about specific agent
/ask orchestrator "What agents are available and their capabilities?"
```

## 🤝 Contributing

### Development Setup

```bash
# Set up development environment
opencode setup dev

# Install development dependencies
opencode setup dependencies

# Configure pre-commit hooks
opencode setup hooks
```

### Contribution Workflow

1. Create a feature branch
2. Make changes using OpenCode agents
3. Run tests and quality checks
4. Submit pull request

```bash
# Run pre-commit checks
opencode quality check

# Run full test suite
opencode workflow run test-pipeline

# Generate contribution report
opencode workflow run documentation-generation
```

## 📈 Performance Optimization

### Profiling

```bash
# Profile Rust code
opencode agent ask performance-optimizer "Profile the neural network training performance"

# Optimize memory usage
opencode agent ask performance-optimizer "Optimize memory usage in the matrix operations"

# GPU optimization
opencode agent ask performance-optimizer "Optimize GPU kernel performance"
```

### Benchmarking

```bash
# Run benchmarks
opencode workflow run performance-benchmarks

# Compare performance
opencode monitor performance --compare

# Generate performance report
opencode monitor performance --report
```

## 🔄 Integration with Existing Tools

### Claude Code Integration

The OpenCode setup integrates seamlessly with existing Claude Code configuration:

- Uses existing `.claude.json` settings
- Leverages `.roomodes` agent definitions
- Integrates with MCP servers via `.mcp.json`

### GitHub Actions

OpenCode workflows complement existing GitHub Actions:

- CI/CD pipelines in `.github/workflows/`
- Issue templates integration
- Automated testing and deployment

### Docker Integration

```bash
# Build Docker images
opencode deploy docker

# Run in container
opencode workflow run build-pipeline --docker

# Deploy to registry
opencode deploy docker --registry ghcr.io
```

## 📞 Support

For support and questions:

1. Check this documentation
2. Use `opencode help` for command assistance
3. Review existing issues and solutions
4. Create new issues for bugs or feature requests

## 📝 License

This OpenCode configuration is part of the ruv-fann project and follows the same license terms (MIT OR Apache-2.0).

---

**Note**: This OpenCode setup is designed to enhance the development experience for the complex ruv-fann multi-project repository. It provides specialized agents configured in the official `opencode.json` format, tailored to the unique requirements of neural network development, WebAssembly integration, and swarm coordination systems.