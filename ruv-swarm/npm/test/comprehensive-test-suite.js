/**
 * Comprehensive Test Suite for ruv-swarm
 * This test suite covers all major functionality and edge cases
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from '@jest/globals';
import { RuvSwarm } from '../src/index.js';
import fs from 'fs';
import path from 'path';

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

// Mock fs for testing
jest.mock('fs', () => ({
  promises: {
    readFile: jest.fn(() => Promise.resolve(new ArrayBuffer(1024))),
    writeFile: jest.fn(() => Promise.resolve()),
    mkdir: jest.fn(() => Promise.resolve()),
    unlink: jest.fn(() => Promise.resolve())
  },
  readFileSync: jest.fn(() => new ArrayBuffer(1024)),
  writeFileSync: jest.fn(),
  existsSync: jest.fn(() => true)
}));

describe('RuvSwarm Comprehensive Test Suite', () => {
  let ruvSwarm;
  let mockWasmModule;

  beforeAll(async () => {
    // Setup mock WASM module
    mockWasmModule = {
      RuvSwarm: jest.fn().mockImplementation(() => ({
        name: 'test-swarm',
        agent_count: 0,
        max_agents: 10,
        spawn: jest.fn(() => Promise.resolve({ id: 'agent-1', status: 'active' })),
        orchestrate: jest.fn(() => Promise.resolve({ success: true, result: 'completed' })),
        get_agents: jest.fn(() => []),
        get_status: jest.fn(() => ({ active: true, agents: 0 }))
      }))
    };

    // Mock the WASM loading
    jest.spyOn(path, 'resolve').mockImplementation((...args) => args.join('/'));
    jest.spyOn(fs.promises, 'readFile').mockResolvedValue(new ArrayBuffer(1024));
  });

  afterAll(() => {
    jest.restoreAllMocks();
  });

  describe('Initialization', () => {
    it('should initialize successfully with valid options', async () => {
      const options = {
        debug: true,
        useSIMD: true,
        workerPoolSize: 4
      };

      ruvSwarm = await RuvSwarm.initialize(options);
      expect(ruvSwarm).toBeDefined();
      expect(ruvSwarm._options.debug).toBe(true);
    });

    it('should detect SIMD support', () => {
      const hasSIMD = RuvSwarm.detectSIMDSupport();
      expect(typeof hasSIMD).toBe('boolean');
    });

    it('should get runtime features', () => {
      const features = RuvSwarm.getRuntimeFeatures();
      expect(features).toHaveProperty('simdAvailable');
      expect(features).toHaveProperty('threadsAvailable');
      expect(features).toHaveProperty('memoryLimit');
    });

    it('should get version', () => {
      const version = RuvSwarm.getVersion();
      expect(typeof version).toBe('string');
    });

    it('should get memory usage', () => {
      const memoryUsage = RuvSwarm.getMemoryUsage();
      expect(typeof memoryUsage).toBe('number');
    });
  });

  describe('Swarm Management', () => {
    beforeEach(async () => {
      ruvSwarm = await RuvSwarm.initialize();
    });

    it('should create a swarm with valid configuration', async () => {
      const config = {
        name: 'test-swarm',
        maxAgents: 5,
        enablePersistence: true
      };

      const swarm = await ruvSwarm.createSwarm(config);
      expect(swarm).toBeDefined();
      expect(swarm.name).toBe('test-swarm');
      expect(swarm.maxAgents).toBe(5);
    });

    it('should handle swarm creation errors gracefully', async () => {
      const invalidConfig = {
        name: '', // Invalid name
        maxAgents: -1 // Invalid number
      };

      await expect(ruvSwarm.createSwarm(invalidConfig)).rejects.toThrow();
    });

    it('should spawn agents successfully', async () => {
      const swarm = await ruvSwarm.createSwarm({ name: 'test-swarm', maxAgents: 5 });
      const agentConfig = {
        type: 'neural',
        capabilities: ['processing', 'learning']
      };

      const agent = await swarm.spawn(agentConfig);
      expect(agent).toBeDefined();
      expect(agent.id).toBeDefined();
      expect(agent.agentType).toBe('neural');
    });

    it('should orchestrate tasks', async () => {
      const swarm = await ruvSwarm.createSwarm({ name: 'test-swarm' });
      const task = {
        type: 'computation',
        data: [1, 2, 3, 4, 5],
        operation: 'sum'
      };

      const result = await swarm.orchestrate(task);
      expect(result).toBeDefined();
      expect(result.success).toBe(true);
    });
  });

  describe('Agent Functionality', () => {
    let swarm, agent;

    beforeEach(async () => {
      ruvSwarm = await RuvSwarm.initialize();
      swarm = await ruvSwarm.createSwarm({ name: 'test-swarm' });
      agent = await swarm.spawn({ type: 'neural' });
    });

    it('should have required agent properties', () => {
      expect(agent.id).toBeDefined();
      expect(agent.agentType).toBeDefined();
      expect(agent.status).toBeDefined();
      expect(agent.tasksCompleted).toBeDefined();
    });

    it('should execute tasks', async () => {
      const task = {
        type: 'computation',
        data: [1, 2, 3],
        operation: 'multiply'
      };

      const result = await agent.execute(task);
      expect(result).toBeDefined();
    });

    it('should get metrics', () => {
      const metrics = agent.getMetrics();
      expect(metrics).toBeDefined();
      expect(typeof metrics).toBe('object');
    });

    it('should get capabilities', () => {
      const capabilities = agent.getCapabilities();
      expect(capabilities).toBeDefined();
      expect(Array.isArray(capabilities)).toBe(true);
    });

    it('should reset agent state', () => {
      expect(() => agent.reset()).not.toThrow();
    });
  });

  describe('Error Handling', () => {
    beforeEach(async () => {
      ruvSwarm = await RuvSwarm.initialize();
    });

    it('should handle initialization errors', async () => {
      // Mock a failure in WASM loading
      jest.spyOn(fs.promises, 'readFile').mockRejectedValue(new Error('File not found'));

      await expect(RuvSwarm.initialize()).rejects.toThrow();
    });

    it('should handle swarm creation with invalid config', async () => {
      const invalidConfig = {
        name: null,
        maxAgents: 'invalid'
      };

      await expect(ruvSwarm.createSwarm(invalidConfig)).rejects.toThrow();
    });

    it('should handle agent spawn failures', async () => {
      const swarm = await ruvSwarm.createSwarm({ name: 'test-swarm' });

      // Mock spawn failure
      swarm._swarm.spawn.mockRejectedValue(new Error('Spawn failed'));

      await expect(swarm.spawn({})).rejects.toThrow('Spawn failed');
    });

    it('should handle task orchestration failures', async () => {
      const swarm = await ruvSwarm.createSwarm({ name: 'test-swarm' });

      // Mock orchestration failure
      swarm._swarm.orchestrate.mockRejectedValue(new Error('Orchestration failed'));

      await expect(swarm.orchestrate({})).rejects.toThrow('Orchestration failed');
    });
  });

  describe('Performance', () => {
    beforeEach(async () => {
      ruvSwarm = await RuvSwarm.initialize();
    });

    it('should handle concurrent operations', async () => {
      const swarm = await ruvSwarm.createSwarm({ name: 'performance-test' });

      const tasks = Array.from({ length: 10 }, (_, i) => ({
        type: 'computation',
        data: [i, i + 1],
        operation: 'add'
      }));

      const startTime = Date.now();
      const results = await Promise.all(tasks.map(task => swarm.orchestrate(task)));
      const endTime = Date.now();

      expect(results).toHaveLength(10);
      expect(endTime - startTime).toBeLessThan(5000); // Should complete within 5 seconds
    });

    it('should manage memory efficiently', async () => {
      const swarm = await ruvSwarm.createSwarm({ name: 'memory-test' });

      // Create multiple agents
      const agents = await Promise.all(
        Array.from({ length: 5 }, () => swarm.spawn({ type: 'neural' }))
      );

      expect(agents).toHaveLength(5);
      agents.forEach(agent => {
        expect(agent.id).toBeDefined();
      });
    });

    it('should handle large datasets', async () => {
      const swarm = await ruvSwarm.createSwarm({ name: 'large-data-test' });

      const largeDataset = Array.from({ length: 1000 }, (_, i) => i);
      const task = {
        type: 'computation',
        data: largeDataset,
        operation: 'sum'
      };

      const result = await swarm.orchestrate(task);
      expect(result).toBeDefined();
      expect(result.success).toBe(true);
    });
  });

  describe('Integration', () => {
    it('should work with neural network components', async () => {
      ruvSwarm = await RuvSwarm.initialize();

      const swarm = await ruvSwarm.createSwarm({
        name: 'neural-integration',
        maxAgents: 3
      });

      const neuralAgent = await swarm.spawn({
        type: 'neural',
        capabilities: ['neural-processing', 'pattern-recognition']
      });

      const neuralTask = {
        type: 'neural-computation',
        data: {
          inputs: [0.5, 0.7, 0.2],
          weights: [0.1, 0.3, 0.5]
        },
        operation: 'feedforward'
      };

      const result = await neuralAgent.execute(neuralTask);
      expect(result).toBeDefined();
    });

    it('should support different agent types', async () => {
      ruvSwarm = await RuvSwarm.initialize();
      const swarm = await ruvSwarm.createSwarm({ name: 'multi-agent' });

      const agentTypes = ['neural', 'coordinator', 'worker'];

      const agents = await Promise.all(
        agentTypes.map(type => swarm.spawn({ type }))
      );

      expect(agents).toHaveLength(3);
      agents.forEach((agent, index) => {
        expect(agent.agentType).toBe(agentTypes[index]);
      });
    });
  });

  describe('Security', () => {
    beforeEach(async () => {
      ruvSwarm = await RuvSwarm.initialize();
    });

    it('should validate input parameters', async () => {
      const swarm = await ruvSwarm.createSwarm({ name: 'security-test' });

      const maliciousTask = {
        type: 'computation',
        data: null, // Invalid data
        operation: undefined // Invalid operation
      };

      await expect(swarm.orchestrate(maliciousTask)).rejects.toThrow();
    });

    it('should handle malformed configurations', async () => {
      const malformedConfig = {
        name: 'test-swarm',
        maxAgents: 'not-a-number',
        invalidProperty: 'should-be-ignored'
      };

      await expect(ruvSwarm.createSwarm(malformedConfig)).rejects.toThrow();
    });

    it('should prevent resource exhaustion', async () => {
      const swarm = await ruvSwarm.createSwarm({
        name: 'resource-test',
        maxAgents: 2 // Limited agents
      });

      // Try to spawn more agents than allowed
      const spawnPromises = Array.from({ length: 5 }, () =>
        swarm.spawn({ type: 'worker' })
      );

      const results = await Promise.allSettled(spawnPromises);
      const fulfilled = results.filter(r => r.status === 'fulfilled');
      const rejected = results.filter(r => r.status === 'rejected');

      expect(fulfilled.length).toBeLessThanOrEqual(2);
      expect(rejected.length).toBeGreaterThan(0);
    });
  });
});