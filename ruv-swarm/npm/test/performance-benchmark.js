/**
 * Performance Benchmark Suite for ruv-swarm
 * Measures performance characteristics and identifies bottlenecks
 */

import { performance } from 'perf_hooks';
import { RuvSwarm } from '../src/index.js';

// Mock WebAssembly for testing
global.WebAssembly = {
  validate: jest.fn(() => true),
  instantiate: jest.fn(() => Promise.resolve({
    instance: {
      exports: {
        get_version: () => '1.0.18',
        get_wasm_memory_usage: () => 1024,
        RuntimeFeatures: function() {
          this.simd_available = true;
          this.threads_available = false;
          this.memory_limit = 1048576;
        }
      }
    },
    module: {}
  }))
};

class PerformanceBenchmark {
  constructor() {
    this.results = {};
    this.ruvSwarm = null;
  }

  async setup() {
    this.ruvSwarm = await RuvSwarm.initialize({
      debug: false,
      useSIMD: true
    });
  }

  measureExecutionTime(fn, label, iterations = 100) {
    return new Promise((resolve) => {
      const times = [];

      const runIteration = async (i) => {
        const start = performance.now();
        await fn();
        const end = performance.now();
        times.push(end - start);

        if (i < iterations - 1) {
          setImmediate(() => runIteration(i + 1));
        } else {
          const avg = times.reduce((a, b) => a + b, 0) / times.length;
          const min = Math.min(...times);
          const max = Math.max(...times);
          const p95 = times.sort((a, b) => a - b)[Math.floor(times.length * 0.95)];

          this.results[label] = {
            average: avg,
            minimum: min,
            maximum: max,
            p95: p95,
            iterations: iterations,
            total: times.reduce((a, b) => a + b, 0)
          };

          resolve(this.results[label]);
        }
      };

      runIteration(0);
    });
  }

  async benchmarkSwarmCreation() {
    console.log('🔧 Benchmarking swarm creation...');

    return await this.measureExecutionTime(async () => {
      const config = {
        name: `benchmark-swarm-${Math.random()}`,
        maxAgents: 10,
        enablePersistence: false
      };
      const swarm = await this.ruvSwarm.createSwarm(config);
      // Cleanup
      swarm._swarm = null;
    }, 'swarm_creation', 50);
  }

  async benchmarkAgentSpawning() {
    console.log('🤖 Benchmarking agent spawning...');

    const swarm = await this.ruvSwarm.createSwarm({
      name: 'agent-spawn-benchmark',
      maxAgents: 100
    });

    return await this.measureExecutionTime(async () => {
      const agent = await swarm.spawn({
        type: 'worker',
        capabilities: ['computation']
      });
      // Cleanup
      agent._agent = null;
    }, 'agent_spawning', 30);
  }

  async benchmarkTaskOrchestration() {
    console.log('⚡ Benchmarking task orchestration...');

    const swarm = await this.ruvSwarm.createSwarm({
      name: 'task-orchestration-benchmark'
    });

    const agent = await swarm.spawn({ type: 'worker' });

    return await this.measureExecutionTime(async () => {
      const task = {
        type: 'computation',
        data: Array.from({ length: 100 }, () => Math.random()),
        operation: 'sum'
      };
      await swarm.orchestrate(task);
    }, 'task_orchestration', 100);
  }

  async benchmarkConcurrentOperations() {
    console.log('🔄 Benchmarking concurrent operations...');

    const swarm = await this.ruvSwarm.createSwarm({
      name: 'concurrent-benchmark',
      maxAgents: 20
    });

    // Spawn multiple agents
    const agents = await Promise.all(
      Array.from({ length: 10 }, () => swarm.spawn({ type: 'worker' }))
    );

    return await this.measureExecutionTime(async () => {
      const tasks = agents.map((agent, i) => ({
        type: 'computation',
        data: [i, i + 1, i + 2],
        operation: 'multiply'
      }));

      await Promise.all(tasks.map(task => swarm.orchestrate(task)));
    }, 'concurrent_operations', 20);
  }

  async benchmarkMemoryUsage() {
    console.log('📊 Benchmarking memory usage...');

    const initialMemory = process.memoryUsage();

    const swarm = await this.ruvSwarm.createSwarm({
      name: 'memory-benchmark',
      maxAgents: 50
    });

    // Spawn many agents to test memory usage
    const agents = await Promise.all(
      Array.from({ length: 25 }, () => swarm.spawn({ type: 'worker' }))
    );

    const peakMemory = process.memoryUsage();

    // Execute tasks
    const tasks = agents.map((agent, i) => ({
      type: 'computation',
      data: Array.from({ length: 1000 }, () => Math.random()),
      operation: 'process'
    }));

    await Promise.all(tasks.map(task => swarm.orchestrate(task)));

    const finalMemory = process.memoryUsage();

    this.results.memory_usage = {
      initial: initialMemory,
      peak: peakMemory,
      final: finalMemory,
      growth: {
        heapUsed: peakMemory.heapUsed - initialMemory.heapUsed,
        external: peakMemory.external - initialMemory.external,
        rss: peakMemory.rss - initialMemory.rss
      }
    };

    return this.results.memory_usage;
  }

  async benchmarkLargeDataset() {
    console.log('📈 Benchmarking large dataset processing...');

    const swarm = await this.ruvSwarm.createSwarm({
      name: 'large-dataset-benchmark'
    });

    const largeDataset = Array.from({ length: 10000 }, () => Math.random());

    return await this.measureExecutionTime(async () => {
      const task = {
        type: 'computation',
        data: largeDataset,
        operation: 'statistics'
      };
      await swarm.orchestrate(task);
    }, 'large_dataset_processing', 10);
  }

  async runAllBenchmarks() {
    console.log('🚀 Starting comprehensive performance benchmarks...\n');

    try {
      await this.setup();

      await this.benchmarkSwarmCreation();
      await this.benchmarkAgentSpawning();
      await this.benchmarkTaskOrchestration();
      await this.benchmarkConcurrentOperations();
      await this.benchmarkMemoryUsage();
      await this.benchmarkLargeDataset();

      this.printResults();

    } catch (error) {
      console.error('❌ Benchmark failed:', error);
    }
  }

  printResults() {
    console.log('\n📊 Performance Benchmark Results');
    console.log('=====================================\n');

    Object.entries(this.results).forEach(([test, result]) => {
      if (test === 'memory_usage') {
        console.log(`📊 ${test.replace('_', ' ').toUpperCase()}:`);
        console.log(`   Initial Heap: ${(result.initial.heapUsed / 1024 / 1024).toFixed(2)} MB`);
        console.log(`   Peak Heap: ${(result.peak.heapUsed / 1024 / 1024).toFixed(2)} MB`);
        console.log(`   Final Heap: ${(result.final.heapUsed / 1024 / 1024).toFixed(2)} MB`);
        console.log(`   Growth: ${(result.growth.heapUsed / 1024 / 1024).toFixed(2)} MB`);
        console.log(`   RSS Growth: ${(result.growth.rss / 1024 / 1024).toFixed(2)} MB\n`);
      } else {
        console.log(`⚡ ${test.replace('_', ' ').toUpperCase()}:`);
        console.log(`   Average: ${result.average.toFixed(2)}ms`);
        console.log(`   Minimum: ${result.minimum.toFixed(2)}ms`);
        console.log(`   Maximum: ${result.maximum.toFixed(2)}ms`);
        console.log(`   P95: ${result.p95.toFixed(2)}ms`);
        console.log(`   Iterations: ${result.iterations}`);
        console.log(`   Total Time: ${result.total.toFixed(2)}ms\n`);
      }
    });

    this.generateReport();
  }

  generateReport() {
    const report = {
      timestamp: new Date().toISOString(),
      results: this.results,
      summary: {
        totalTests: Object.keys(this.results).length,
        performanceScore: this.calculatePerformanceScore(),
        recommendations: this.generateRecommendations()
      }
    };

    // Save report to file
    const fs = require('fs');
    const path = require('path');

    const reportPath = path.join(process.cwd(), 'benchmark-report.json');
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

    console.log(`📄 Detailed report saved to: ${reportPath}`);
  }

  calculatePerformanceScore() {
    const scores = [];

    // Swarm creation should be < 50ms
    if (this.results.swarm_creation) {
      scores.push(this.results.swarm_creation.average < 50 ? 100 : Math.max(0, 100 - (this.results.swarm_creation.average - 50)));
    }

    // Agent spawning should be < 100ms
    if (this.results.agent_spawning) {
      scores.push(this.results.agent_spawning.average < 100 ? 100 : Math.max(0, 100 - (this.results.agent_spawning.average - 100)));
    }

    // Task orchestration should be < 200ms
    if (this.results.task_orchestration) {
      scores.push(this.results.task_orchestration.average < 200 ? 100 : Math.max(0, 100 - (this.results.task_orchestration.average - 200)));
    }

    return scores.length > 0 ? scores.reduce((a, b) => a + b, 0) / scores.length : 0;
  }

  generateRecommendations() {
    const recommendations = [];

    if (this.results.swarm_creation?.average > 100) {
      recommendations.push('Optimize swarm creation time - consider lazy initialization');
    }

    if (this.results.agent_spawning?.average > 200) {
      recommendations.push('Improve agent spawning performance - consider object pooling');
    }

    if (this.results.task_orchestration?.average > 500) {
      recommendations.push('Task orchestration is slow - consider parallel processing');
    }

    if (this.results.memory_usage?.growth.heapUsed > 50 * 1024 * 1024) { // 50MB
      recommendations.push('High memory usage detected - implement memory pooling');
    }

    if (this.results.concurrent_operations?.p95 > 2000) {
      recommendations.push('Concurrent operations are slow - optimize thread management');
    }

    return recommendations;
  }
}

// Export for use in other modules
export { PerformanceBenchmark };

// Run benchmarks if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const benchmark = new PerformanceBenchmark();
  benchmark.runAllBenchmarks().catch(console.error);
}