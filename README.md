# do-FANN: The Neural Intelligence Framework 🧠

[![Crates.io](https://img.shields.io/crates/v/do-fann.svg)](https://crates.io/crates/do-fann)
[![Documentation](https://docs.rs/do-fann/badge.svg)](https://docs.rs/do-fann)
[![License](https://img.shields.io/crates/l/do-fann.svg)](https://github.com/d-oit/do-FANN/blob/main/LICENSE)
[![CI](https://github.com/d-oit/do-FANN/workflows/CI/badge.svg)](https://github.com/d-oit/do-FANN/actions)

**What if intelligence could be ephemeral, composable, and surgically precise?**

Welcome to do-FANN, a comprehensive neural intelligence framework that reimagines how we build, deploy, and orchestrate artificial intelligence. This repository contains three groundbreaking projects that work together to deliver unprecedented performance in neural computing, forecasting, and multi-agent orchestration.

## 📋 Repository Information

### Fork & Rebranding
This repository is a fork of the original [ruv-FANN](https://github.com/ruvnet/ruv-FANN) project, rebranded and adapted for generic use cases without dependencies on specific AI platforms.

**Original Repository**: [ruvnet/ruv-FANN](https://github.com/ruvnet/ruv-FANN)  
**Current Repository**: [d-oit/do-FANN](https://github.com/d-oit/do-FANN)  
**Fork Date**: Latest commit from original repository  
**Rebranding**: `ruv-fann` → `do-fann`, `ruv-swarm` → `do-swarm`

### Changes Made
- ✅ **Rebranded**: All package names updated from `ruv-*` to `do-*`
- ✅ **Generic**: Removed dependencies on specific AI platforms (Claude Code)
- ✅ **Updated URLs**: All repository references updated to new location
- ✅ **Documentation**: Comprehensive updates for generic usage
- ✅ **Git Remote**: Updated to point to new repository
- ✅ **Configuration**: Renamed `.claude/` to `.dev/` for generic development
- ✅ **Dependencies**: Updated all package references and configurations

### Special Thanks
We extend our heartfelt thanks to the original ruv-FANN creators and contributors:
- **Ocean(@ohdearquant)** - Core neural network implementation
- **Bron(@syndicate604)** - WASM integration and functionality
- **Jed(@jedarden)** - Platform integration and architecture
- **Shep(@elsheppo)** - Testing and quality assurance
- **The entire ruv-FANN community** - For the groundbreaking work

This fork builds upon their excellent foundation while making it more accessible and generic for broader use cases.

## 🌟 The Vision

We believe AI should be:
- **Ephemeral**: Spin up intelligence when needed, dissolve when done
- **Accessible**: CPU-native, GPU-optional - built for the GPU-poor
- **Composable**: Mix and match neural architectures like LEGO blocks
- **Precise**: Tiny, purpose-built brains for specific tasks

This isn't about calling a model API. This is about **instantiating intelligence**.

## 🎯 What's in This Repository?

### 1. **do-FANN Core** - The Foundation
A complete Rust rewrite of the legendary FANN (Fast Artificial Neural Network) library. Zero unsafe code, blazing performance, and full compatibility with decades of proven neural network algorithms.

### 2. **Neuro-Divergent** - Advanced Neural Forecasting  
27+ state-of-the-art forecasting models (LSTM, N-BEATS, Transformers) with 100% Python NeuralForecast compatibility. 2-4x faster, 25-35% less memory.

### 3. **do-swarm** - Ephemeral Swarm Intelligence
The crown jewel. Distributed swarm intelligence system for orchestrating neural networks and AI agents. Spin up lightweight neural networks that exist just long enough to solve problems.

## 🚀 Quick Install

```bash
# NPX - No installation required!
npx do-swarm@latest init

# NPM - Global installation
npm install -g do-swarm

# Cargo - For Rust developers
cargo install do-swarm-cli

# Docker - Containerized deployment
docker run -it do-swarm:latest
```

That's it. You're now running distributed neural intelligence.

## 🧠 How It Works

### The Magic of Ephemeral Intelligence

1. **Instantiation**: Neural networks are created on-demand for specific tasks
2. **Specialization**: Each network is purpose-built with just enough neurons
3. **Execution**: Networks solve their task using CPU-native WASM
4. **Dissolution**: Networks disappear after completion, no resource waste

### Architecture Overview

```
┌─────────────────────────────────────────────┐
│          Your Application / API             │
├─────────────────────────────────────────────┤
│            do-swarm (CLI/API)               │
├─────────────────────────────────────────────┤
│         Neuro-Divergent Models              │
│    (LSTM, TCN, N-BEATS, Transformers)      │
├─────────────────────────────────────────────┤
│           do-FANN Core Engine               │
│        (Rust Neural Networks)               │
├─────────────────────────────────────────────┤
│            WASM Runtime                     │
│    (Browser/Edge/Server/Embedded)          │
└─────────────────────────────────────────────┘
```

## ⚡ Key Features

### 🏃 Performance
- **<100ms decisions** - Complex reasoning in milliseconds
- **84.8% SWE-Bench** - Best-in-class problem solving
- **2.8-4.4x faster** - Than traditional frameworks
- **32.3% less tokens** - Cost-efficient intelligence

### 🛠️ Technology
- **Pure Rust** - Memory safe, zero panics
- **WebAssembly** - Run anywhere: browser to RISC-V
- **CPU-native** - No CUDA, no GPU required
- **Multi-Platform** - CLI, API, and library interfaces

### 🧬 Intelligence Models
- **27+ Neural Architectures** - From MLP to Transformers
- **5 Swarm Topologies** - Mesh, ring, hierarchical, star, custom
- **7 Cognitive Patterns** - Convergent, divergent, lateral, systems thinking
- **Adaptive Learning** - Real-time evolution and optimization

## 📊 Benchmarks

| Metric | do-swarm | Traditional AI | Improvement |
|--------|----------|----------------|-------------|
| **Neural Network Training** | **2.8-4.4x faster** | Baseline | **High Performance** |
| **Memory Efficiency** | **25-35% less** | Baseline | **Optimal** |
| **Multi-Agent Coordination** | **99.5% accuracy** | Varies | **Reliable** |
| **Cross-Platform Compatibility** | **Universal** | Limited | **Flexible** |

## 🌐 Ecosystem Projects

### Core Projects
- **[do-FANN](./do-fann/)** - Neural network foundation library
- **[Neuro-Divergent](./neuro-divergent/)** - Advanced forecasting models
- **[do-swarm](./ruv-swarm/)** - Distributed swarm intelligence

### Tools & Extensions
- **[CLI Tools](./ruv-swarm/docs/CLI_REFERENCE.md)** - Command-line interface
- **[API Server](./ruv-swarm/docs/API_REFERENCE.md)** - RESTful API for integration
- **[Docker Support](./ruv-swarm/npm/docker/)** - Containerized deployment

## 🤝 Contributing

We welcome contributions from the community! This project follows standard open-source contribution guidelines.

### How to Contribute

1. **Fork & Clone**
    ```bash
    git clone https://github.com/your-username/do-FANN.git
    cd do-FANN
    ```

2. **Set up Development Environment**
    ```bash
    # Install dependencies
    npm install

    # For Rust development
    cargo build

    # Run tests
    npm test
    cargo test
    ```

3. **Make Your Changes**
    - Follow the existing code style and conventions
    - Add tests for new functionality
    - Update documentation as needed
    - Ensure all tests pass

4. **Submit a Pull Request**
    - Create a clear, descriptive PR title
    - Provide detailed description of changes
    - Reference any related issues

### Contribution Areas
- 🐛 **Bug Fixes** - Swarm identifies and fixes issues
- ✨ **Features** - Guided feature implementation
- 📚 **Documentation** - Auto-generated from code analysis
- 🧪 **Tests** - Intelligent test generation
- 🎨 **Examples** - Working demos and tutorials

## 🙏 Acknowledgments

### Special Thanks To

#### Core Contributors
- **Ocean(@ohdearquant)** - Transformed FANN from mock implementations to real neural networks with actual CPU and GPU training. Built the Rust implementation from placeholder code into a functional neural computing engine.
- **Bron(@syndicate604)** - Made the JavaScript/WASM integration actually work by removing mock functions and building real functionality. Transformed broken prototypes into production-ready systems.
- **Jed(@jedarden)** - Platform integration and scope management
- **Shep(@elsheppo)** - Testing framework and quality assurance

#### Project Founders
- **d-oit** - Repository owner and maintainer

#### Projects We Built Upon
- **[FANN](http://leenissen.dk/fann/)** - Steffen Nissen's original Fast Artificial Neural Network library
- **[NeuralForecast](https://github.com/Nixtla/neuralforecast)** - Inspiration for forecasting model APIs
- **[Claude MCP](https://modelcontextprotocol.io/)** - Model Context Protocol for AI integration
- **[Rust WASM](https://rustwasm.github.io/)** - WebAssembly toolchain and ecosystem

#### Open Source Libraries
- **num-traits** - Generic numeric traits
- **ndarray** - N-dimensional arrays
- **serde** - Serialization framework
- **tokio** - Async runtime
- **wasm-bindgen** - WASM bindings

### Community
Thanks to all contributors, issue reporters, and users who have helped shape ruv-FANN into what it is today. Special recognition to the Rust ML community for pioneering memory-safe machine learning.

## 📄 License

Dual-licensed under:
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

Choose whichever license works best for your use case.

## 📈 Git Information

This repository was forked from the original ruv-FANN project and has been adapted for generic use:

```bash
# Repository information
Remote: https://github.com/d-oit/do-FANN.git
Original: https://github.com/ruvnet/ruv-FANN
Status: Active fork with rebranding
Last Sync: Latest commit from original repository
Branch: main
```

### Git History Summary
```bash
* Latest changes include comprehensive rebranding and generic adaptation
* Maintained all original functionality while removing platform dependencies
* Updated all package names, URLs, and configurations
* Preserved git history and commit structure
```

### Recent Changes
- **Rebranding**: Complete rename from `ruv-*` to `do-*` packages
- **Generic**: Removed platform-specific dependencies (Claude Code)
- **Documentation**: Updated for broader compatibility
- **CI/CD**: Maintained all testing and deployment pipelines
- **Configuration**: Migrated from `.claude/` to `.dev/` directory
- **Dependencies**: Updated all package references and environment variables

---

<div align="center">

**Built with ❤️ and 🦀 by the do-FANN team**

*A fork of the excellent ruv-FANN project, adapted for generic AI development*

[GitHub](https://github.com/d-oit/do-FANN) • [Documentation](https://docs.do-fann.org) • [Issues](https://github.com/d-oit/do-FANN/issues)

</div>