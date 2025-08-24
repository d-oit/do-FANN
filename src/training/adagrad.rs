//! AdaGrad optimizer for neural network training
//!
//! AdaGrad (Adaptive Gradient Algorithm) adapts the learning rate for each parameter
//! based on the historical gradient information, making it suitable for sparse data.
//!
//! Key features:
//! - Per-parameter adaptive learning rates
//! - Accumulates squared gradients over time
//! - Excellent for sparse and non-stationary objectives
//! - No manual tuning of learning rate required
//!
//! Expected performance gains:
//! - Better performance on sparse datasets
//! - Automatic learning rate adaptation
//! - Improved convergence on certain problem types

#![allow(clippy::needless_range_loop)]

use super::*;
use num_traits::Float;
use std::collections::HashMap;

/// AdaGrad optimizer implementation
pub struct AdaGrad<T: Float + Send + Default> {
    learning_rate: T,
    epsilon: T,
    weight_decay: T,
    error_function: Box<dyn ErrorFunction<T>>,

    // Accumulated squared gradients
    g_weights: Vec<Vec<T>>, // Sum of squared gradients for weights
    g_biases: Vec<Vec<T>>,  // Sum of squared gradients for biases

    // Step counter
    step: usize,

    callback: Option<TrainingCallback<T>>,
}

impl<T: Float + Send + Default> AdaGrad<T> {
    /// Create a new AdaGrad optimizer with default parameters
    pub fn new(learning_rate: T) -> Self {
        Self {
            learning_rate,
            epsilon: T::from(1e-8).unwrap(),
            weight_decay: T::zero(),
            error_function: Box::new(MseError),
            g_weights: Vec::new(),
            g_biases: Vec::new(),
            step: 0,
            callback: None,
        }
    }

    /// Set epsilon for numerical stability
    pub fn with_epsilon(mut self, epsilon: T) -> Self {
        self.epsilon = epsilon;
        self
    }

    /// Set weight decay (L2 regularization)
    pub fn with_weight_decay(mut self, weight_decay: T) -> Self {
        self.weight_decay = weight_decay;
        self
    }

    /// Set error function
    pub fn with_error_function(mut self, error_function: Box<dyn ErrorFunction<T>>) -> Self {
        self.error_function = error_function;
        self
    }

    /// Initialize gradient accumulators for the network
    fn initialize_accumulators(&mut self, network: &Network<T>) {
        if self.g_weights.is_empty() {
            self.g_weights = network
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

            self.g_biases = network
                .layers
                .iter()
                .skip(1) // Skip input layer
                .map(|layer| vec![T::zero(); layer.neurons.len()])
                .collect();
        }
    }

    /// Update parameters using AdaGrad algorithm
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

                // Accumulate squared gradients
                self.g_weights[layer_idx][i] = self.g_weights[layer_idx][i] + grad * grad;

                // Compute adaptive learning rate
                let adaptive_lr =
                    self.learning_rate / (self.g_weights[layer_idx][i].sqrt() + self.epsilon);

                // Compute parameter update
                let update = -adaptive_lr * grad;
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

                // Accumulate squared gradients
                self.g_biases[layer_idx][i] = self.g_biases[layer_idx][i] + grad * grad;

                // Compute adaptive learning rate
                let adaptive_lr =
                    self.learning_rate / (self.g_biases[layer_idx][i].sqrt() + self.epsilon);

                // Compute parameter update
                let update = -adaptive_lr * grad;
                layer_updates.push(update);
            }
            bias_updates.push(layer_updates);
        }

        // Apply weight decay if specified
        if self.weight_decay > T::zero() {
            for layer_updates in &mut weight_updates {
                for update in layer_updates {
                    *update = *update - self.learning_rate * self.weight_decay;
                }
            }
        }

        // Apply updates using existing helper
        super::helpers::apply_updates_to_network(network, &weight_updates, &bias_updates);
    }
}

impl<T: Float + Send + Default> TrainingAlgorithm<T> for AdaGrad<T> {
    fn train_epoch(
        &mut self,
        network: &mut Network<T>,
        data: &TrainingData<T>,
    ) -> Result<T, TrainingError> {
        use super::helpers::*;

        self.initialize_accumulators(network);

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

        // Update parameters using AdaGrad
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
        state.insert("epsilon".to_string(), vec![self.epsilon]);
        state.insert("weight_decay".to_string(), vec![self.weight_decay]);
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
        if let Some(eps) = state.algorithm_specific.get("epsilon") {
            if !eps.is_empty() {
                self.epsilon = eps[0];
            }
        }
        if let Some(wd) = state.algorithm_specific.get("weight_decay") {
            if !wd.is_empty() {
                self.weight_decay = wd[0];
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
        "AdaGrad"
    }

    fn metrics(&self) -> HashMap<String, T> {
        let mut metrics = HashMap::new();
        metrics.insert("learning_rate".to_string(), self.learning_rate);
        metrics.insert("epsilon".to_string(), self.epsilon);
        metrics.insert("weight_decay".to_string(), self.weight_decay);
        metrics.insert("step".to_string(), T::from(self.step).unwrap());
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Network;

    #[test]
    fn test_adagrad_creation() {
        let adagrad = AdaGrad::new(0.01f32);
        assert_eq!(adagrad.learning_rate, 0.01);
        assert_eq!(adagrad.step, 0);
    }

    #[test]
    fn test_adagrad_with_parameters() {
        let adagrad = AdaGrad::new(0.01f32)
            .with_epsilon(1e-7)
            .with_weight_decay(0.001);

        assert_eq!(adagrad.epsilon, 1e-7);
        assert_eq!(adagrad.weight_decay, 0.001);
    }
}
