//! Momentum SGD optimizer for neural network training
//!
//! Stochastic Gradient Descent with momentum accelerates convergence by
//! accumulating a velocity vector in the direction of persistent gradient descent.
//!
//! Key features:
//! - Momentum term accelerates convergence in relevant directions
//! - Dampens oscillations in high-curvature directions
//! - Nesterov accelerated gradient variant available
//! - Better performance than vanilla SGD
//!
//! Expected performance gains:
//! - 2-5x faster convergence than vanilla SGD
//! - Better handling of ravines and flat regions
//! - Reduced oscillations during training

#![allow(clippy::needless_range_loop)]

use super::*;
use num_traits::Float;
use std::collections::HashMap;

/// Momentum SGD optimizer implementation
pub struct MomentumSGD<T: Float + Send + Default> {
    learning_rate: T,
    momentum: T,
    weight_decay: T,
    nesterov: bool,
    error_function: Box<dyn ErrorFunction<T>>,

    // Velocity (momentum) terms
    v_weights: Vec<Vec<T>>, // Velocity for weights
    v_biases: Vec<Vec<T>>,  // Velocity for biases

    // Step counter
    step: usize,

    callback: Option<TrainingCallback<T>>,
}

impl<T: Float + Send + Default> MomentumSGD<T> {
    /// Create a new Momentum SGD optimizer with default parameters
    pub fn new(learning_rate: T) -> Self {
        Self {
            learning_rate,
            momentum: T::from(0.9).unwrap(), // Common default momentum
            weight_decay: T::zero(),
            nesterov: false,
            error_function: Box::new(MseError),
            v_weights: Vec::new(),
            v_biases: Vec::new(),
            step: 0,
            callback: None,
        }
    }

    /// Set momentum coefficient
    pub fn with_momentum(mut self, momentum: T) -> Self {
        self.momentum = momentum;
        self
    }

    /// Set weight decay (L2 regularization)
    pub fn with_weight_decay(mut self, weight_decay: T) -> Self {
        self.weight_decay = weight_decay;
        self
    }

    /// Enable Nesterov accelerated gradient
    pub fn with_nesterov(mut self, nesterov: bool) -> Self {
        self.nesterov = nesterov;
        self
    }

    /// Set error function
    pub fn with_error_function(mut self, error_function: Box<dyn ErrorFunction<T>>) -> Self {
        self.error_function = error_function;
        self
    }

    /// Initialize velocity terms for the network
    fn initialize_velocity(&mut self, network: &Network<T>) {
        if self.v_weights.is_empty() {
            self.v_weights = network
                .layers
                .iter()
                .skip(1) // Skip input layer
                .map(|layer| {
                    let num_neurons = layer.neurons.len();
                    let num_connections = if layer.neurons.is_empty() {
                        0
                    } else {
                        layer.neurons[0].connections.len()
                    };
                    vec![T::zero(); num_neurons * num_connections]
                })
                .collect();

            self.v_biases = network
                .layers
                .iter()
                .skip(1) // Skip input layer
                .map(|layer| vec![T::zero(); layer.neurons.len()])
                .collect();
        }
    }

    /// Update parameters using Momentum SGD algorithm
    fn update_parameters(
        &mut self,
        network: &mut Network<T>,
        weight_gradients: &[Vec<T>],
        bias_gradients: &[Vec<T>],
    ) {
        self.step += 1;

        // Update weight parameters
        let mut weight_updates = Vec::new();
        for layer_idx in 0..weight_gradients.len() {
            let mut layer_updates = Vec::new();
            for i in 0..weight_gradients[layer_idx].len() {
                let grad = weight_gradients[layer_idx][i];

                // Apply weight decay to gradient if specified
                let effective_grad = if self.weight_decay > T::zero() {
                    grad + self.weight_decay
                } else {
                    grad
                };

                // Update velocity (momentum)
                self.v_weights[layer_idx][i] = self.momentum * self.v_weights[layer_idx][i]
                    - self.learning_rate * effective_grad;

                // Compute parameter update
                let update = if self.nesterov {
                    // Nesterov accelerated gradient
                    let nesterov_grad =
                        effective_grad + self.momentum * self.v_weights[layer_idx][i];
                    -self.learning_rate * nesterov_grad
                } else {
                    self.v_weights[layer_idx][i]
                };

                layer_updates.push(update);
            }
            weight_updates.push(layer_updates);
        }

        // Update bias parameters
        let mut bias_updates = Vec::new();
        for layer_idx in 0..bias_gradients.len() {
            let mut layer_updates = Vec::new();
            for i in 0..bias_gradients[layer_idx].len() {
                let grad = bias_gradients[layer_idx][i];

                // Update velocity (momentum)
                self.v_biases[layer_idx][i] =
                    self.momentum * self.v_biases[layer_idx][i] - self.learning_rate * grad;

                // Compute parameter update
                let update = if self.nesterov {
                    // Nesterov accelerated gradient
                    let nesterov_grad = grad + self.momentum * self.v_biases[layer_idx][i];
                    -self.learning_rate * nesterov_grad
                } else {
                    self.v_biases[layer_idx][i]
                };

                layer_updates.push(update);
            }
            bias_updates.push(layer_updates);
        }

        // Apply updates using existing helper
        super::helpers::apply_updates_to_network(network, &weight_updates, &bias_updates);
    }
}

impl<T: Float + Send + Default> TrainingAlgorithm<T> for MomentumSGD<T> {
    fn train_epoch(
        &mut self,
        network: &mut Network<T>,
        data: &TrainingData<T>,
    ) -> Result<T, TrainingError> {
        use super::helpers::*;

        self.initialize_velocity(network);

        let mut total_error = T::zero();

        // Convert network to simplified form for easier manipulation
        let simple_network = network_to_simple(network);

        // Accumulate gradients over entire batch
        let mut accumulated_weight_gradients = simple_network
            .weights
            .iter()
            .map(|w| vec![T::zero(); w.len()])
            .collect::<Vec<_>>();
        let mut accumulated_bias_gradients = simple_network
            .biases
            .iter()
            .map(|b| vec![T::zero(); b.len()])
            .collect::<Vec<_>>();

        // Process all samples in the batch
        for (input, desired_output) in data.inputs.iter().zip(data.outputs.iter()) {
            // Forward propagation to get all layer activations
            let activations = forward_propagate(&simple_network, input);

            // Get output from last layer
            let output = &activations[activations.len() - 1];

            // Calculate error
            total_error = total_error + self.error_function.calculate(output, desired_output);

            // Calculate gradients using backpropagation
            let (weight_gradients, bias_gradients) = calculate_gradients(
                &simple_network,
                &activations,
                desired_output,
                self.error_function.as_ref(),
            );

            // Accumulate gradients
            for layer_idx in 0..weight_gradients.len() {
                for i in 0..weight_gradients[layer_idx].len() {
                    accumulated_weight_gradients[layer_idx][i] =
                        accumulated_weight_gradients[layer_idx][i] + weight_gradients[layer_idx][i];
                }
                for i in 0..bias_gradients[layer_idx].len() {
                    accumulated_bias_gradients[layer_idx][i] =
                        accumulated_bias_gradients[layer_idx][i] + bias_gradients[layer_idx][i];
                }
            }
        }

        // Average gradients over batch size
        let batch_size = T::from(data.inputs.len()).unwrap();
        for layer_idx in 0..accumulated_weight_gradients.len() {
            for i in 0..accumulated_weight_gradients[layer_idx].len() {
                accumulated_weight_gradients[layer_idx][i] =
                    accumulated_weight_gradients[layer_idx][i] / batch_size;
            }
            for i in 0..accumulated_bias_gradients[layer_idx].len() {
                accumulated_bias_gradients[layer_idx][i] =
                    accumulated_bias_gradients[layer_idx][i] / batch_size;
            }
        }

        // Update parameters using Momentum SGD
        self.update_parameters(
            network,
            &accumulated_weight_gradients,
            &accumulated_bias_gradients,
        );

        Ok(total_error / batch_size)
    }

    fn calculate_error(&self, network: &Network<T>, data: &TrainingData<T>) -> T {
        let mut total_error = T::zero();
        let mut network_clone = network.clone();

        for (input, desired_output) in data.inputs.iter().zip(data.outputs.iter()) {
            let output = network_clone.run(input);
            total_error = total_error + self.error_function.calculate(&output, desired_output);
        }

        total_error / T::from(data.inputs.len()).unwrap()
    }

    fn count_bit_fails(
        &self,
        network: &Network<T>,
        data: &TrainingData<T>,
        bit_fail_limit: T,
    ) -> usize {
        let mut bit_fails = 0;
        let mut network_clone = network.clone();

        for (input, desired_output) in data.inputs.iter().zip(data.outputs.iter()) {
            let output = network_clone.run(input);
            for (&actual, &desired) in output.iter().zip(desired_output.iter()) {
                if (actual - desired).abs() > bit_fail_limit {
                    bit_fails += 1;
                }
            }
        }

        bit_fails
    }

    fn save_state(&self) -> TrainingState<T> {
        let mut state = HashMap::new();
        state.insert("learning_rate".to_string(), vec![self.learning_rate]);
        state.insert("momentum".to_string(), vec![self.momentum]);
        state.insert("weight_decay".to_string(), vec![self.weight_decay]);
        state.insert(
            "nesterov".to_string(),
            vec![if self.nesterov { T::one() } else { T::zero() }],
        );
        state.insert("step".to_string(), vec![T::from(self.step).unwrap()]);

        TrainingState {
            epoch: 0,
            best_error: T::from(f32::MAX).unwrap(),
            algorithm_specific: state,
        }
    }

    fn restore_state(&mut self, state: TrainingState<T>) {
        if let Some(lr) = state.algorithm_specific.get("learning_rate") {
            if !lr.is_empty() {
                self.learning_rate = lr[0];
            }
        }
        if let Some(m) = state.algorithm_specific.get("momentum") {
            if !m.is_empty() {
                self.momentum = m[0];
            }
        }
        if let Some(wd) = state.algorithm_specific.get("weight_decay") {
            if !wd.is_empty() {
                self.weight_decay = wd[0];
            }
        }
        if let Some(n) = state.algorithm_specific.get("nesterov") {
            if !n.is_empty() {
                self.nesterov = n[0] > T::zero();
            }
        }
        if let Some(s) = state.algorithm_specific.get("step") {
            if !s.is_empty() {
                self.step = s[0].to_usize().unwrap_or(0);
            }
        }
    }

    fn set_callback(&mut self, callback: TrainingCallback<T>) {
        self.callback = Some(callback);
    }

    fn call_callback(
        &mut self,
        epoch: usize,
        network: &Network<T>,
        data: &TrainingData<T>,
    ) -> bool {
        let error = self.calculate_error(network, data);
        if let Some(ref mut callback) = self.callback {
            callback(epoch, error)
        } else {
            true
        }
    }

    fn name(&self) -> &str {
        if self.nesterov {
            "MomentumSGD-Nesterov"
        } else {
            "MomentumSGD"
        }
    }

    fn metrics(&self) -> HashMap<String, T> {
        let mut metrics = HashMap::new();
        metrics.insert("learning_rate".to_string(), self.learning_rate);
        metrics.insert("momentum".to_string(), self.momentum);
        metrics.insert("weight_decay".to_string(), self.weight_decay);
        metrics.insert(
            "nesterov".to_string(),
            if self.nesterov { T::one() } else { T::zero() },
        );
        metrics.insert("step".to_string(), T::from(self.step).unwrap());
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Network;

    #[test]
    fn test_momentum_sgd_creation() {
        let momentum_sgd = MomentumSGD::new(0.01f32);
        assert_eq!(momentum_sgd.learning_rate, 0.01);
        assert_eq!(momentum_sgd.momentum, 0.9);
        assert_eq!(momentum_sgd.nesterov, false);
        assert_eq!(momentum_sgd.step, 0);
    }

    #[test]
    fn test_momentum_sgd_with_parameters() {
        let momentum_sgd = MomentumSGD::new(0.01f32)
            .with_momentum(0.95)
            .with_weight_decay(0.001)
            .with_nesterov(true);

        assert_eq!(momentum_sgd.momentum, 0.95);
        assert_eq!(momentum_sgd.weight_decay, 0.001);
        assert_eq!(momentum_sgd.nesterov, true);
    }
}
