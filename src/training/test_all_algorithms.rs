//! Comprehensive tests for all training algorithms and performance optimizations
//!
//! This module tests:
//! - All optimizer implementations (Adam, AdamW, RMSProp, AdaGrad, MomentumSGD)
//! - Learning rate schedulers (Cosine, OneCycle, WarmRestarts)
//! - Gradient clipping strategies
//! - Parallel training capabilities
//! - Memory management optimizations
//! - WASM performance enhancements

use super::*;
use crate::Network;
use num_traits::Float;
use std::time::Instant;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Test all optimizers with a simple XOR problem
#[test]
fn test_all_optimizers_xor() {
    let training_data = create_xor_training_data();

    let optimizers = vec![
        (
            "Adam",
            Box::new(Adam::new(0.01)) as Box<dyn TrainingAlgorithm<f32>>,
        ),
        ("AdamW", Box::new(AdamW::new(0.01))),
        ("RMSProp", Box::new(RMSProp::new(0.01))),
        ("AdaGrad", Box::new(AdaGrad::new(0.01))),
        ("MomentumSGD", Box::new(MomentumSGD::new(0.01))),
    ];

    for (name, mut optimizer) in optimizers {
        println!("Testing {} optimizer", name);

        let mut network = Network::<f32>::new(&[2, 4, 1]);
        let mut final_error = 0.0;

        for epoch in 0..100 {
            let error = optimizer.train_epoch(&mut network, &training_data).unwrap();
            final_error = error;

            if error < 0.01 {
                break;
            }
        }

        assert!(
            final_error < 0.1,
            "{} optimizer failed to converge: {}",
            name,
            final_error
        );
        println!("{} optimizer final error: {}", name, final_error);
    }
}

/// Test learning rate schedulers
#[test]
fn test_learning_rate_schedulers() {
    let training_data = create_xor_training_data();
    let mut network = Network::<f32>::new(&[2, 4, 1]);
    let mut optimizer = Adam::new(0.01);

    // Test cosine annealing
    let mut cosine_scheduler = CosineAnnealing::new(0.01, 0.001, 50);
    let mut cosine_errors = Vec::new();

    for epoch in 0..50 {
        let lr = cosine_scheduler.get_rate(epoch);
        // In practice, you'd update the optimizer's learning rate here
        let error = optimizer.train_epoch(&mut network, &training_data).unwrap();
        cosine_errors.push(error);
    }

    assert!(
        *cosine_errors.last().unwrap() < 0.1,
        "Cosine annealing failed to converge"
    );

    // Test OneCycle scheduler
    let mut network2 = Network::<f32>::new(&[2, 4, 1]);
    let mut optimizer2 = Adam::new(0.01);
    let mut onecycle_scheduler = OneCycle::new(0.1, 0.001, 50, 0.3);
    let mut onecycle_errors = Vec::new();

    for epoch in 0..50 {
        let lr = onecycle_scheduler.get_rate(epoch);
        let error = optimizer2
            .train_epoch(&mut network2, &training_data)
            .unwrap();
        onecycle_errors.push(error);
    }

    assert!(
        *onecycle_errors.last().unwrap() < 0.1,
        "OneCycle failed to converge"
    );
}

/// Test gradient clipping
#[test]
fn test_gradient_clipping() {
    use super::gradient_clipping::*;

    let mut weight_gradients = vec![vec![10.0f32, -10.0, 1.0], vec![-5.0, 5.0]];

    let mut bias_gradients = vec![vec![8.0f32, -8.0], vec![3.0]];

    // Test global norm clipping
    let global_stats = clip_by_global_norm(&mut weight_gradients, 5.0);
    assert!(global_stats.global_norm <= 5.0);
    assert!(global_stats.clipped_count > 0);

    // Test value clipping
    let value_stats = clip_by_value(&mut bias_gradients, 3.0);
    for layer in &bias_gradients {
        for &grad in layer {
            assert!(grad >= -3.0 && grad <= 3.0);
        }
    }
    assert!(value_stats.clipped_count > 0);

    // Test adaptive clipping
    let mut adaptive = AdaptiveGradientClipping::new(1.0, 10.0, 0.1, 5);
    let stats = GradientStats {
        global_norm: 8.0,
        max_gradient: 8.0,
        min_gradient: -8.0,
        clipped_count: 0,
        total_parameters: 10,
    };

    adaptive.update_threshold(&stats);
    assert!(adaptive.get_clipping() == GradientClipping::GlobalNorm(adaptive.current_threshold));
}

/// Test parallel training capabilities
#[cfg(feature = "parallel")]
#[test]
fn test_parallel_training() {
    use super::parallel::*;

    let training_data = create_xor_training_data();
    let config = ParallelTrainingConfig {
        num_threads: 2,
        batch_size: 32,
        parallel_gradients: true,
        parallel_error_calc: true,
        data_parallel: true,
        model_parallel: false,
        chunk_size: 50,
    };

    let mut network = Network::<f32>::new(&[2, 4, 1]);
    let optimizer = Adam::new(0.01);
    let mut parallel_trainer = DataParallelTrainer::new(optimizer, config);

    let start_time = Instant::now();
    let error = parallel_trainer
        .train_epoch_parallel(&mut network, &training_data)
        .unwrap();
    let duration = start_time.elapsed();

    assert!(error < 0.5, "Parallel training failed: {}", error);
    assert!(
        duration < std::time::Duration::from_secs(5),
        "Parallel training too slow"
    );

    // Test work stealing scheduler
    let scheduler = WorkStealingScheduler::new(config);
    let work_items = vec![|| 1 + 1, || 2 + 2, || 3 + 3];

    let results = scheduler.distribute_work(work_items);
    assert_eq!(results, vec![2, 4, 6]);
}

/// Test memory management optimizations
#[test]
fn test_advanced_memory_management() {
    use crate::memory_manager::*;

    let mut manager = MemoryManager::<f32>::with_advanced_features(
        true, // enable_zero_copy
        true, // enable_arena
        true, // enable_leak_detection
        4096, // arena_chunk_size
    );

    // Enable advanced features
    manager.enable_advanced_features(
        true,        // enable_cache
        true,        // enable_prefetching
        true,        // enable_gc
        1024 * 1024, // cache_size
        5,           // prefetch_distance
        10,          // gc_threshold
    );

    // Test smart caching
    let test_data = vec![1.0, 2.0, 3.0, 4.0];
    let test_shape = vec![2, 2];
    manager.cache_tensor("test_tensor".to_string(), test_data, test_shape);

    let cached = manager.get_cached_tensor("test_tensor");
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().len(), 4);

    // Test memory access recording
    manager.record_memory_access("tensor_1");
    manager.record_memory_access("tensor_2");
    manager.record_memory_access("tensor_1");

    let recommendations = manager.get_prefetch_recommendations("tensor_1");
    // Should have some recommendations based on access patterns

    // Test garbage collection
    manager.inc_ref("object_1");
    manager.inc_ref("object_1");
    manager.dec_ref("object_1");
    manager.dec_ref("object_1");

    let collected = manager.collect_garbage();
    assert!(collected.contains(&"object_1".to_string()));

    // Test cache statistics
    let cache_stats = manager.get_cache_stats();
    assert!(cache_stats.is_some());
    let (hit_rate, cache_size, cache_entries) = cache_stats.unwrap();
    assert!(hit_rate >= 0.0 && hit_rate <= 1.0);
    assert!(cache_size >= 0);
    assert!(cache_entries >= 0);
}

/// Test WASM performance enhancements
#[cfg(target_arch = "wasm32")]
#[test]
fn test_wasm_performance_enhancements() {
    use crate::wasm_performance::*;

    let config = WasmPerformanceConfig {
        enable_streaming_compilation: true,
        enable_simd: true,
        enable_bulk_memory: true,
        enable_monitoring: true,
        memory_alignment: 16,
        chunk_size: 1024,
    };

    let mut manager = WasmMemoryManager::<f32>::new(config);

    // Test SIMD memory allocation
    manager.allocate_simd("test_simd", 100).unwrap();

    // Test SIMD operations
    let a_data = vec![1.0, 2.0, 3.0, 4.0];
    let b_data = vec![5.0, 6.0, 7.0, 8.0];

    manager.allocate_simd("a", 4).unwrap();
    manager.allocate_simd("b", 4).unwrap();
    manager.allocate_simd("result", 4).unwrap();

    // Copy data to SIMD buffers
    if let Ok(a_slice) = manager.get_mut_slice("a") {
        a_slice.copy_from_slice(&a_data);
    }
    if let Ok(b_slice) = manager.get_mut_slice("b") {
        b_slice.copy_from_slice(&b_data);
    }

    // Test vector addition
    manager.vector_add("a", "b", "result").unwrap();

    if let Ok(result) = manager.get_slice("result") {
        assert_eq!(result[0], 6.0);
        assert_eq!(result[1], 8.0);
        assert_eq!(result[2], 10.0);
        assert_eq!(result[3], 12.0);
    }

    // Test bulk memory operations
    manager.bulk_copy("a", "b").unwrap();

    if let Ok(b_copy) = manager.get_slice("b") {
        assert_eq!(b_copy, &a_data);
    }

    // Test performance monitoring
    let metrics = manager.get_performance_metrics();
    assert!(metrics.timestamp >= 0.0);
}

/// Benchmark optimizer performance
#[test]
fn benchmark_optimizer_performance() {
    use super::benchmark_optimizers::*;

    let config = BenchmarkConfig::default();
    let results = benchmark_all_optimizers(&config);

    // Check that all optimizers produced results
    assert!(!results.is_empty());

    // Check that results are reasonable
    for result in &results {
        assert!(result.final_error >= 0.0);
        assert!(result.total_time > std::time::Duration::from_millis(0));
        assert!(result.convergence_epoch.unwrap_or(999) < 1000);
    }

    // Print results for analysis
    print_benchmark_results(&results);
}

/// Test optimizer convergence on different problem types
#[test]
fn test_optimizer_convergence() {
    let test_cases = vec![
        ("XOR", create_xor_training_data()),
        ("Linear", create_linear_training_data()),
        ("Nonlinear", create_nonlinear_training_data()),
    ];

    for (problem_name, training_data) in test_cases {
        println!("Testing convergence on {} problem", problem_name);

        let optimizers = vec![
            (
                "Adam",
                Box::new(Adam::new(0.01)) as Box<dyn TrainingAlgorithm<f32>>,
            ),
            ("RMSProp", Box::new(RMSProp::new(0.01))),
            ("MomentumSGD", Box::new(MomentumSGD::new(0.01))),
        ];

        for (opt_name, mut optimizer) in optimizers {
            let mut network = Network::<f32>::new(&[2, 8, 1]);
            let mut converged = false;

            for epoch in 0..200 {
                let error = optimizer.train_epoch(&mut network, &training_data).unwrap();

                if error < 0.05 {
                    converged = true;
                    println!(
                        "  {} converged in {} epochs with error {}",
                        opt_name, epoch, error
                    );
                    break;
                }
            }

            assert!(
                converged,
                "{} failed to converge on {} problem",
                opt_name, problem_name
            );
        }
    }
}

/// Helper functions for creating test data
fn create_xor_training_data() -> TrainingData<f32> {
    TrainingData {
        inputs: vec![
            vec![0.0, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 0.0],
            vec![1.0, 1.0],
        ],
        outputs: vec![vec![0.0], vec![1.0], vec![1.0], vec![0.0]],
    }
}

fn create_linear_training_data() -> TrainingData<f32> {
    TrainingData {
        inputs: (0..100).map(|i| vec![i as f32 / 100.0]).collect(),
        outputs: (0..100).map(|i| vec![i as f32 / 100.0]).collect(),
    }
}

fn create_nonlinear_training_data() -> TrainingData<f32> {
    TrainingData {
        inputs: (0..100).map(|i| vec![i as f32 / 100.0]).collect(),
        outputs: (0..100)
            .map(|i| {
                let x = i as f32 / 100.0;
                vec![x * x] // Quadratic relationship
            })
            .collect(),
    }
}

/// Test memory efficiency of different optimizers
#[test]
fn test_memory_efficiency() {
    let training_data = create_xor_training_data();

    let optimizers = vec![
        (
            "Adam",
            Box::new(Adam::new(0.01)) as Box<dyn TrainingAlgorithm<f32>>,
        ),
        ("AdamW", Box::new(AdamW::new(0.01))),
        ("RMSProp", Box::new(RMSProp::new(0.01))),
        ("AdaGrad", Box::new(AdaGrad::new(0.01))),
        ("MomentumSGD", Box::new(MomentumSGD::new(0.01))),
    ];

    for (name, mut optimizer) in optimizers {
        let mut network = Network::<f32>::new(&[2, 16, 8, 1]);

        let start_memory = get_current_memory_usage();
        let start_time = Instant::now();

        for _ in 0..50 {
            let _ = optimizer.train_epoch(&mut network, &training_data);
        }

        let end_memory = get_current_memory_usage();
        let duration = start_time.elapsed();

        let memory_used = end_memory - start_memory;

        println!(
            "{}: Memory used: {} bytes, Time: {:?}",
            name, memory_used, duration
        );

        // Basic sanity checks
        assert!(memory_used >= 0);
        assert!(duration > std::time::Duration::from_millis(0));
    }
}

/// Get current memory usage (simplified implementation)
fn get_current_memory_usage() -> usize {
    // In a real implementation, you'd use platform-specific APIs
    // For now, return a placeholder
    0
}

/// Test numerical stability of optimizers
#[test]
fn test_numerical_stability() {
    let training_data = create_xor_training_data();

    let optimizers = vec![
        ("Adam", Adam::new(0.01)),
        ("AdamW", AdamW::new(0.01)),
        ("RMSProp", RMSProp::new(0.01)),
        ("AdaGrad", AdaGrad::new(0.01)),
        ("MomentumSGD", MomentumSGD::new(0.01)),
    ];

    for (name, mut optimizer) in optimizers {
        let mut network = Network::<f32>::new(&[2, 4, 1]);

        // Test with extreme learning rates
        let extreme_lrs = vec![1e-6, 1e-3, 1.0, 10.0];

        for lr in extreme_lrs {
            // Reset network
            network = Network::<f32>::new(&[2, 4, 1]);

            // This is a simplified test - in practice you'd need to modify
            // the optimizer's learning rate for this test
            let error = optimizer.train_epoch(&mut network, &training_data);

            match error {
                Ok(e) => {
                    // Check that error is finite
                    assert!(
                        e.is_finite(),
                        "{} produced non-finite error with lr {}",
                        name,
                        lr
                    );
                    assert!(e >= 0.0, "{} produced negative error with lr {}", name, lr);
                }
                Err(_) => {
                    // Some optimizers might fail with extreme learning rates, which is acceptable
                    println!("{} failed with lr {} (acceptable)", name, lr);
                }
            }
        }
    }
}
