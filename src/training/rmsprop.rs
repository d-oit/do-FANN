//! RMSProp optimizer for neural network training
//!
//! RMSProp (Root Mean Square Propagation) addresses the diminishing learning rates
//! problem of AdaGrad by using a moving average of squared gradients.
//!
//! Key features:
//! - Adaptive learning rates per parameter
//! - Moving average of squared gradients prevents accumulation
//! - Better handling of non-stationary objectives
//! - Improved convergence over vanilla SGD
//!
//! Expected performance gains:
//! - 2-3x faster convergence than SGD
//! - Better performance on noisy gradients
//! - More stable training than AdaGrad

#![allow(clippy::needless_range_loop)]

use super::*;
use num_traits::Float;
use std::collections::HashMap;

/// RMSProp optimizer implementation
pub struct RMSProp<T: Float + Send + Default> {
    learning_rate: T,
    decay_rate: T,
    epsilon: T,
    weight_decay: T,
    error_function: Box<dyn ErrorFunction<T>>,

    // Moving average of squared gradients
    v_weights: Vec<Vec<T>>, // Second moment (uncentered variance)
    v_biases: Vec<Vec<T>>,

    // Step counter for bias correction
    step: usize,

    callback: Option<TrainingCallback<T>>,
}

impl<T: Float + Send + Default> RMSProp<T> {
    /// Create a new RMSProp optimizer with default parameters
    pub fn new(learning_rate: T) -> Self {
        Self {
            learning_rate,
            decay_rate: T::from(0.9).unwrap(), // Common default for RMSProp
            epsilon: T::from(1e-8).unwrap(),
            weight_decay: T::zero(),
            error_function: Box::new(MseError),
            v_weights: Vec::new(),
            v_biases: Vec::new(),
            step: 0,
            callback: None,
        }
    }

    /// Set decay rate (rho parameter)
    pub fn with_decay_rate(mut self, decay_rate: T) -> Self {
        self.decay_rate = decay_rate;
        self
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

    /// Initialize moment estimates for the network
    fn initialize_moments(&mut self, network: &Network<T>) {
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

    /// Update parameters using RMSProp algorithm
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

                // Update moving average of squared gradients
                self.v_weights[layer_idx][i] = self.decay_rate * self.v_weights[layer_idx][i]
                    + (T::one() - self.decay_rate) * grad * grad;

                // Compute adaptive learning rate
                let adaptive_lr =
                    self.learning_rate / (self.v_weights[layer_idx][i].sqrt() + self.epsilon);

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

                // Update moving average of squared gradients
                self.v_biases[layer_idx][i] = self.decay_rate * self.v_biases[layer_idx][i]
                    + (T::one() - self.decay_rate) * grad * grad;

                // Compute adaptive learning rate
                let adaptive_lr =
                    self.learning_rate / (self.v_biases[layer_idx][i].sqrt() + self.epsilon);

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

impl<T: Float + Send + Default> TrainingAlgorithm<T> for RMSProp<T> {
    fn train_epoch(
        &mut self,
        network: &mut Network<T>,
        data: &TrainingData<T>,
    ) -> Result<T, TrainingError> {
        use super::helpers::*;

        self.initialize_moments(network);

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

        // Update parameters using RMSProp
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
        state.insert("decay_rate".to_string(), vec![self.decay_rate]);
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
        if let Some(dr) = state.algorithm_specific.get("decay_rate") {
            if !dr.is_empty() {
                self.decay_rate = dr[0];
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
        "RMSProp"
    }

    fn metrics(&self) -> HashMap<String, T> {
        let mut metrics = HashMap::new();
        metrics.insert("learning_rate".to_string(), self.learning_rate);
        metrics.insert("decay_rate".to_string(), self.decay_rate);
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
    fn test_rmsprop_creation() {
        let rmsprop = RMSProp::new(0.001f32);
        assert_eq!(rmsprop.learning_rate, 0.001);
        assert_eq!(rmsprop.decay_rate, 0.9);
        assert_eq!(rmsprop.step, 0);
    }

    #[test]
    fn test_rmsprop_with_parameters() {
        let rmsprop = RMSProp::new(0.001f32)
            .with_decay_rate(0.95)
            .with_epsilon(1e-7)
            .with_weight_decay(0.001);

        assert_eq!(rmsprop.decay_rate, 0.95);
        assert_eq!(rmsprop.epsilon, 1e-7);
        assert_eq!(rmsprop.weight_decay, 0.001);
    }
}
