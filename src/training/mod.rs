//! Training algorithms for neural networks
//!
//! This module implements various training algorithms including:
//! - Incremental (online) backpropagation
//! - Batch backpropagation
//! - RPROP (Resilient Propagation)
//! - Quickprop
//!
//! All training algorithms implement the `TrainingAlgorithm` trait for extensibility.

#![allow(clippy::needless_range_loop)]

use crate::Network;
use num_traits::Float;
use std::collections::HashMap;
use thiserror::Error;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct TrainingData<T: Float> {
    pub inputs: Vec<Vec<T>>,
    pub outputs: Vec<Vec<T>>,
}

/// Options for parallel training
#[derive(Debug, Clone)]
pub struct ParallelTrainingOptions {
    /// Number of threads to use (0 = use all available cores)
    pub num_threads: usize,
    /// Batch size for parallel processing
    pub batch_size: usize,
    /// Whether to use parallel gradient computation
    pub parallel_gradients: bool,
    /// Whether to use parallel error calculation
    pub parallel_error_calc: bool,
}

impl Default for ParallelTrainingOptions {
    fn default() -> Self {
        Self {
            num_threads: 0, // Use all available cores
            batch_size: 32,
            parallel_gradients: true,
            parallel_error_calc: true,
        }
    }
}

/// Error types for training operations
#[derive(Error, Debug)]
pub enum TrainingError {
    #[error("Invalid training data: {0}")]
    InvalidData(String),

    #[error("Network configuration error: {0}")]
    NetworkError(String),

    #[error("Training failed: {0}")]
    TrainingFailed(String),
}

/// Trait for error/loss functions
pub trait ErrorFunction<T: Float>: Send + Sync {
    /// Calculate the error between actual and desired outputs
    fn calculate(&self, actual: &[T], desired: &[T]) -> T;

    /// Calculate the derivative of the error function
    fn derivative(&self, actual: T, desired: T) -> T;
}

/// Mean Squared Error (MSE)
#[derive(Clone)]
pub struct MseError;

impl<T: Float> ErrorFunction<T> for MseError {
    fn calculate(&self, actual: &[T], desired: &[T]) -> T {
        let sum = actual
            .iter()
            .zip(desired.iter())
            .map(|(&a, &d)| {
                let diff = a - d;
                diff * diff
            })
            .fold(T::zero(), |acc, x| acc + x);
        sum / T::from(actual.len()).unwrap()
    }

    fn derivative(&self, actual: T, desired: T) -> T {
        T::from(2.0).unwrap() * (actual - desired)
    }
}

/// Mean Absolute Error (MAE)
#[derive(Clone)]
pub struct MaeError;

impl<T: Float> ErrorFunction<T> for MaeError {
    fn calculate(&self, actual: &[T], desired: &[T]) -> T {
        let sum = actual
            .iter()
            .zip(desired.iter())
            .map(|(&a, &d)| (a - d).abs())
            .fold(T::zero(), |acc, x| acc + x);
        sum / T::from(actual.len()).unwrap()
    }

    fn derivative(&self, actual: T, desired: T) -> T {
        if actual > desired {
            T::one()
        } else if actual < desired {
            -T::one()
        } else {
            T::zero()
        }
    }
}

/// Tanh Error Function
#[derive(Clone)]
pub struct TanhError;

impl<T: Float> ErrorFunction<T> for TanhError {
    fn calculate(&self, actual: &[T], desired: &[T]) -> T {
        let sum = actual
            .iter()
            .zip(desired.iter())
            .map(|(&a, &d)| {
                let diff = a - d;
                let tanh_diff = diff.tanh();
                tanh_diff * tanh_diff
            })
            .fold(T::zero(), |acc, x| acc + x);
        sum / T::from(actual.len()).unwrap()
    }

    fn derivative(&self, actual: T, desired: T) -> T {
        let diff = actual - desired;
        let tanh_diff = diff.tanh();
        T::from(2.0).unwrap() * tanh_diff * (T::one() - tanh_diff * tanh_diff)
    }
}

/// Learning rate schedule trait
pub trait LearningRateSchedule<T: Float> {
    fn get_rate(&mut self, epoch: usize) -> T;
}

/// Advanced learning rate scheduler with additional capabilities
pub trait AdvancedLearningRateSchedule<T: Float>: LearningRateSchedule<T> {
    /// Get the current learning rate without updating internal state
    fn peek_rate(&self, epoch: usize) -> T;

    /// Reset the scheduler to initial state
    fn reset(&mut self);

    /// Get scheduler-specific metrics
    fn metrics(&self) -> HashMap<String, T> {
        HashMap::new()
    }
}

// Learning rate schedulers are defined below and exported at module level

/// Exponential decay learning rate schedule
pub struct ExponentialDecay<T: Float> {
    initial_rate: T,
    decay_rate: T,
}

impl<T: Float> ExponentialDecay<T> {
    pub fn new(initial_rate: T, decay_rate: T) -> Self {
        Self {
            initial_rate,
            decay_rate,
        }
    }
}

impl<T: Float> LearningRateSchedule<T> for ExponentialDecay<T> {
    fn get_rate(&mut self, epoch: usize) -> T {
        self.initial_rate * self.decay_rate.powi(epoch as i32)
    }
}

impl<T: Float> AdvancedLearningRateSchedule<T> for ExponentialDecay<T> {
    fn peek_rate(&self, epoch: usize) -> T {
        self.initial_rate * self.decay_rate.powi(epoch as i32)
    }

    fn reset(&mut self) {
        // No internal state to reset
    }

    fn metrics(&self) -> HashMap<String, T> {
        let mut metrics = HashMap::new();
        metrics.insert("initial_rate".to_string(), self.initial_rate);
        metrics.insert("decay_rate".to_string(), self.decay_rate);
        metrics
    }
}

/// Step decay learning rate schedule
pub struct StepDecay<T: Float> {
    initial_rate: T,
    drop_rate: T,
    epochs_per_drop: usize,
}

impl<T: Float> StepDecay<T> {
    pub fn new(initial_rate: T, drop_rate: T, epochs_per_drop: usize) -> Self {
        Self {
            initial_rate,
            drop_rate,
            epochs_per_drop,
        }
    }
}

impl<T: Float> LearningRateSchedule<T> for StepDecay<T> {
    fn get_rate(&mut self, epoch: usize) -> T {
        let drops = epoch / self.epochs_per_drop;
        self.initial_rate * self.drop_rate.powi(drops as i32)
    }
}

impl<T: Float> AdvancedLearningRateSchedule<T> for StepDecay<T> {
    fn peek_rate(&self, epoch: usize) -> T {
        let drops = epoch / self.epochs_per_drop;
        self.initial_rate * self.drop_rate.powi(drops as i32)
    }

    fn reset(&mut self) {
        // No internal state to reset
    }

    fn metrics(&self) -> HashMap<String, T> {
        let mut metrics = HashMap::new();
        metrics.insert("initial_rate".to_string(), self.initial_rate);
        metrics.insert("drop_rate".to_string(), self.drop_rate);
        metrics.insert(
            "epochs_per_drop".to_string(),
            T::from(self.epochs_per_drop).unwrap(),
        );
        metrics
    }
}

/// Cosine annealing learning rate schedule
/// Gradually decreases learning rate following a cosine curve
pub struct CosineAnnealing<T: Float> {
    initial_rate: T,
    min_rate: T,
    total_epochs: usize,
}

impl<T: Float> CosineAnnealing<T> {
    pub fn new(initial_rate: T, min_rate: T, total_epochs: usize) -> Self {
        Self {
            initial_rate,
            min_rate,
            total_epochs,
        }
    }
}

impl<T: Float> LearningRateSchedule<T> for CosineAnnealing<T> {
    fn get_rate(&mut self, epoch: usize) -> T {
        if epoch >= self.total_epochs {
            return self.min_rate;
        }

        let progress = T::from(epoch).unwrap() / T::from(self.total_epochs).unwrap();
        let cosine = (T::from(std::f64::consts::PI).unwrap() * progress).cos();

        let rate_range = self.initial_rate - self.min_rate;
        self.min_rate + rate_range * (T::one() + cosine) / (T::one() + T::one())
    }
}

impl<T: Float> AdvancedLearningRateSchedule<T> for CosineAnnealing<T> {
    fn peek_rate(&self, epoch: usize) -> T {
        if epoch >= self.total_epochs {
            return self.min_rate;
        }

        let progress = T::from(epoch).unwrap() / T::from(self.total_epochs).unwrap();
        let cosine = (T::from(std::f64::consts::PI).unwrap() * progress).cos();

        let rate_range = self.initial_rate - self.min_rate;
        self.min_rate + rate_range * (T::one() + cosine) / (T::one() + T::one())
    }

    fn reset(&mut self) {
        // No internal state to reset
    }

    fn metrics(&self) -> HashMap<String, T> {
        let mut metrics = HashMap::new();
        metrics.insert("initial_rate".to_string(), self.initial_rate);
        metrics.insert("min_rate".to_string(), self.min_rate);
        metrics.insert(
            "total_epochs".to_string(),
            T::from(self.total_epochs).unwrap(),
        );
        metrics
    }
}

/// Warm restarts learning rate schedule
/// Periodically resets learning rate to initial value with cosine annealing
pub struct WarmRestarts<T: Float> {
    initial_rate: T,
    min_rate: T,
    restart_period: usize,
    current_period: usize,
}

impl<T: Float> WarmRestarts<T> {
    pub fn new(initial_rate: T, min_rate: T, restart_period: usize) -> Self {
        Self {
            initial_rate,
            min_rate,
            restart_period,
            current_period: 0,
        }
    }
}

impl<T: Float> LearningRateSchedule<T> for WarmRestarts<T> {
    fn get_rate(&mut self, epoch: usize) -> T {
        let cycle_epoch = epoch % self.restart_period;
        self.current_period = epoch / self.restart_period;

        if cycle_epoch == 0 && epoch > 0 {
            // Reset to initial rate at restart
            return self.initial_rate;
        }

        let progress = T::from(cycle_epoch).unwrap() / T::from(self.restart_period).unwrap();
        let cosine = (T::from(std::f64::consts::PI).unwrap() * progress).cos();

        let rate_range = self.initial_rate - self.min_rate;
        self.min_rate + rate_range * (T::one() + cosine) / (T::one() + T::one())
    }
}

impl<T: Float> AdvancedLearningRateSchedule<T> for WarmRestarts<T> {
    fn peek_rate(&self, epoch: usize) -> T {
        let cycle_epoch = epoch % self.restart_period;

        if cycle_epoch == 0 && epoch > 0 {
            return self.initial_rate;
        }

        let progress = T::from(cycle_epoch).unwrap() / T::from(self.restart_period).unwrap();
        let cosine = (T::from(std::f64::consts::PI).unwrap() * progress).cos();

        let rate_range = self.initial_rate - self.min_rate;
        self.min_rate + rate_range * (T::one() + cosine) / (T::one() + T::one())
    }

    fn reset(&mut self) {
        self.current_period = 0;
    }

    fn metrics(&self) -> HashMap<String, T> {
        let mut metrics = HashMap::new();
        metrics.insert("initial_rate".to_string(), self.initial_rate);
        metrics.insert("min_rate".to_string(), self.min_rate);
        metrics.insert(
            "restart_period".to_string(),
            T::from(self.restart_period).unwrap(),
        );
        metrics.insert(
            "current_period".to_string(),
            T::from(self.current_period).unwrap(),
        );
        metrics
    }
}

/// OneCycle learning rate schedule
/// Increases learning rate to a maximum then decreases following cosine annealing
pub struct OneCycle<T: Float> {
    max_rate: T,
    min_rate: T,
    total_epochs: usize,
    pct_start: T, // Percentage of epochs for increasing phase
}

impl<T: Float> OneCycle<T> {
    pub fn new(max_rate: T, min_rate: T, total_epochs: usize, pct_start: T) -> Self {
        Self {
            max_rate,
            min_rate,
            total_epochs,
            pct_start,
        }
    }
}

impl<T: Float> LearningRateSchedule<T> for OneCycle<T> {
    fn get_rate(&mut self, epoch: usize) -> T {
        let progress = T::from(epoch).unwrap() / T::from(self.total_epochs).unwrap();

        if progress <= self.pct_start {
            // Increasing phase - linear increase from min_rate to max_rate
            let phase_progress = progress / self.pct_start;
            self.min_rate + (self.max_rate - self.min_rate) * phase_progress
        } else {
            // Decreasing phase - cosine annealing from max_rate to min_rate
            let phase_progress = (progress - self.pct_start) / (T::one() - self.pct_start);
            let cosine = (T::from(std::f64::consts::PI).unwrap() * phase_progress).cos();

            let rate_range = self.max_rate - self.min_rate;
            self.min_rate + rate_range * (T::one() + cosine) / (T::one() + T::one())
        }
    }
}

impl<T: Float> AdvancedLearningRateSchedule<T> for OneCycle<T> {
    fn peek_rate(&self, epoch: usize) -> T {
        let progress = T::from(epoch).unwrap() / T::from(self.total_epochs).unwrap();

        if progress <= self.pct_start {
            let phase_progress = progress / self.pct_start;
            self.min_rate + (self.max_rate - self.min_rate) * phase_progress
        } else {
            let phase_progress = (progress - self.pct_start) / (T::one() - self.pct_start);
            let cosine = (T::from(std::f64::consts::PI).unwrap() * phase_progress).cos();

            let rate_range = self.max_rate - self.min_rate;
            self.min_rate + rate_range * (T::one() + cosine) / (T::one() + T::one())
        }
    }

    fn reset(&mut self) {
        // No internal state to reset
    }

    fn metrics(&self) -> HashMap<String, T> {
        let mut metrics = HashMap::new();
        metrics.insert("max_rate".to_string(), self.max_rate);
        metrics.insert("min_rate".to_string(), self.min_rate);
        metrics.insert(
            "total_epochs".to_string(),
            T::from(self.total_epochs).unwrap(),
        );
        metrics.insert("pct_start".to_string(), self.pct_start);
        metrics
    }
}

/// Training state that can be saved and restored
#[derive(Clone, Debug)]
pub struct TrainingState<T: Float> {
    pub epoch: usize,
    pub best_error: T,
    pub algorithm_specific: HashMap<String, Vec<T>>,
}

/// Stop criteria trait
pub trait StopCriteria<T: Float> {
    fn should_stop(
        &self,
        trainer: &dyn TrainingAlgorithm<T>,
        network: &Network<T>,
        data: &TrainingData<T>,
        epoch: usize,
    ) -> bool;
}

/// MSE-based stop criteria
pub struct MseStopCriteria<T: Float> {
    pub target_error: T,
}

impl<T: Float> StopCriteria<T> for MseStopCriteria<T> {
    fn should_stop(
        &self,
        trainer: &dyn TrainingAlgorithm<T>,
        network: &Network<T>,
        data: &TrainingData<T>,
        _epoch: usize,
    ) -> bool {
        let error = trainer.calculate_error(network, data);
        error <= self.target_error
    }
}

/// Bit fail based stop criteria
pub struct BitFailStopCriteria<T: Float> {
    pub target_bit_fail: usize,
    pub bit_fail_limit: T,
}

impl<T: Float> StopCriteria<T> for BitFailStopCriteria<T> {
    fn should_stop(
        &self,
        trainer: &dyn TrainingAlgorithm<T>,
        network: &Network<T>,
        data: &TrainingData<T>,
        _epoch: usize,
    ) -> bool {
        let bit_fails = trainer.count_bit_fails(network, data, self.bit_fail_limit);
        bit_fails <= self.target_bit_fail
    }
}

/// Callback function type for training progress
pub type TrainingCallback<T> = Box<dyn FnMut(usize, T) -> bool + Send>;

/// Main trait for training algorithms
pub trait TrainingAlgorithm<T: Float>: Send {
    /// Train for one epoch
    fn train_epoch(
        &mut self,
        network: &mut Network<T>,
        data: &TrainingData<T>,
    ) -> Result<T, TrainingError>;

    /// Calculate the current error
    fn calculate_error(&self, network: &Network<T>, data: &TrainingData<T>) -> T;

    /// Count bit fails
    fn count_bit_fails(
        &self,
        network: &Network<T>,
        data: &TrainingData<T>,
        bit_fail_limit: T,
    ) -> usize;

    /// Save training state
    fn save_state(&self) -> TrainingState<T>;

    /// Restore training state
    fn restore_state(&mut self, state: TrainingState<T>);

    /// Set a callback function
    fn set_callback(&mut self, callback: TrainingCallback<T>);

    /// Call the callback if set
    fn call_callback(&mut self, epoch: usize, network: &Network<T>, data: &TrainingData<T>)
        -> bool;

    /// Get algorithm name for logging
    fn name(&self) -> &str {
        "Unknown"
    }

    /// Get algorithm-specific metrics
    fn metrics(&self) -> HashMap<String, T> {
        HashMap::new()
    }
}

/// Enhanced training algorithm with additional capabilities
pub trait AdvancedTrainingAlgorithm<T: Float>: TrainingAlgorithm<T> {
    /// Train with early stopping
    fn train_with_early_stopping(
        &mut self,
        network: &mut Network<T>,
        training_data: &TrainingData<T>,
        validation_data: &TrainingData<T>,
        max_epochs: usize,
        patience: usize,
    ) -> Result<TrainingResult<T>, TrainingError>;

    /// Train with learning rate scheduling
    fn train_with_lr_schedule(
        &mut self,
        network: &mut Network<T>,
        data: &TrainingData<T>,
        lr_schedule: &mut dyn LearningRateSchedule<T>,
    ) -> Result<TrainingResult<T>, TrainingError>;

    /// Get training statistics
    fn statistics(&self) -> TrainingStatistics<T>;
}

/// Training result with comprehensive metrics
#[derive(Debug, Clone)]
pub struct TrainingResult<T: Float> {
    pub final_error: T,
    pub epochs_trained: usize,
    pub total_time: std::time::Duration,
    pub learning_curve: Vec<T>,
    pub best_epoch: usize,
    pub early_stopped: bool,
}

/// Training statistics for monitoring
#[derive(Debug, Clone)]
pub struct TrainingStatistics<T: Float> {
    pub epochs_completed: usize,
    pub total_samples_processed: usize,
    pub average_epoch_time: std::time::Duration,
    pub peak_memory_usage: usize,
    pub gradient_norm_history: Vec<T>,
}

// Module declarations for specific algorithms
mod adagrad;
mod adam;
mod backprop;
mod gradient_clipping;
mod momentum_sgd;
mod parallel;
mod quickprop;
mod rmsprop;
mod rprop;

// GPU training module (when GPU features are enabled)
#[cfg(feature = "gpu")]
mod gpu_backprop;
#[cfg(feature = "gpu")]
mod gpu_batch_training;
#[cfg(feature = "gpu")]
mod gpu_training;

// Re-export main types
pub use adagrad::AdaGrad;
pub use adam::{Adam, AdamW};
pub use backprop::{BatchBackprop, IncrementalBackprop};
pub use gradient_clipping::{
    clip_bias_gradients, clip_weight_gradients, AdaptiveGradientClipping, GradientClipping,
    GradientStats,
};
pub use momentum_sgd::MomentumSGD;
pub use parallel::{
    DataParallelTrainer, ParallelTrainingConfig, TrainingThreadPool, WorkStealingScheduler,
};
pub use quickprop::Quickprop;
pub use rmsprop::RMSProp;
pub use rprop::Rprop;

// Re-export GPU training types when available
#[cfg(feature = "gpu")]
pub use gpu_training::{
    get_gpu_capabilities, is_gpu_available, GpuAdam, GpuAdamW, GpuBatchBackprop,
    GpuPerformanceStats,
};

/// Helper functions for forward propagation and gradient calculation
pub mod helpers {
    use super::*;

    /// Simple network representation for training algorithms
    #[derive(Debug, Clone)]
    pub struct SimpleNetwork<T: Float> {
        pub layer_sizes: Vec<usize>,
        pub weights: Vec<Vec<T>>,
        pub biases: Vec<Vec<T>>,
    }

    /// Convert a real Network to a simplified representation for training
    pub fn network_to_simple<T: Float + Default>(network: &Network<T>) -> SimpleNetwork<T> {
        let layer_sizes: Vec<usize> = network
            .layers
            .iter()
            .map(|layer| layer.num_regular_neurons())
            .collect();

        // Extract weights and biases from the complex structure
        let mut weights = Vec::new();
        let mut biases = Vec::new();

        for layer_idx in 1..network.layers.len() {
            let current_layer = &network.layers[layer_idx];
            let _prev_layer_size = network.layers[layer_idx - 1].size(); // Include bias neurons

            let mut layer_weights = Vec::new();
            let mut layer_biases = Vec::new();

            for neuron in &current_layer.neurons {
                if !neuron.is_bias {
                    // Extract bias (connection index 0 should be bias)
                    let bias = if !neuron.connections.is_empty() {
                        neuron.connections[0].weight
                    } else {
                        T::zero()
                    };
                    layer_biases.push(bias);

                    // Extract weights (skip bias connection)
                    for connection in neuron.connections.iter().skip(1) {
                        layer_weights.push(connection.weight);
                    }
                }
            }

            weights.push(layer_weights);
            biases.push(layer_biases);
        }

        SimpleNetwork {
            layer_sizes,
            weights,
            biases,
        }
    }

    /// Apply weight and bias updates back to the real Network
    pub fn apply_updates_to_network<T: Float>(
        network: &mut Network<T>,
        weight_updates: &[Vec<T>],
        bias_updates: &[Vec<T>],
    ) {
        for layer_idx in 1..network.layers.len() {
            let current_layer = &mut network.layers[layer_idx];
            let weight_layer_idx = layer_idx - 1;

            let mut neuron_idx = 0;
            let mut weight_idx = 0;

            for neuron in &mut current_layer.neurons {
                if !neuron.is_bias {
                    // Update bias (connection index 0)
                    if !neuron.connections.is_empty() {
                        neuron.connections[0].weight = neuron.connections[0].weight
                            + bias_updates[weight_layer_idx][neuron_idx];
                    }

                    // Update weights (skip bias connection)
                    for connection in neuron.connections.iter_mut().skip(1) {
                        connection.weight =
                            connection.weight + weight_updates[weight_layer_idx][weight_idx];
                        weight_idx += 1;
                    }

                    neuron_idx += 1;
                }
            }
        }
    }

    /// Activation function that works with our simplified representation
    pub fn sigmoid<T: Float>(x: T) -> T {
        T::one() / (T::one() + (-x).exp())
    }

    /// Sigmoid derivative
    pub fn sigmoid_derivative<T: Float>(output: T) -> T {
        output * (T::one() - output)
    }

    /// Forward propagation through the simplified network
    pub fn forward_propagate<T: Float>(network: &SimpleNetwork<T>, input: &[T]) -> Vec<Vec<T>> {
        let mut activations = vec![input.to_vec()];

        for layer_idx in 1..network.layer_sizes.len() {
            let prev_activations = &activations[layer_idx - 1];
            let weights = &network.weights[layer_idx - 1];
            let biases = &network.biases[layer_idx - 1];

            let mut layer_activations = Vec::with_capacity(network.layer_sizes[layer_idx]);

            for neuron_idx in 0..network.layer_sizes[layer_idx] {
                let mut sum = biases[neuron_idx];
                let weight_start = neuron_idx * prev_activations.len();

                for (input_idx, &input_val) in prev_activations.iter().enumerate() {
                    if weight_start + input_idx < weights.len() {
                        sum = sum + input_val * weights[weight_start + input_idx];
                    }
                }

                layer_activations.push(sigmoid(sum));
            }

            activations.push(layer_activations);
        }

        activations
    }

    /// Calculate gradients using backpropagation on simplified network
    pub fn calculate_gradients<T: Float>(
        network: &SimpleNetwork<T>,
        activations: &[Vec<T>],
        desired_output: &[T],
        error_function: &dyn ErrorFunction<T>,
    ) -> (Vec<Vec<T>>, Vec<Vec<T>>) {
        let mut weight_gradients = network
            .weights
            .iter()
            .map(|w| vec![T::zero(); w.len()])
            .collect::<Vec<_>>();
        let mut bias_gradients = network
            .biases
            .iter()
            .map(|b| vec![T::zero(); b.len()])
            .collect::<Vec<_>>();

        // Initialize errors for each layer
        let mut layer_errors = vec![vec![]; network.layer_sizes.len()];

        // Calculate output layer errors
        let output_idx = activations.len() - 1;
        layer_errors[output_idx] = activations[output_idx]
            .iter()
            .zip(desired_output.iter())
            .map(|(&actual, &desired)| {
                error_function.derivative(actual, desired) * sigmoid_derivative(actual)
            })
            .collect();

        // Backpropagate errors to hidden layers
        for layer_idx in (1..network.layer_sizes.len() - 1).rev() {
            layer_errors[layer_idx] = vec![T::zero(); network.layer_sizes[layer_idx]];

            for neuron_idx in 0..network.layer_sizes[layer_idx] {
                let mut error_sum = T::zero();

                // Sum weighted errors from next layer
                let next_layer_idx = layer_idx + 1;
                let next_layer_weights_idx = layer_idx; // weights[i] connects layer i to layer i+1

                for next_neuron_idx in 0..network.layer_sizes[next_layer_idx] {
                    // Weight from current neuron to next layer neuron
                    let weight_idx = next_neuron_idx * network.layer_sizes[layer_idx] + neuron_idx;
                    if weight_idx < network.weights[next_layer_weights_idx].len() {
                        error_sum = error_sum
                            + layer_errors[next_layer_idx][next_neuron_idx]
                                * network.weights[next_layer_weights_idx][weight_idx];
                    }
                }

                layer_errors[layer_idx][neuron_idx] =
                    error_sum * sigmoid_derivative(activations[layer_idx][neuron_idx]);
            }
        }

        // Calculate gradients for each layer
        for layer_idx in 0..network.weights.len() {
            let current_layer_idx = layer_idx + 1; // weights[i] connects layer i to layer i+1
            let prev_activations = &activations[layer_idx];
            let current_errors = &layer_errors[current_layer_idx];

            for neuron_idx in 0..current_errors.len() {
                // Bias gradient
                bias_gradients[layer_idx][neuron_idx] = current_errors[neuron_idx];

                // Weight gradients
                let weight_start = neuron_idx * prev_activations.len();
                for (input_idx, &activation) in prev_activations.iter().enumerate() {
                    if weight_start + input_idx < weight_gradients[layer_idx].len() {
                        weight_gradients[layer_idx][weight_start + input_idx] =
                            current_errors[neuron_idx] * activation;
                    }
                }
            }
        }

        (weight_gradients, bias_gradients)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigmoid() {
        use helpers::sigmoid;

        assert!((sigmoid(0.0) - 0.5).abs() < 1e-6);
        assert!(sigmoid(10.0) > 0.99);
        assert!(sigmoid(-10.0) < 0.01);
    }
}

#[cfg(test)]
mod test_all_algorithms;
