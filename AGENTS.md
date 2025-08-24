# AGENTS.md - Development Guidelines for ruv-FANN

## Project Structure Overview

This is a multi-crate Rust workspace with JavaScript/TypeScript components:

- **Main crate**: `ruv-fann` - Core neural network library
- **Workspace**: `ruv-swarm` - Swarm orchestration with 15+ crates
- **JavaScript**: `ruv-swarm/npm` - Node.js CLI and WebAssembly bindings
- **Features**: WASM, GPU acceleration, MCP integration, no_std support

## ⚠️ CRITICAL VALIDATION REQUIREMENTS

**IMPORTANT**: Agents and developers MUST NOT claim "success", "production ready", or "complete" status without completing the full verification cycle.

### Required Verification Steps (ALL must pass):
1. ✅ **Build Verification**: `cargo build --all-features` must succeed
2. ✅ **Unit Tests**: `cargo test --all-features` must pass (100% success rate)
3. ✅ **Integration Tests**: All integration tests must pass
4. ✅ **Code Quality**: `cargo clippy --all-targets --all-features -- -D warnings` must pass
5. ✅ **Documentation**: `cargo doc --all-features --no-deps` must generate without errors
6. ✅ **Security Audit**: `cargo audit` must pass without critical vulnerabilities
7. ✅ **Cross-Platform**: Must build for all supported targets (x86_64, ARM64, WASM)
8. ✅ **E2E Tests**: End-to-end functionality tests must pass

### Validation Status Requirements:
- **❌ INCOMPLETE**: If any step fails or is blocked (e.g., missing C compiler)
- **⏳ PARTIALLY VALIDATED**: Static analysis complete, but build/test verification pending
- **✅ FULLY VALIDATED**: All verification steps completed successfully

### Prohibited Actions:
- ❌ Claiming "production ready" without full build verification
- ❌ Claiming "success" without all tests passing
- ❌ Claiming "complete" without code quality checks passing
- ❌ Proceeding to deployment without security audit

**Remember**: Code structure and design can be excellent, but true readiness requires successful compilation, testing, and quality verification.

## Build/Lint/Test Commands

### Rust Commands (Main Crate)
```bash
# Build all features
cargo build --all-features

# Build specific package
cargo build -p ruv-fann

# Run all tests
cargo test --all-features

# Run single test
cargo test test_name --all-features
cargo test -- --exact test_name

# Run tests for specific package
cargo test -p ruv-fann

# Run doctests
cargo test --doc --all-features

# Lint with clippy (with project-specific allowances)
cargo clippy --all-targets --all-features -- -A dead_code -A unused_imports -A unused_variables -A clippy::field_reassign_with_default -A clippy::manual_clamp -A clippy::needless_range_loop -A clippy::approx_constant

# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Security audit
cargo audit

# Check licenses
cargo deny check

# Build documentation
cargo doc --all-features --no-deps

# Run benchmarks
cargo bench --all-features
```

### Rust Commands (Workspace - ruv-swarm)
```bash
# Navigate to workspace
cd ruv-swarm

# Build entire workspace
cargo build --workspace --all-features

# Build specific crate
cargo build -p ruv-swarm-core

# Test entire workspace
cargo test --workspace --all-features

# Test specific crate
cargo test -p ruv-swarm-core

# Run workspace clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build WASM packages
cargo build --target wasm32-unknown-unknown -p ruv-swarm-wasm
cargo build --target wasm32-unknown-unknown -p ruv-swarm-wasm-unified

# Build CLI
cargo build --release -p ruv-swarm-cli
```

### JavaScript/Node.js Commands
```bash
# Navigate to JavaScript package
cd ruv-swarm/npm

# Install dependencies
npm install

# Run all tests
npm test
npm run test:all

# Run single test
npm test -- --grep "test_name"
npm run test:jest -- --testNamePattern="test_name"

# Run Jest tests
npm run test:jest

# Run Jest with coverage
npm run test:jest:coverage

# Lint JavaScript/TypeScript (using flat config)
npm run lint

# Fix linting issues
npm run lint:fix

# Check linting (no warnings)
npm run lint:check

# Format code with Prettier
npx prettier --write "src/**/*.{js,ts,mjs,cjs}"

# Check formatting
npx prettier --check "src/**/*.{js,ts,mjs,cjs}"

# Build JavaScript
npm run build

# Build WASM bindings
npm run build:wasm
npm run build:wasm-simd
npm run build:wasm-opt

# Build all components
npm run build:all

# Quality check (lint + test)
npm run quality:check

# Deploy/prepare for release
npm run deploy:prepare
```

### Cross-Platform Testing
```bash
# Test no_std compatibility
cargo build --no-default-features --features no_std

# Test WASM compatibility
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm

# Test GPU features
cargo build --features gpu

# Test WebGPU features
cargo build --features webgpu

# Test workspace no_std
cd ruv-swarm && cargo build --workspace --no-default-features --features no-std

# Test workspace WASM
cd ruv-swarm && cargo build --workspace --target wasm32-unknown-unknown --features wasm
```

## Code Style Guidelines

### Rust Code Style

#### General Conventions
- **Edition**: Rust 2021 (workspace uses 1.85+)
- **Formatting**: Use `rustfmt` (default settings)
- **Linting**: Use `clippy` with project-specific allowances (see `.cargo/config.toml`)
- **Naming**: snake_case for variables/functions, PascalCase for types, SCREAMING_SNAKE for constants
- **Documentation**: Comprehensive doc comments for public APIs with examples
- **Error Handling**: Use `thiserror` for custom error types, `anyhow` for generic errors
- **Async**: Use `tokio` for async runtime, `async-trait` for async traits

#### Imports and Organization
```rust
// Standard library imports first
use std::collections::HashMap;
use std::sync::Arc;

// External crate imports (alphabetically)
use serde::{Deserialize, Serialize};
use thiserror::Error;
use num_traits::Float;
use tokio::sync::RwLock;

// Workspace dependencies
use ruv_swarm_core::agent::Agent;
use ruv_swarm_persistence::Database;

// Local crate imports
use crate::network::Network;
use crate::errors::NetworkError;

// Feature-gated imports
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// Development imports
#[cfg(test)]
use proptest::prelude::*;
```

#### Error Handling
```rust
// Define custom error types with thiserror
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Input size mismatch: expected {expected}, got {actual}")]
    InputSizeMismatch { expected: usize, actual: usize },

    #[error("Invalid layer configuration: {reason}")]
    InvalidLayerConfiguration { reason: String },

    #[error("Training failed: {source}")]
    TrainingFailed {
        #[from]
        source: TrainingError,
    },
}

// Use Result for fallible operations
pub async fn train(&mut self, data: &TrainingData) -> Result<(), NetworkError> {
    // Implementation
    Ok(())
}

// Use anyhow for generic error handling in applications
use anyhow::{Context, Result};

pub async fn complex_operation(&self) -> Result<()> {
    some_fallible_operation()
        .await
        .context("Failed to perform complex operation")?;
    Ok(())
}
```

#### Documentation
```rust
/// A feedforward neural network with swarm orchestration capabilities
///
/// This implementation provides both standalone neural network functionality
/// and integration with the ruv-swarm distributed intelligence framework.
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use ruv_fann::Network;
///
/// let network = Network::<f32>::new(&[2, 3, 1]);
/// ```
///
/// ## With Swarm Integration
/// ```
/// use ruv_swarm_core::Swarm;
/// use ruv_fann::Network;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let swarm = Swarm::new()?;
/// let network = Network::<f32>::with_swarm(swarm, &[2, 3, 1]).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Performance
/// - O(n) forward pass complexity
/// - O(n²) training complexity for backpropagation
/// - Memory efficient with arena allocation
///
/// # Safety
/// This implementation uses safe Rust with no unsafe code blocks.
/// Memory safety is guaranteed by the borrow checker.
#[derive(Debug, Clone)]
pub struct Network<T: Float> {
    /// The layers of the network
    pub layers: Vec<Layer<T>>,

    /// Connection rate (1.0 = fully connected, 0.0 = no connections)
    pub connection_rate: T,
}
```

#### Memory Management
- Use references (`&`) when possible for zero-copy operations
- Use `Arc` for shared ownership across threads
- Use `RwLock` for interior mutability in async contexts
- Implement `Drop` for manual cleanup when needed
- Prefer stack allocation over heap when possible
- Use arena allocation for neural network structures
- Avoid unnecessary allocations in hot paths

#### Async and Concurrency
```rust
// Use tokio for async operations
use tokio::sync::RwLock;

pub struct SharedNetwork<T: Float> {
    network: Arc<RwLock<Network<T>>>,
}

impl<T: Float> SharedNetwork<T> {
    pub async fn train_batch(&self, batch: &[TrainingData]) -> Result<(), NetworkError> {
        let mut network = self.network.write().await;
        network.train_batch(batch).await
    }

    pub async fn predict(&self, input: &[T]) -> Result<Vec<T>, NetworkError> {
        let network = self.network.read().await;
        network.predict(input)
    }
}
```

#### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_network_creation() {
        let network = Network::<f32>::new(&[2, 3, 1]);
        assert_eq!(network.num_inputs(), 2);
        assert_eq!(network.num_outputs(), 1);
    }

    // Property-based testing
    proptest! {
        #[test]
        fn test_network_properties(layer_sizes in prop::collection::vec(1..10usize, 2..5)) {
            let network = Network::<f32>::new(&layer_sizes);
            prop_assert!(network.num_layers() == layer_sizes.len());
        }
    }

    // Async testing
    #[tokio::test]
    async fn test_async_training() {
        let network = Network::<f32>::new(&[2, 2, 1]);
        let training_data = vec![/* ... */];
        let result = network.train(&training_data).await;
        assert!(result.is_ok());
    }
}
```

### JavaScript/TypeScript Code Style

#### General Conventions
- **Module System**: ES6 modules with `.js` extensions (TypeScript uses `.ts`)
- **Formatting**: Prettier with 2-space indentation, single quotes, semicolons
- **Linting**: ESLint v9+ with flat config, TypeScript parser
- **Naming**: camelCase for variables/functions, PascalCase for classes/types/interfaces
- **Types**: Use TypeScript for all new code, strict mode enabled
- **Node.js**: Target Node.js 18.20.8+ with full ES2022 support

#### Imports and Organization
```javascript
// Node.js built-ins (alphabetically)
import { promises as fs } from 'fs';
import { join, resolve } from 'path';
import { createRequire } from 'module';

// External dependencies (alphabetically)
import { v4 as uuidv4 } from 'uuid';
import WebSocket from 'ws';
import { Command } from 'commander';

// Local imports (relative paths)
import { NeuralNetwork } from './neural-network.js';
import { SwarmCoordinator } from './swarm-coordinator.js';
import { validateConfig } from './utils/validation.js';

// Type imports (separated)
import type { Config } from './types.js';
import type { TrainingData } from './types/training.js';
import type { WebSocketConnection } from './types/websocket.js';

// WASM imports
import init, { NeuralNetwork as WasmNetwork } from './wasm/ruv_swarm_wasm.js';
```

#### Error Handling
```javascript
// Use async/await with try-catch
async function loadModel(path) {
  try {
    const data = await fs.readFile(path, 'utf8');
    return JSON.parse(data);
  } catch (error) {
    // Enhanced error with context
    throw new Error(`Failed to load model from ${path}: ${error.message}`, {
      cause: error,
    });
  }
}

// Custom error classes with proper inheritance
class ValidationError extends Error {
  constructor(message, field, value) {
    super(message);
    this.name = 'ValidationError';
    this.field = field;
    this.value = value;
  }
}

// Error handling with specific types
class NetworkError extends Error {
  constructor(message, code, details = {}) {
    super(message);
    this.name = 'NetworkError';
    this.code = code;
    this.details = details;
  }
}

// Use error boundaries for async operations
async function safeOperation() {
  try {
    const result = await riskyOperation();
    return { success: true, data: result };
  } catch (error) {
    return { success: false, error: error.message };
  }
}
```

#### TypeScript Best Practices
```typescript
// Use strict typing with proper interfaces
interface NetworkConfig {
  readonly layers: readonly number[];
  readonly learningRate: number;
  readonly activation: ActivationFunction;
  readonly optimizer?: Optimizer;
  readonly loss?: LossFunction;
}

// Use branded types for type safety
type UserId = string & { readonly __brand: 'UserId' };
type ModelId = string & { readonly __brand: 'ModelId' };

// Avoid any type - use unknown instead
function processData(data: unknown): ProcessedData {
  if (!isValidData(data)) {
    throw new ValidationError('Invalid data format', 'data', data);
  }
  return data as ProcessedData;
}

// Use generics with constraints
class NeuralNetwork<T extends number | Float32Array = Float32Array> {
  private readonly config: NetworkConfig;
  private readonly weights: T[];

  constructor(config: NetworkConfig) {
    this.config = config;
    this.weights = [];
  }

  // Use proper return types
  public async train(data: TrainingData): Promise<TrainingResult> {
    // Implementation
    return { epochs: 100, loss: 0.01 };
  }

  // Use readonly for immutable data
  public getConfig(): Readonly<NetworkConfig> {
    return this.config;
  }
}

// Use utility types
type Optional<T> = T | undefined;
type ValueOf<T> = T[keyof T];

// Use template literal types for better type safety
type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';
type ApiEndpoint = `/${string}`;
type HttpRequest = `${HttpMethod} ${ApiEndpoint}`;
```

#### Async/Await Patterns
```javascript
// Prefer async/await over promises
async function trainNetwork(network, data) {
  const results = [];

  for (const batch of data.batches) {
    try {
      const result = await network.trainBatch(batch);
      results.push(result);

      // Yield control to avoid blocking event loop
      await new Promise(resolve => setImmediate(resolve));
    } catch (error) {
      console.error(`Training batch failed: ${error.message}`);
      // Continue with next batch or implement retry logic
    }
  }

  return results;
}

// Use Promise.allSettled for parallel operations
async function trainMultipleNetworks(networks, data) {
  const results = await Promise.allSettled(
    networks.map(async (network, index) => {
      try {
        return await network.train(data);
      } catch (error) {
        throw new Error(`Network ${index} training failed: ${error.message}`);
      }
    })
  );

  return results.map((result, index) => ({
    network: index,
    success: result.status === 'fulfilled',
    data: result.status === 'fulfilled' ? result.value : null,
    error: result.status === 'rejected' ? result.reason : null,
  }));
}

// Use AbortController for cancellable operations
async function cancellableTraining(network, data, signal) {
  const trainingPromise = network.train(data);

  // Race between training and abort signal
  const result = await Promise.race([
    trainingPromise,
    new Promise((_, reject) => {
      signal.addEventListener('abort', () => {
        reject(new Error('Training was cancelled'));
      });
    }),
  ]);

  return result;
}
```

#### WebAssembly Integration
```javascript
// Proper WASM initialization
async function initializeWasm() {
  try {
    // Initialize the WASM module
    await init();

    // Create WASM instances
    const network = new WasmNetwork();

    return network;
  } catch (error) {
    throw new Error(`WASM initialization failed: ${error.message}`);
  }
}

// Memory management with WASM
class WasmNeuralNetwork {
  constructor() {
    this.memory = null;
    this.network = null;
  }

  async initialize() {
    await init();
    this.network = new WasmNetwork();
    this.memory = this.network.memory; // Access WASM memory if needed
  }

  // Proper cleanup
  destroy() {
    if (this.network) {
      this.network.free(); // Free WASM memory
      this.network = null;
    }
    this.memory = null;
  }
}
```

#### Testing Patterns
```javascript
import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';

// Use describe blocks for organization
describe('NeuralNetwork', () => {
  let network;

  beforeEach(async () => {
    network = new NeuralNetwork({ layers: [2, 3, 1] });
    await network.initialize();
  });

  afterEach(() => {
    if (network) {
      network.destroy();
    }
  });

  it('should create network with correct structure', () => {
    expect(network.numInputs).toBe(2);
    expect(network.numOutputs).toBe(1);
  });

  it('should perform forward pass', async () => {
    const inputs = [0.5, 0.7];
    const outputs = await network.forward(inputs);
    expect(outputs).toHaveLength(1);
    expect(typeof outputs[0]).toBe('number');
    expect(outputs[0]).toBeGreaterThanOrEqual(0);
    expect(outputs[0]).toBeLessThanOrEqual(1);
  });

  it('should handle errors gracefully', async () => {
    await expect(network.forward([])).rejects.toThrow(ValidationError);
  });

  // Test async operations
  it('should train asynchronously', async () => {
    const trainingData = [
      { input: [0, 0], output: [0] },
      { input: [0, 1], output: [1] },
      { input: [1, 0], output: [1] },
      { input: [1, 1], output: [0] },
    ];

    const result = await network.train(trainingData);
    expect(result).toHaveProperty('epochs');
    expect(result).toHaveProperty('loss');
    expect(result.loss).toBeLessThan(0.1);
  });
});
```

### Testing Guidelines

#### Rust Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use proptest::prelude::*;

    // Unit tests
    #[test]
    fn test_network_creation() {
        let network = Network::<f32>::new(&[2, 3, 1]);
        assert_eq!(network.num_inputs(), 2);
        assert_eq!(network.num_outputs(), 1);
    }

    #[test]
    fn test_forward_pass() {
        let network = Network::<f32>::new(&[2, 2, 1]);
        let inputs = vec![0.5, 0.7];
        let outputs = network.run(&inputs).unwrap();
        assert_eq!(outputs.len(), 1);
        assert!(outputs[0] >= 0.0 && outputs[0] <= 1.0);
    }

    // Property-based testing with proptest
    proptest! {
        #[test]
        fn test_network_properties(layer_sizes in prop::collection::vec(1..10usize, 2..5)) {
            let network = Network::<f32>::new(&layer_sizes);
            prop_assert!(network.num_layers() == layer_sizes.len());
            prop_assert!(network.total_neurons() > 0);
        }

        #[test]
        fn test_input_output_consistency(
            layer_sizes in prop::collection::vec(2..20usize, 2..6),
            inputs in prop::collection::vec(-10.0..10.0f32, 1..20)
        ) {
            let network = Network::<f32>::new(&layer_sizes);
            let input_size = *layer_sizes.first().unwrap();

            // Ensure inputs match network input size
            let test_inputs = &inputs[..input_size.min(inputs.len())];
            let outputs = network.run(test_inputs).unwrap();

            prop_assert_eq!(outputs.len(), *layer_sizes.last().unwrap());
        }
    }

    // Integration tests
    #[cfg(feature = "serde")]
    #[test]
    fn test_serialization() {
        let network = Network::<f32>::new(&[2, 3, 1]);
        let serialized = serde_json::to_string(&network).unwrap();
        let deserialized: Network<f32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(network.num_layers(), deserialized.num_layers());
        assert_eq!(network.num_inputs(), deserialized.num_inputs());
    }

    // Benchmark tests
    #[bench]
    fn bench_forward_pass(b: &mut Bencher) {
        let network = Network::<f32>::new(&[784, 128, 64, 10]);
        let inputs = vec![0.5; 784];

        b.iter(|| {
            let _outputs = network.run(&inputs).unwrap();
        });
    }

    // Async tests
    #[tokio::test]
    async fn test_async_training() {
        let mut network = Network::<f32>::new(&[2, 3, 1]);
        let training_data = create_training_data();

        let result = network.train_async(&training_data).await;
        assert!(result.is_ok());

        let loss = result.unwrap();
        assert!(loss >= 0.0 && loss < 1.0);
    }

    // Memory safety tests
    #[test]
    fn test_memory_safety() {
        let network = Network::<f32>::new(&[1000, 100, 10]);

        // Test with large inputs
        let inputs = vec![1.0; 1000];
        let outputs = network.run(&inputs).unwrap();

        assert_eq!(outputs.len(), 10);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_full_training_pipeline() {
        let mut network = Network::<f32>::new(&[4, 8, 3]);
        let training_data = generate_xor_training_data();

        // Train for multiple epochs
        for epoch in 0..100 {
            let error = network.train_epoch(&training_data).unwrap();
            if error < 0.01 {
                break;
            }
        }

        // Test trained network
        let test_cases = vec![
            (vec![0.0, 0.0], vec![0.0]),
            (vec![0.0, 1.0], vec![1.0]),
            (vec![1.0, 0.0], vec![1.0]),
            (vec![1.0, 1.0], vec![0.0]),
        ];

        for (input, expected) in test_cases {
            let output = network.run(&input).unwrap();
            let predicted = if output[0] > 0.5 { 1.0 } else { 0.0 };
            assert!((predicted - expected[0]).abs() < 0.1);
        }
    }

    #[test]
    fn test_persistence() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("network.bin");

        let original_network = Network::<f32>::new(&[2, 3, 1]);

        // Save network
        original_network.save(&file_path).unwrap();

        // Load network
        let loaded_network = Network::<f32>::load(&file_path).unwrap();

        // Compare networks
        assert_eq!(original_network.num_layers(), loaded_network.num_layers());
        assert_eq!(original_network.num_inputs(), loaded_network.num_inputs());
    }
}
```

#### JavaScript Testing
```javascript
import { describe, it, expect, beforeEach, afterEach, beforeAll, afterAll } from '@jest/globals';

// Use describe blocks for organization
describe('NeuralNetwork', () => {
  let network;

  beforeAll(async () => {
    // Setup shared resources
    await initializeWasm();
  });

  afterAll(async () => {
    // Cleanup shared resources
    await cleanupWasm();
  });

  beforeEach(async () => {
    network = new NeuralNetwork({ layers: [2, 3, 1] });
    await network.initialize();
  });

  afterEach(() => {
    if (network) {
      network.destroy();
    }
  });

  describe('Initialization', () => {
    it('should create network with correct structure', () => {
      expect(network.numInputs).toBe(2);
      expect(network.numOutputs).toBe(1);
      expect(network.numLayers).toBe(3);
    });

    it('should initialize with default parameters', () => {
      expect(network.learningRate).toBe(0.01);
      expect(network.activation).toBe('sigmoid');
    });
  });

  describe('Forward Pass', () => {
    it('should perform forward pass with valid inputs', async () => {
      const inputs = [0.5, 0.7];
      const outputs = await network.forward(inputs);

      expect(outputs).toHaveLength(1);
      expect(typeof outputs[0]).toBe('number');
      expect(outputs[0]).toBeGreaterThanOrEqual(0);
      expect(outputs[0]).toBeLessThanOrEqual(1);
    });

    it('should handle batch inputs', async () => {
      const batchInputs = [
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 0.0],
        [1.0, 1.0],
      ];

      const batchOutputs = await network.forwardBatch(batchInputs);

      expect(batchOutputs).toHaveLength(4);
      batchOutputs.forEach(outputs => {
        expect(outputs).toHaveLength(1);
      });
    });
  });

  describe('Training', () => {
    it('should train asynchronously', async () => {
      const trainingData = [
        { input: [0, 0], output: [0] },
        { input: [0, 1], output: [1] },
        { input: [1, 0], output: [1] },
        { input: [1, 1], output: [0] },
      ];

      const result = await network.train(trainingData);

      expect(result).toHaveProperty('epochs');
      expect(result).toHaveProperty('loss');
      expect(result.loss).toBeLessThan(0.1);
    });

    it('should handle training cancellation', async () => {
      const trainingData = generateLargeDataset();
      const abortController = new AbortController();

      // Cancel training after 100ms
      setTimeout(() => abortController.abort(), 100);

      await expect(
        network.train(trainingData, { signal: abortController.signal })
      ).rejects.toThrow('Training was cancelled');
    });
  });

  describe('Error Handling', () => {
    it('should handle invalid inputs', async () => {
      await expect(network.forward([])).rejects.toThrow(ValidationError);
      await expect(network.forward(null)).rejects.toThrow(ValidationError);
      await expect(network.forward(undefined)).rejects.toThrow(ValidationError);
    });

    it('should handle network errors', async () => {
      const corruptedNetwork = new NeuralNetwork({ layers: [] });

      await expect(corruptedNetwork.initialize()).rejects.toThrow(NetworkError);
    });
  });

  describe('Performance', () => {
    it('should meet performance requirements', async () => {
      const largeInputs = Array.from({ length: 1000 }, () => Math.random());

      const startTime = performance.now();
      const outputs = await network.forward(largeInputs);
      const endTime = performance.now();

      expect(endTime - startTime).toBeLessThan(100); // Should complete within 100ms
      expect(outputs).toHaveLength(1);
    });
  });
});

// Integration tests
describe('Swarm Integration', () => {
  let swarm;
  let network;

  beforeAll(async () => {
    swarm = new Swarm();
    await swarm.initialize();
    network = new NeuralNetwork({ layers: [10, 20, 5] });
    await network.initialize();
  });

  afterAll(async () => {
    await swarm.destroy();
    network.destroy();
  });

  it('should integrate with swarm coordinator', async () => {
    const coordinator = new SwarmCoordinator(swarm, network);

    const result = await coordinator.distributeTraining([
      { input: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10], output: [0.5] },
    ]);

    expect(result).toHaveProperty('success');
    expect(result.success).toBe(true);
  });
});
```

### Performance Considerations

#### Rust Performance
- **Hot Path Optimization**: Use `#[inline]` for small functions called frequently
- **Memory Layout**: Prefer stack allocation over heap when possible
- **Parallel Processing**: Use `rayon` for CPU-bound parallel operations
- **SIMD Acceleration**: Leverage SIMD instructions for vectorized operations
- **Zero-Copy Operations**: Use references and slices to avoid unnecessary copying
- **Arena Allocation**: Use arena allocators for neural network structures
- **Profiling**: Use `cargo flamegraph` and `cargo bench` for performance analysis
- **Memory Pool**: Implement custom memory pools for tensor operations
- **Cache-Friendly Code**: Structure data for better cache locality

```rust
// Example of performance-optimized code
#[inline]
pub fn activate_sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

pub fn forward_pass_optimized(&self, inputs: &[f32], outputs: &mut [f32]) {
    // Zero-copy operations
    for (i, &input) in inputs.iter().enumerate() {
        let mut sum = 0.0;
        for (j, &weight) in self.weights[i].iter().enumerate() {
            sum += input * weight;
        }
        outputs[i] = activate_sigmoid(sum);
    }
}
```

#### JavaScript Performance
- **Web Workers**: Use Web Workers for CPU-intensive neural network operations
- **WebAssembly**: Leverage WASM for performance-critical computations
- **Typed Arrays**: Use `Float32Array` and `Float64Array` for numerical data
- **Streaming**: Implement streaming for large datasets to reduce memory usage
- **Memory Management**: Explicitly manage memory in WASM contexts
- **SIMD**: Use WebAssembly SIMD instructions when available
- **GPU Acceleration**: Use WebGPU for hardware-accelerated computations
- **Profiling**: Use Chrome DevTools Performance tab for analysis

```javascript
// Example of performance-optimized JavaScript
class OptimizedNeuralNetwork {
  constructor() {
    this.weights = new Float32Array(1000);
    this.inputs = new Float32Array(784);
    this.outputs = new Float32Array(10);
  }

  // Use SharedArrayBuffer for cross-worker communication
  async trainInWorker(data) {
    const worker = new Worker('training-worker.js');
    const sharedBuffer = new SharedArrayBuffer(1024 * 1024); // 1MB

    return new Promise((resolve) => {
      worker.postMessage({ data, sharedBuffer });
      worker.onmessage = (event) => resolve(event.data);
    });
  }

  // Batch processing for better performance
  async processBatch(batch) {
    const batchSize = batch.length;
    const inputBuffer = new Float32Array(batchSize * 784);
    const outputBuffer = new Float32Array(batchSize * 10);

    // Fill input buffer
    for (let i = 0; i < batchSize; i++) {
      inputBuffer.set(batch[i].input, i * 784);
    }

    // Process in WASM
    await this.wasmNetwork.processBatch(inputBuffer, outputBuffer, batchSize);

    return outputBuffer;
  }
}
```

#### WASM-Specific Performance
- **Memory Layout**: Design data structures for efficient WASM memory access
- **Function Calls**: Minimize WASM/JS boundary crossings
- **SIMD Instructions**: Use WebAssembly SIMD when targeting modern browsers
- **Bulk Memory**: Use bulk memory operations for data transfer
- **Streaming Compilation**: Use streaming compilation for faster module loading

### Security Guidelines

#### Rust Security
- **Regular Audits**: Run `cargo audit` regularly to check for vulnerabilities
- **No Unsafe Code**: Avoid `unsafe` code blocks unless absolutely necessary
- **Input Validation**: Validate all inputs at API boundaries
- **Clippy Checks**: Use `clippy::pedantic` for additional security checks
- **Memory Safety**: Rely on Rust's ownership system for memory safety
- **Cryptography**: Use `rust-crypto` crates for cryptographic operations
- **Fuzz Testing**: Implement fuzz testing for critical functions

```rust
// Example of secure input validation
pub fn validate_network_config(config: &NetworkConfig) -> Result<(), ValidationError> {
    if config.layers.is_empty() {
        return Err(ValidationError::InvalidConfiguration("Network must have at least one layer".to_string()));
    }

    if config.layers.iter().any(|&size| size == 0) {
        return Err(ValidationError::InvalidConfiguration("Layer size cannot be zero".to_string()));
    }

    if config.learning_rate <= 0.0 || config.learning_rate > 1.0 {
        return Err(ValidationError::InvalidConfiguration("Learning rate must be between 0 and 1".to_string()));
    }

    Ok(())
}
```

#### JavaScript Security
- **Input Sanitization**: Validate and sanitize all user inputs
- **XSS Prevention**: Use proper encoding for dynamic content
- **CSRF Protection**: Implement CSRF tokens for state-changing operations
- **Content Security Policy**: Implement strict CSP headers
- **Dependency Updates**: Keep all dependencies updated with `npm audit`
- **Secure Headers**: Use helmet.js for security headers
- **Authentication**: Implement proper JWT validation
- **Rate Limiting**: Implement rate limiting for API endpoints

```javascript
// Example of secure input validation
function validateTrainingData(data) {
  if (!Array.isArray(data)) {
    throw new ValidationError('Training data must be an array');
  }

  if (data.length === 0) {
    throw new ValidationError('Training data cannot be empty');
  }

  for (let i = 0; i < data.length; i++) {
    const sample = data[i];

    if (!sample.input || !Array.isArray(sample.input)) {
      throw new ValidationError(`Sample ${i}: input must be an array`);
    }

    if (!sample.output || !Array.isArray(sample.output)) {
      throw new ValidationError(`Sample ${i}: output must be an array`);
    }

    // Validate input/output ranges
    for (const value of sample.input) {
      if (typeof value !== 'number' || !isFinite(value)) {
        throw new ValidationError(`Sample ${i}: invalid input value ${value}`);
      }
    }

    for (const value of sample.output) {
      if (typeof value !== 'number' || !isFinite(value) || value < 0 || value > 1) {
        throw new ValidationError(`Sample ${i}: output value must be between 0 and 1`);
      }
    }
  }

  return true;
}
```

#### WASM Security
- **Sandboxing**: WASM runs in a sandboxed environment
- **Input Validation**: Validate all data passed to/from WASM
- **Memory Bounds**: Ensure proper bounds checking for WASM memory access
- **Deterministic Execution**: Ensure WASM code is deterministic for reproducibility

### Development Workflow

#### ⚠️ CRITICAL: Complete Validation Checklist

**Before claiming any success or production readiness, ALL of these must be completed:**

```bash
# Phase 1: Environment Setup
echo "🔧 Environment Check:"
which cc || which gcc || which clang || echo "❌ C compiler required"
rustc --version && cargo --version || echo "❌ Rust toolchain required"

# Phase 2: Code Quality (Static Analysis)
echo "📋 Static Analysis:"
cargo fmt --all -- --check || echo "❌ Code formatting issues"
cargo check --workspace --all-features || echo "❌ Compilation check failed"

# Phase 3: Build Verification (REQUIRES C COMPILER)
echo "🔨 Build Verification:"
cargo build --workspace --all-features || echo "❌ Build failed"
cargo build --target wasm32-unknown-unknown --features wasm || echo "❌ WASM build failed"

# Phase 4: Testing (REQUIRES SUCCESSFUL BUILD)
echo "🧪 Testing:"
cargo test --workspace --all-features || echo "❌ Tests failed"
cargo test --doc --all-features || echo "❌ Documentation tests failed"

# Phase 5: Code Quality (REQUIRES SUCCESSFUL BUILD)
echo "🔍 Code Quality:"
cargo clippy --workspace --all-targets --all-features -- -D warnings || echo "❌ Code quality issues"

# Phase 6: Documentation & Security
echo "📚 Documentation & Security:"
cargo doc --workspace --all-features --no-deps || echo "❌ Documentation generation failed"
cargo audit || echo "❌ Security vulnerabilities found"

# Phase 7: Cross-Platform Verification
echo "🌐 Cross-Platform:"
cargo build --target aarch64-unknown-linux-gnu || echo "❌ ARM64 build failed"

# Phase 8: Performance & Benchmarks
echo "⚡ Performance:"
cargo bench --all-features || echo "❌ Benchmarks failed"
```

#### Validation Status Commands
```bash
# Quick status check
echo "=== VALIDATION STATUS ==="
echo "✅ Static Analysis: $(cargo fmt --all -- --check && echo 'PASSED' || echo 'FAILED')"
echo "✅ Build: $(cargo build --all-features 2>/dev/null && echo 'PASSED' || echo 'FAILED')"
echo "✅ Tests: $(cargo test --all-features 2>/dev/null && echo 'PASSED' || echo 'FAILED')"
echo "✅ Quality: $(cargo clippy --all-targets --all-features -- -D warnings 2>/dev/null && echo 'PASSED' || echo 'FAILED')"
echo "✅ Security: $(cargo audit 2>/dev/null && echo 'PASSED' || echo 'FAILED')"
```

#### Pre-Commit Checks
```bash
# Rust checks
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo audit

# JavaScript checks
cd ruv-swarm/npm
npm run lint:check
npm run test:all
npm audit

# WASM checks
npm run build:wasm
npm run test:wasm
```

#### Git Workflow
```bash
# Feature development workflow
git checkout -b feature/neural-network-optimization
# Make changes following the guidelines above
# Run pre-commit checks
cargo fmt --all && cargo clippy --workspace --all-targets --all-features -- -D warnings
cd ruv-swarm/npm && npm run lint:fix && npm run test:all

# Commit with conventional format
git commit -m "feat: optimize neural network forward pass performance

- Add SIMD acceleration for vector operations
- Implement zero-copy memory management
- Add comprehensive benchmarks
- Update documentation with performance guidelines"

# Push and create PR
git push origin feature/neural-network-optimization
```

#### Code Review Guidelines
- **Security**: Review for security vulnerabilities and proper input validation
- **Performance**: Ensure code meets performance requirements
- **Testing**: Verify comprehensive test coverage
- **Documentation**: Check that public APIs are properly documented
- **Style**: Ensure code follows established style guidelines

#### Release Process
```bash
# Update version
cargo update
cd ruv-swarm/npm && npm version patch

# Run full test suite
cargo test --workspace --all-features
cd ruv-swarm/npm && npm run test:all

# Build all components
cargo build --workspace --release
cd ruv-swarm/npm && npm run build:all

# Security checks
cargo audit
cd ruv-swarm/npm && npm audit

# Create release
git tag v1.0.8
git push origin v1.0.8
cd ruv-swarm/npm && npm publish
```

### Project-Specific Guidelines

#### Workspace Organization
- **Main Crate**: `ruv-fann` - Core neural network functionality
- **Workspace Crates**: Located in `ruv-swarm/crates/` with specific responsibilities
- **JavaScript Package**: `ruv-swarm/npm` - Node.js CLI and WebAssembly bindings
- **Documentation**: Comprehensive docs with examples and performance notes

#### Feature Flags
- Use feature flags for optional functionality
- Document feature dependencies in `Cargo.toml`
- Test with `--all-features` and `--no-default-features`

#### Error Handling Strategy
- Use `thiserror` for custom error types in Rust
- Implement proper error propagation across crate boundaries
- Provide meaningful error messages for debugging

#### Testing Strategy
- **Unit Tests**: Test individual functions and modules
- **Integration Tests**: Test component interactions
- **Property-Based Tests**: Use `proptest` for edge cases
- **Performance Tests**: Benchmark critical paths
- **WASM Tests**: Test WebAssembly functionality

## 🚨 VALIDATION PROTOCOL - Preventing Premature Success Claims

### The Critical Mistake to Avoid
**NEVER claim "success", "production ready", or "complete" without full verification.**

Common mistakes that lead to premature success claims:
- ✅ "Code looks good" → ❌ Missing: Build verification
- ✅ "Documentation is complete" → ❌ Missing: Test execution
- ✅ "Architecture is sound" → ❌ Missing: Code quality checks
- ✅ "Features are implemented" → ❌ Missing: Security audit

### Proper Validation Sequence

#### Phase 1: Environment Verification
```bash
# Must have these tools available
which cc && echo "✅ C compiler available" || echo "❌ C compiler missing"
rustc --version && echo "✅ Rust toolchain ready" || echo "❌ Rust toolchain issue"
```

#### Phase 2: Static Analysis (Can run without C compiler)
```bash
cargo fmt --all -- --check && echo "✅ Formatting correct"
cargo check --workspace --all-features && echo "✅ Compilation check passed"
```

#### Phase 3: Build Verification (REQUIRES C compiler)
```bash
cargo build --workspace --all-features && echo "✅ Build successful"
cargo build --target wasm32-unknown-unknown --features wasm && echo "✅ WASM build successful"
```

#### Phase 4: Test Execution (REQUIRES successful build)
```bash
cargo test --workspace --all-features && echo "✅ All tests passed"
cargo test --doc --all-features && echo "✅ Documentation tests passed"
```

#### Phase 5: Quality Assurance (REQUIRES successful build)
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings && echo "✅ Code quality verified"
cargo audit && echo "✅ Security audit passed"
```

#### Phase 6: Documentation & Cross-Platform
```bash
cargo doc --workspace --all-features --no-deps && echo "✅ Documentation generated"
cargo build --target aarch64-unknown-linux-gnu && echo "✅ Cross-platform build successful"
```

### Validation Status Matrix

| Phase | Status | Description |
|-------|--------|-------------|
| Environment | ✅ Ready | C compiler and Rust toolchain available |
| Static Analysis | ✅ Complete | Code structure and formatting verified |
| Build | ❌ Blocked | Requires C compiler |
| Tests | ❌ Blocked | Requires successful build |
| Quality | ❌ Blocked | Requires successful build |
| Security | ❌ Blocked | Requires successful build |
| Cross-Platform | ❌ Blocked | Requires successful build |

**Current Status: PARTIALLY VALIDATED** ⏳

### Success Claim Prevention Rules

1. **❌ NEVER** claim success without build verification
2. **❌ NEVER** claim production ready without all tests passing
3. **❌ NEVER** claim complete without code quality checks
4. **❌ NEVER** proceed to deployment without security audit

### Correct Status Reporting

**Instead of:**
- ❌ "The codebase is production ready!"
- ❌ "Implementation is complete!"
- ❌ "All features are working!"

**Use:**
- ✅ "Static analysis complete - code structure is excellent"
- ✅ "Architecture design is sound and well-documented"
- ✅ "Features are implemented with proper error handling"
- ✅ "Ready for full verification once C compiler is available"

### Final Validation Command
```bash
# Complete verification sequence
echo "=== FINAL VALIDATION ===" && \
cargo build --all-features && echo "✅ Build: PASSED" && \
cargo test --all-features && echo "✅ Tests: PASSED" && \
cargo clippy --all-targets --all-features -- -D warnings && echo "✅ Quality: PASSED" && \
cargo audit && echo "✅ Security: PASSED" && \
echo "🎉 FULLY VALIDATED - Production Ready!"
```

This document should be updated as the project evolves and new patterns emerge.