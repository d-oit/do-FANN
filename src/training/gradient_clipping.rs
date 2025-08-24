//! Gradient clipping utilities for neural network training
//!
//! Gradient clipping prevents exploding gradients by scaling gradients when they exceed a threshold.
//! This is crucial for training stability, especially with recurrent networks and deep architectures.
//!
//! Supported clipping methods:
//! - Global norm clipping: Scales all gradients so their global norm doesn't exceed a threshold
//! - Per-parameter clipping: Clips each parameter's gradient individually
//! - Value clipping: Clips gradient values to a specified range
//!
//! Expected benefits:
//! - Improved training stability
//! - Prevention of gradient explosion
//! - Better convergence on difficult optimization landscapes

use num_traits::Float;
use std::collections::HashMap;

/// Gradient clipping strategy
#[derive(Debug, Clone)]
pub enum GradientClipping<T: Float> {
    /// No gradient clipping
    None,
    /// Clip by global norm - scales all gradients so their L2 norm doesn't exceed threshold
    GlobalNorm(T),
    /// Clip each gradient value individually to [-threshold, threshold]
    Value(T),
    /// Clip by per-parameter L2 norm
    PerParameter(T),
}

/// Gradient statistics for monitoring
#[derive(Debug, Clone)]
pub struct GradientStats<T: Float> {
    pub global_norm: T,
    pub max_gradient: T,
    pub min_gradient: T,
    pub clipped_count: usize,
    pub total_parameters: usize,
}

/// Apply gradient clipping to weight gradients
pub fn clip_weight_gradients<T: Float>(
    weight_gradients: &mut [Vec<T>],
    clipping: &GradientClipping<T>,
) -> GradientStats<T> {
    match clipping {
        GradientClipping::None => compute_gradient_stats(weight_gradients),
        GradientClipping::GlobalNorm(threshold) => {
            clip_by_global_norm(weight_gradients, *threshold)
        }
        GradientClipping::Value(threshold) => clip_by_value(weight_gradients, *threshold),
        GradientClipping::PerParameter(threshold) => {
            clip_per_parameter(weight_gradients, *threshold)
        }
    }
}

/// Apply gradient clipping to bias gradients
pub fn clip_bias_gradients<T: Float>(
    bias_gradients: &mut [Vec<T>],
    clipping: &GradientClipping<T>,
) -> GradientStats<T> {
    match clipping {
        GradientClipping::None => compute_gradient_stats(bias_gradients),
        GradientClipping::GlobalNorm(threshold) => clip_by_global_norm(bias_gradients, *threshold),
        GradientClipping::Value(threshold) => clip_by_value(bias_gradients, *threshold),
        GradientClipping::PerParameter(threshold) => clip_per_parameter(bias_gradients, *threshold),
    }
}

/// Compute gradient statistics without clipping
fn compute_gradient_stats<T: Float>(gradients: &[Vec<T>]) -> GradientStats<T> {
    let mut global_norm = T::zero();
    let mut max_gradient = T::from(f64::NEG_INFINITY).unwrap();
    let mut min_gradient = T::from(f64::INFINITY).unwrap();
    let mut total_parameters = 0;

    for layer_gradients in gradients {
        for &grad in layer_gradients {
            global_norm = global_norm + grad * grad;
            max_gradient = max_gradient.max(grad);
            min_gradient = min_gradient.min(grad);
            total_parameters += 1;
        }
    }

    global_norm = global_norm.sqrt();

    GradientStats {
        global_norm,
        max_gradient,
        min_gradient,
        clipped_count: 0,
        total_parameters,
    }
}

/// Clip gradients by global L2 norm
fn clip_by_global_norm<T: Float>(gradients: &mut [Vec<T>], threshold: T) -> GradientStats<T> {
    let mut stats = compute_gradient_stats(gradients);

    if stats.global_norm > threshold && stats.global_norm > T::zero() {
        let scale_factor = threshold / stats.global_norm;

        for layer_gradients in gradients.iter_mut() {
            for grad in layer_gradients.iter_mut() {
                *grad = *grad * scale_factor;
            }
        }

        stats.clipped_count = stats.total_parameters;
        stats.global_norm = threshold;
    }

    stats
}

/// Clip each gradient value individually
fn clip_by_value<T: Float>(gradients: &mut [Vec<T>], threshold: T) -> GradientStats<T> {
    let mut stats = compute_gradient_stats(gradients);
    let mut clipped_count = 0;

    for layer_gradients in gradients.iter_mut() {
        for grad in layer_gradients.iter_mut() {
            if grad.abs() > threshold {
                *grad = if *grad > T::zero() {
                    threshold
                } else {
                    -threshold
                };
                clipped_count += 1;
            }
        }
    }

    stats.clipped_count = clipped_count;
    stats
}

/// Clip by per-parameter L2 norm
fn clip_per_parameter<T: Float>(gradients: &mut [Vec<T>], threshold: T) -> GradientStats<T> {
    let mut stats = compute_gradient_stats(gradients);
    let mut clipped_count = 0;

    for layer_gradients in gradients.iter_mut() {
        for grad in layer_gradients.iter_mut() {
            let grad_norm = grad.abs();
            if grad_norm > threshold {
                *grad = *grad * threshold / grad_norm;
                clipped_count += 1;
            }
        }
    }

    stats.clipped_count = clipped_count;
    stats
}

/// Adaptive gradient clipping that adjusts threshold based on training progress
pub struct AdaptiveGradientClipping<T: Float> {
    base_threshold: T,
    max_threshold: T,
    adaptation_factor: T,
    current_threshold: T,
    gradient_history: Vec<T>,
    history_size: usize,
}

impl<T: Float> AdaptiveGradientClipping<T> {
    pub fn new(
        base_threshold: T,
        max_threshold: T,
        adaptation_factor: T,
        history_size: usize,
    ) -> Self {
        Self {
            base_threshold,
            max_threshold,
            adaptation_factor,
            current_threshold: base_threshold,
            gradient_history: Vec::with_capacity(history_size),
            history_size,
        }
    }

    pub fn get_clipping(&mut self) -> GradientClipping<T> {
        GradientClipping::GlobalNorm(self.current_threshold)
    }

    pub fn update_threshold(&mut self, stats: &GradientStats<T>) {
        // Add current gradient norm to history
        self.gradient_history.push(stats.global_norm);

        // Keep history size limited
        if self.gradient_history.len() > self.history_size {
            self.gradient_history.remove(0);
        }

        // Compute average gradient norm over history
        if !self.gradient_history.is_empty() {
            let avg_norm = self
                .gradient_history
                .iter()
                .fold(T::zero(), |sum, &x| sum + x)
                / T::from(self.gradient_history.len()).unwrap();

            // Adapt threshold based on average gradient norm
            let new_threshold = self.base_threshold + self.adaptation_factor * avg_norm;

            // Clamp to reasonable bounds
            self.current_threshold = new_threshold.min(self.max_threshold);
        }
    }

    pub fn reset(&mut self) {
        self.current_threshold = self.base_threshold;
        self.gradient_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_norm_clipping() {
        let mut gradients = vec![vec![1.0f32, 2.0, 3.0], vec![4.0, 5.0]];

        let stats = clip_by_global_norm(&mut gradients, 5.0);

        // Global norm should be <= threshold
        assert!(stats.global_norm <= 5.0);
        assert_eq!(stats.clipped_count, 5); // All parameters clipped
    }

    #[test]
    fn test_value_clipping() {
        let mut gradients = vec![vec![10.0f32, -10.0, 1.0], vec![-5.0, 5.0]];

        let stats = clip_by_value(&mut gradients, 3.0);

        // Check that values are within bounds
        for layer in &gradients {
            for &grad in layer {
                assert!(grad >= -3.0 && grad <= 3.0);
            }
        }

        assert_eq!(stats.clipped_count, 4); // 4 values were clipped
    }

    #[test]
    fn test_no_clipping() {
        let mut gradients = vec![vec![0.1f32, 0.2, 0.3], vec![0.4, 0.5]];

        let stats = compute_gradient_stats(&gradients);

        assert!(stats.global_norm > 0.0);
        assert_eq!(stats.clipped_count, 0);
        assert_eq!(stats.total_parameters, 5);
    }

    #[test]
    fn test_adaptive_clipping() {
        let mut adaptive = AdaptiveGradientClipping::new(1.0f32, 10.0, 0.1, 5);

        let stats = GradientStats {
            global_norm: 5.0,
            max_gradient: 5.0,
            min_gradient: -5.0,
            clipped_count: 0,
            total_parameters: 10,
        };

        adaptive.update_threshold(&stats);

        // Threshold should have increased
        assert!(adaptive.current_threshold > 1.0);
        assert!(adaptive.current_threshold <= 10.0);
    }
}
