//! Comprehensive error handling system for ruv-FANN
//!
//! This module provides a unified error handling framework with detailed error categories,
//! context information, and recovery mechanisms for robust neural network operations.

use crate::{NetworkError, TrainingError};
use std::error::Error;
use thiserror::Error;

/// Main error type for all ruv-FANN operations
#[derive(Error, Debug)]
pub enum RuvFannError {
    /// Network configuration and topology errors
    #[error("Network error: {category:?} - {message}")]
    Network {
        category: NetworkErrorCategory,
        message: String,
        context: Option<String>,
    },

    /// Training and learning algorithm errors
    #[error("Training error: {category:?} - {message}")]
    Training {
        category: TrainingErrorCategory,
        message: String,
        context: Option<String>,
    },

    /// Cascade correlation specific errors
    #[error("Cascade error: {category:?} - {message}")]
    Cascade {
        category: CascadeErrorCategory,
        message: String,
        context: Option<String>,
    },

    /// Data validation and format errors
    #[error("Validation error: {category:?} - {message}")]
    Validation {
        category: ValidationErrorCategory,
        message: String,
        details: Vec<String>,
    },

    /// I/O and serialization errors
    #[error("I/O error: {category:?} - {message}")]
    Io {
        category: IoErrorCategory,
        message: String,
        source: Option<Box<dyn Error + Send + Sync>>,
    },

    /// Parallel processing and concurrency errors
    #[error("Parallel processing error: {message}")]
    Parallel {
        message: String,
        thread_count: usize,
        context: Option<String>,
    },

    /// Memory allocation and management errors
    #[error("Memory error: {message}")]
    Memory {
        message: String,
        requested_bytes: Option<usize>,
        available_bytes: Option<usize>,
    },

    /// Performance and optimization errors
    #[error("Performance error: {message}")]
    Performance {
        message: String,
        metric: String,
        threshold: f64,
        actual: f64,
    },

    /// FANN compatibility errors
    #[error("FANN compatibility error: {message}")]
    Compatibility {
        message: String,
        fann_version: Option<String>,
        operation: String,
    },

    /// WASM-specific errors
    #[error("WASM error: {message}")]
    Wasm {
        message: String,
        operation: String,
        context: Option<WasmErrorContext>,
    },

    /// Training recovery errors
    #[error("Training recovery error: {message}")]
    TrainingRecovery {
        message: String,
        recovery_attempt: usize,
        original_error: String,
        context: Option<TrainingRecoveryContext>,
    },

    /// Performance degradation errors
    #[error("Performance degradation error: {message}")]
    PerformanceDegradation {
        message: String,
        metric: String,
        expected: f64,
        actual: f64,
        degradation_threshold: f64,
    },
}

/// Network error categories for detailed classification
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkErrorCategory {
    /// Invalid network topology or structure
    Topology,
    /// Weight and bias configuration issues
    Weights,
    /// Layer configuration problems
    Layers,
    /// Neuron connection issues
    Connections,
    /// Activation function problems
    Activation,
    /// Forward propagation errors
    Propagation,
}

/// Training error categories
#[derive(Debug, Clone, PartialEq)]
pub enum TrainingErrorCategory {
    /// Learning algorithm failures
    Algorithm,
    /// Convergence problems
    Convergence,
    /// Gradient calculation issues
    Gradients,
    /// Learning rate problems
    LearningRate,
    /// Epoch and iteration errors
    Iteration,
    /// Stop criteria issues
    StopCriteria,
}

/// Cascade correlation error categories
#[derive(Debug, Clone, PartialEq)]
pub enum CascadeErrorCategory {
    /// Candidate neuron generation issues
    CandidateGeneration,
    /// Candidate training failures
    CandidateTraining,
    /// Candidate selection problems
    CandidateSelection,
    /// Network topology modification errors
    TopologyModification,
    /// Correlation calculation issues
    CorrelationCalculation,
    /// Output training problems
    OutputTraining,
}

/// Validation error categories
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorCategory {
    /// Input data validation
    InputData,
    /// Output data validation
    OutputData,
    /// Network configuration validation
    NetworkConfig,
    /// Training parameter validation
    TrainingParams,
    /// Cascade parameter validation
    CascadeParams,
}

/// I/O error categories
#[derive(Debug, Clone, PartialEq)]
pub enum IoErrorCategory {
    /// File reading/writing issues
    FileAccess,
    /// Serialization/deserialization problems
    Serialization,
    /// Format compatibility issues
    Format,
    /// Network export/import errors
    NetworkIo,
    /// Training data I/O problems
    DataIo,
}

/// Comprehensive error category enum for uniform handling
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    Network(NetworkErrorCategory),
    Training(TrainingErrorCategory),
    Cascade(CascadeErrorCategory),
    Validation(ValidationErrorCategory),
    Io(IoErrorCategory),
    Parallel,
    Memory,
    Performance,
    Compatibility,
}

/// Validation error for detailed parameter checking
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Parameter out of range: {parameter} = {value}, expected {min} <= value <= {max}")]
    OutOfRange {
        parameter: String,
        value: f64,
        min: f64,
        max: f64,
    },

    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    #[error("Missing required parameter: {parameter}")]
    MissingParameter { parameter: String },

    #[error("Incompatible parameters: {message}")]
    IncompatibleParams { message: String },

    #[error("Data format error: {message}")]
    DataFormat { message: String },
}

/// Error context for providing additional debugging information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub network_id: Option<String>,
    pub layer_index: Option<usize>,
    pub neuron_index: Option<usize>,
    pub epoch: Option<usize>,
    pub timestamp: std::time::SystemTime,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            network_id: None,
            layer_index: None,
            neuron_index: None,
            epoch: None,
            timestamp: std::time::SystemTime::now(),
            additional_info: std::collections::HashMap::new(),
        }
    }

    pub fn with_network_id(mut self, id: impl Into<String>) -> Self {
        self.network_id = Some(id.into());
        self
    }

    pub fn with_layer(mut self, index: usize) -> Self {
        self.layer_index = Some(index);
        self
    }

    pub fn with_neuron(mut self, index: usize) -> Self {
        self.neuron_index = Some(index);
        self
    }

    pub fn with_epoch(mut self, epoch: usize) -> Self {
        self.epoch = Some(epoch);
        self
    }

    pub fn with_info(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_info.insert(key.into(), value.into());
        self
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation with the same parameters
    Retry,
    /// Retry with modified parameters
    RetryWithModification(std::collections::HashMap<String, String>),
    /// Reset to a known good state
    Reset,
    /// Skip the problematic operation
    Skip,
    /// Abort the entire process
    Abort,
    /// Use fallback implementation
    Fallback(String),
    /// Checkpoint and continue (for training)
    CheckpointAndContinue,
    /// Reduce learning rate and retry
    ReduceLearningRate { factor: f64 },
    /// Increase batch size and retry
    IncreaseBatchSize { factor: usize },
    /// Use gradient clipping and retry
    GradientClipping { threshold: f64 },
    /// Memory optimization and retry
    MemoryOptimization,
}

/// Error recovery context
#[derive(Debug)]
pub struct RecoveryContext {
    pub strategy: RecoveryStrategy,
    pub max_retries: usize,
    pub current_retry: usize,
    pub fallback_available: bool,
    pub checkpoints: Vec<String>,
}

/// Training error recovery context
#[derive(Debug)]
pub struct TrainingRecoveryContext {
    pub base_context: RecoveryContext,
    pub learning_rate_backup: Option<f64>,
    pub gradient_clip_threshold: Option<f64>,
    pub batch_size_backup: Option<usize>,
    pub checkpoint_path: Option<String>,
    pub last_successful_epoch: Option<usize>,
    pub memory_usage_at_failure: Option<usize>,
    pub error_pattern: Vec<String>,
}

impl TrainingRecoveryContext {
    pub fn new() -> Self {
        Self {
            base_context: RecoveryContext::new(RecoveryStrategy::Retry),
            learning_rate_backup: None,
            gradient_clip_threshold: None,
            batch_size_backup: None,
            checkpoint_path: None,
            last_successful_epoch: None,
            memory_usage_at_failure: None,
            error_pattern: Vec::new(),
        }
    }

    pub fn with_checkpoint(mut self, path: String) -> Self {
        self.checkpoint_path = Some(path);
        self
    }

    pub fn with_learning_rate_backup(mut self, lr: f64) -> Self {
        self.learning_rate_backup = Some(lr);
        self
    }

    pub fn record_error(&mut self, error: &str) {
        self.error_pattern.push(error.to_string());
        if self.error_pattern.len() > 10 {
            self.error_pattern.remove(0); // Keep only last 10 errors
        }
    }

    pub fn should_attempt_recovery(&self) -> bool {
        self.base_context.should_retry()
    }

    pub fn get_recovery_suggestion(&self) -> RecoveryStrategy {
        // Analyze error pattern to suggest recovery strategy
        let error_count = self.error_pattern.len();

        if error_count >= 3 {
            let memory_errors = self
                .error_pattern
                .iter()
                .filter(|e| e.contains("memory") || e.contains("allocation"))
                .count();

            let gradient_errors = self
                .error_pattern
                .iter()
                .filter(|e| e.contains("gradient") || e.contains("NaN") || e.contains("Inf"))
                .count();

            let convergence_errors = self
                .error_pattern
                .iter()
                .filter(|e| e.contains("convergence") || e.contains("diverge"))
                .count();

            if memory_errors > gradient_errors && memory_errors > convergence_errors {
                RecoveryStrategy::MemoryOptimization
            } else if gradient_errors > convergence_errors {
                RecoveryStrategy::GradientClipping { threshold: 1.0 }
            } else {
                RecoveryStrategy::ReduceLearningRate { factor: 0.5 }
            }
        } else {
            RecoveryStrategy::Retry
        }
    }
}

/// WASM-specific error context
#[derive(Debug)]
pub struct WasmErrorContext {
    pub operation: String,
    pub memory_available: Option<usize>,
    pub memory_used: Option<usize>,
    pub wasm_memory_pages: Option<u32>,
    pub browser_info: Option<String>,
    pub webgl_support: Option<bool>,
    pub webgpu_support: Option<bool>,
    pub fallback_implementation: Option<String>,
}

impl WasmErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            memory_available: None,
            memory_used: None,
            wasm_memory_pages: None,
            browser_info: None,
            webgl_support: None,
            webgpu_support: None,
            fallback_implementation: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn detect_wasm_environment(&mut self) {
        // WASM-specific environment detection
        self.memory_available = web_sys::window()
            .and_then(|w| w.navigator().device_memory())
            .map(|m| (m as usize) * 1024 * 1024 * 1024); // Convert GB to bytes

        // Try to detect WebGL support
        self.webgl_support = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.create_element("canvas").ok())
            .and_then(|c| c.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .and_then(|c| c.get_context("webgl").ok().flatten())
            .map(|_| true);

        // Try to detect WebGPU support
        self.webgpu_support = web_sys::window()
            .and_then(|w| w.navigator().gpu())
            .map(|_| true);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn detect_wasm_environment(&mut self) {
        // No-op for non-WASM targets
    }
}

impl RecoveryContext {
    pub fn new(strategy: RecoveryStrategy) -> Self {
        Self {
            strategy,
            max_retries: 3,
            current_retry: 0,
            fallback_available: false,
            checkpoints: Vec::new(),
        }
    }

    pub fn should_retry(&self) -> bool {
        self.current_retry < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.current_retry += 1;
    }

    pub fn reset_retry_count(&mut self) {
        self.current_retry = 0;
    }
}

/// Professional error logging and debugging facilities
pub struct ErrorLogger {
    #[cfg(feature = "logging")]
    log_level: log::Level,
    #[cfg(not(feature = "logging"))]
    log_level: u8, // Simple placeholder when log feature is disabled
    structured_logging: bool,
    performance_tracking: bool,
}

impl ErrorLogger {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "logging")]
            log_level: log::Level::Warn,
            #[cfg(not(feature = "logging"))]
            log_level: 2, // 2 as a placeholder for Warn level
            structured_logging: true,
            performance_tracking: false,
        }
    }

    #[cfg(feature = "logging")]
    pub fn with_level(mut self, level: log::Level) -> Self {
        self.log_level = level;
        self
    }

    #[cfg(not(feature = "logging"))]
    pub fn with_level(self, _level: u8) -> Self {
        // No-op when logging is disabled
        self
    }

    pub fn with_structured_logging(mut self, enabled: bool) -> Self {
        self.structured_logging = enabled;
        self
    }

    pub fn with_performance_tracking(mut self, enabled: bool) -> Self {
        self.performance_tracking = enabled;
        self
    }

    pub fn log_error(&self, error: &RuvFannError, context: Option<&ErrorContext>) {
        if self.structured_logging {
            self.log_structured_error(error, context);
        } else {
            self.log_simple_error(error, context);
        }
    }

    fn log_structured_error(&self, error: &RuvFannError, context: Option<&ErrorContext>) {
        #[cfg(feature = "serde")]
        {
            let mut fields = serde_json::Map::new();
            fields.insert(
                "error_type".to_string(),
                serde_json::Value::String(format!("{error:?}")),
            );
            fields.insert(
                "message".to_string(),
                serde_json::Value::String(error.to_string()),
            );

            if let Some(ctx) = context {
                fields.insert(
                    "operation".to_string(),
                    serde_json::Value::String(ctx.operation.clone()),
                );
                if let Some(ref network_id) = ctx.network_id {
                    fields.insert(
                        "network_id".to_string(),
                        serde_json::Value::String(network_id.clone()),
                    );
                }
                if let Some(layer_idx) = ctx.layer_index {
                    fields.insert(
                        "layer_index".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(layer_idx)),
                    );
                }
                if let Some(neuron_idx) = ctx.neuron_index {
                    fields.insert(
                        "neuron_index".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(neuron_idx)),
                    );
                }
                if let Some(epoch) = ctx.epoch {
                    fields.insert(
                        "epoch".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(epoch)),
                    );
                }
            }

            #[cfg(feature = "logging")]
            {
                log::log!(self.log_level, "{}", serde_json::Value::Object(fields));
            }
        }

        #[cfg(not(feature = "serde"))]
        {
            // Simple fallback when serde_json is not available
            let _ = error;
            let _ = context;
        }

        #[cfg(all(feature = "logging", not(feature = "serde")))]
        log::log!(self.log_level, "Error: {}", error);
    }

    fn log_simple_error(&self, error: &RuvFannError, context: Option<&ErrorContext>) {
        let context_str = context
            .map(|c| format!(" [{}]", c.operation))
            .unwrap_or_default();

        #[cfg(feature = "logging")]
        log::log!(self.log_level, "Error{context_str}: {error}");
    }
}

impl Default for ErrorLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert from legacy error types for backward compatibility
impl From<NetworkError> for RuvFannError {
    fn from(error: NetworkError) -> Self {
        match error {
            NetworkError::InputSizeMismatch { expected, actual } => RuvFannError::Network {
                category: NetworkErrorCategory::Topology,
                message: format!("Input size mismatch: expected {expected}, got {actual}"),
                context: None,
            },
            NetworkError::WeightCountMismatch { expected, actual } => RuvFannError::Network {
                category: NetworkErrorCategory::Weights,
                message: format!("Weight count mismatch: expected {expected}, got {actual}"),
                context: None,
            },
            NetworkError::InvalidLayerConfiguration => RuvFannError::Network {
                category: NetworkErrorCategory::Layers,
                message: "Invalid layer configuration".to_string(),
                context: None,
            },
            NetworkError::NoLayers => RuvFannError::Network {
                category: NetworkErrorCategory::Topology,
                message: "Network has no layers".to_string(),
                context: None,
            },
        }
    }
}

impl From<TrainingError> for RuvFannError {
    fn from(error: TrainingError) -> Self {
        match error {
            TrainingError::InvalidData(msg) => RuvFannError::Validation {
                category: ValidationErrorCategory::InputData,
                message: msg,
                details: vec![],
            },
            TrainingError::NetworkError(msg) => RuvFannError::Network {
                category: NetworkErrorCategory::Topology,
                message: msg,
                context: None,
            },
            TrainingError::TrainingFailed(msg) => RuvFannError::Training {
                category: TrainingErrorCategory::Algorithm,
                message: msg,
                context: None,
            },
        }
    }
}

/// Helper macros for error creation with context
#[macro_export]
macro_rules! network_error {
    ($category:expr, $msg:expr) => {
        RuvFannError::Network {
            category: $category,
            message: $msg.to_string(),
            context: None,
        }
    };
    ($category:expr, $msg:expr, $context:expr) => {
        RuvFannError::Network {
            category: $category,
            message: $msg.to_string(),
            context: Some($context.to_string()),
        }
    };
}

/// WASM error handling macro
#[macro_export]
macro_rules! wasm_error {
    ($operation:expr, $msg:expr) => {
        RuvFannError::Wasm {
            message: $msg.to_string(),
            operation: $operation.to_string(),
            context: None,
        }
    };
    ($operation:expr, $msg:expr, $context:expr) => {
        RuvFannError::Wasm {
            message: $msg.to_string(),
            operation: $operation.to_string(),
            context: Some($context),
        }
    };
}

/// Training recovery error macro
#[macro_export]
macro_rules! training_recovery_error {
    ($msg:expr, $attempt:expr, $original:expr) => {
        RuvFannError::TrainingRecovery {
            message: $msg.to_string(),
            recovery_attempt: $attempt,
            original_error: $original.to_string(),
            context: None,
        }
    };
    ($msg:expr, $attempt:expr, $original:expr, $context:expr) => {
        RuvFannError::TrainingRecovery {
            message: $msg.to_string(),
            recovery_attempt: $attempt,
            original_error: $original.to_string(),
            context: Some($context),
        }
    };
}

/// Performance error macro
#[macro_export]
macro_rules! performance_error {
    ($msg:expr, $metric:expr, $expected:expr, $actual:expr, $threshold:expr) => {
        RuvFannError::Performance {
            message: $msg.to_string(),
            metric: $metric.to_string(),
            expected: $expected,
            actual: $actual,
            degradation_threshold: $threshold,
        }
    };
}

/// Error handling with recovery macro
#[macro_export]
macro_rules! handle_error_with_recovery {
    ($handler:expr, $error:expr, $recovery_fn:expr) => {
        $handler.handle_error($error, $recovery_fn)
    };
}

/// WASM-safe error handling macro
#[macro_export]
macro_rules! wasm_safe {
    ($operation:expr, $code:expr) => {
        match $code {
            Ok(result) => Ok(result),
            Err(error) => {
                let wasm_error = RuvFannError::Wasm {
                    message: format!("Operation '{}' failed: {}", $operation, error),
                    operation: $operation.to_string(),
                    context: None,
                };
                Err(wasm_error)
            }
        }
    };
}

#[macro_export]
macro_rules! training_error {
    ($category:expr, $msg:expr) => {
        RuvFannError::Training {
            category: $category,
            message: $msg.to_string(),
            context: None,
        }
    };
    ($category:expr, $msg:expr, $context:expr) => {
        RuvFannError::Training {
            category: $category,
            message: $msg.to_string(),
            context: Some($context.to_string()),
        }
    };
}

#[macro_export]
macro_rules! cascade_error {
    ($category:expr, $msg:expr) => {
        RuvFannError::Cascade {
            category: $category,
            message: $msg.to_string(),
            context: None,
        }
    };
    ($category:expr, $msg:expr, $context:expr) => {
        RuvFannError::Cascade {
            category: $category,
            message: $msg.to_string(),
            context: Some($context.to_string()),
        }
    };
}

/// Comprehensive result type for all ruv-FANN operations
pub type RuvFannResult<T> = Result<T, RuvFannError>;

/// Error handling utilities
pub struct ErrorHandler {
    logger: ErrorLogger,
    recovery_context: Option<RecoveryContext>,
    training_recovery: Option<TrainingRecoveryContext>,
    wasm_context: Option<WasmErrorContext>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            logger: ErrorLogger::new(),
            recovery_context: None,
            training_recovery: None,
            wasm_context: None,
        }
    }

    pub fn with_logger(mut self, logger: ErrorLogger) -> Self {
        self.logger = logger;
        self
    }

    pub fn with_recovery_context(mut self, context: RecoveryContext) -> Self {
        self.recovery_context = Some(context);
        self
    }

    pub fn with_training_recovery(mut self, context: TrainingRecoveryContext) -> Self {
        self.training_recovery = Some(context);
        self
    }

    pub fn with_wasm_context(mut self, context: WasmErrorContext) -> Self {
        self.wasm_context = Some(context);
        self
    }

    /// Handle an error with automatic recovery
    pub fn handle_error<F, T>(&mut self, error: RuvFannError, recovery_fn: F) -> RuvFannResult<T>
    where
        F: Fn(&RecoveryStrategy) -> RuvFannResult<T>,
    {
        // Log the error
        self.logger.log_error(&error, None);

        // Try recovery if available
        if let Some(ref mut context) = self.recovery_context {
            if context.should_retry() {
                let strategy = if let Some(ref mut training_ctx) = self.training_recovery {
                    training_ctx.record_error(&error.to_string());
                    training_ctx.get_recovery_suggestion()
                } else {
                    context.strategy.clone()
                };

                match recovery_fn(&strategy) {
                    Ok(result) => {
                        context.reset_retry_count();
                        return Ok(result);
                    }
                    Err(recovery_error) => {
                        context.increment_retry();
                        self.logger.log_error(&recovery_error, None);

                        if !context.should_retry() {
                            return Err(RuvFannError::TrainingRecovery {
                                message: "All recovery attempts failed".to_string(),
                                recovery_attempt: context.current_retry,
                                original_error: error.to_string(),
                                context: self.training_recovery.take(),
                            });
                        }
                    }
                }
            }
        }

        Err(error)
    }

    /// Handle WASM-specific errors
    pub fn handle_wasm_error(&mut self, error: RuvFannError) -> RuvFannError {
        if let Some(ref mut wasm_ctx) = self.wasm_context {
            wasm_ctx.detect_wasm_environment();

            match error {
                RuvFannError::Memory { .. } => RuvFannError::Wasm {
                    message: "Memory allocation failed in WASM environment".to_string(),
                    operation: wasm_ctx.operation.clone(),
                    context: Some(wasm_ctx.clone()),
                },
                RuvFannError::Training { .. } => RuvFannError::Wasm {
                    message: "Training operation failed in WASM environment".to_string(),
                    operation: wasm_ctx.operation.clone(),
                    context: Some(wasm_ctx.clone()),
                },
                _ => error,
            }
        } else {
            error
        }
    }

    /// Create a checkpoint for training recovery
    pub fn create_checkpoint(&self, path: &str) -> RuvFannResult<()> {
        // In a real implementation, this would save model state
        log::info!("Creating checkpoint at: {}", path);
        Ok(())
    }

    /// Restore from checkpoint
    pub fn restore_checkpoint(&self, path: &str) -> RuvFannResult<()> {
        // In a real implementation, this would load model state
        log::info!("Restoring checkpoint from: {}", path);
        Ok(())
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = RuvFannError::Network {
            category: NetworkErrorCategory::Topology,
            message: "Test error".to_string(),
            context: None,
        };

        assert!(matches!(error, RuvFannError::Network { .. }));
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_operation")
            .with_network_id("network_1")
            .with_layer(2)
            .with_epoch(100);

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.network_id, Some("network_1".to_string()));
        assert_eq!(context.layer_index, Some(2));
        assert_eq!(context.epoch, Some(100));
    }

    #[test]
    fn test_recovery_context() {
        let mut recovery = RecoveryContext::new(RecoveryStrategy::Retry);
        assert!(recovery.should_retry());

        recovery.max_retries = 2;
        recovery.current_retry = 2;
        assert!(!recovery.should_retry());
    }

    #[test]
    fn test_error_conversion() {
        let network_error = NetworkError::NoLayers;
        let ruv_error: RuvFannError = network_error.into();

        match ruv_error {
            RuvFannError::Network { category, .. } => {
                assert_eq!(category, NetworkErrorCategory::Topology);
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_training_recovery_context() {
        let mut context = TrainingRecoveryContext::new()
            .with_checkpoint("test_checkpoint.bin".to_string())
            .with_learning_rate_backup(0.01);

        assert_eq!(
            context.checkpoint_path,
            Some("test_checkpoint.bin".to_string())
        );
        assert_eq!(context.learning_rate_backup, Some(0.01));

        context.record_error("Memory allocation failed");
        context.record_error("Gradient explosion");

        assert_eq!(context.error_pattern.len(), 2);
        assert_eq!(
            context.get_recovery_suggestion() as u8,
            RecoveryStrategy::GradientClipping { threshold: 1.0 } as u8
        );
    }

    #[test]
    fn test_wasm_error_context() {
        let mut context = WasmErrorContext::new("test_operation");
        context.memory_available = Some(1024 * 1024 * 1024); // 1GB
        context.webgl_support = Some(true);

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.memory_available, Some(1024 * 1024 * 1024));
        assert_eq!(context.webgl_support, Some(true));
    }

    #[test]
    fn test_error_handler() {
        let mut handler = ErrorHandler::new();
        let recovery_context = RecoveryContext::new(RecoveryStrategy::Retry);
        handler = handler.with_recovery_context(recovery_context);

        let test_error = RuvFannError::Network {
            category: NetworkErrorCategory::Topology,
            message: "Test error".to_string(),
            context: None,
        };

        let result = handler.handle_error(test_error, |_| Ok(42));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_wasm_error_handling() {
        let mut handler = ErrorHandler::new();
        let wasm_context = WasmErrorContext::new("test_operation");
        handler = handler.with_wasm_context(wasm_context);

        let memory_error = RuvFannError::Memory {
            message: "Out of memory".to_string(),
            requested_bytes: Some(1024),
            available_bytes: Some(512),
        };

        let handled_error = handler.handle_wasm_error(memory_error);

        match handled_error {
            RuvFannError::Wasm { operation, .. } => {
                assert_eq!(operation, "test_operation");
            }
            _ => panic!("Expected WASM error"),
        }
    }

    #[test]
    fn test_recovery_strategies() {
        let retry_strategy = RecoveryStrategy::Retry;
        let reduce_lr = RecoveryStrategy::ReduceLearningRate { factor: 0.5 };
        let gradient_clip = RecoveryStrategy::GradientClipping { threshold: 1.0 };
        let memory_opt = RecoveryStrategy::MemoryOptimization;

        // Test that all strategies can be created and compared
        assert!(matches!(retry_strategy, RecoveryStrategy::Retry));
        assert!(matches!(
            reduce_lr,
            RecoveryStrategy::ReduceLearningRate { .. }
        ));
        assert!(matches!(
            gradient_clip,
            RecoveryStrategy::GradientClipping { .. }
        ));
        assert!(matches!(memory_opt, RecoveryStrategy::MemoryOptimization));
    }

    #[test]
    fn test_error_macros() {
        let network_err = network_error!(NetworkErrorCategory::Topology, "Test network error");
        let wasm_err = wasm_error!("test_op", "Test WASM error");
        let perf_err = performance_error!("Slow operation", "latency", 100.0, 200.0, 0.5);

        match network_err {
            RuvFannError::Network {
                category, message, ..
            } => {
                assert_eq!(category, NetworkErrorCategory::Topology);
                assert_eq!(message, "Test network error");
            }
            _ => panic!("Expected Network error"),
        }

        match wasm_err {
            RuvFannError::Wasm {
                operation, message, ..
            } => {
                assert_eq!(operation, "test_op");
                assert_eq!(message, "Test WASM error");
            }
            _ => panic!("Expected WASM error"),
        }

        match perf_err {
            RuvFannError::Performance {
                metric,
                expected,
                actual,
                ..
            } => {
                assert_eq!(metric, "latency");
                assert_eq!(expected, 100.0);
                assert_eq!(actual, 200.0);
            }
            _ => panic!("Expected Performance error"),
        }
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_operation")
            .with_network_id("network_1")
            .with_layer(2)
            .with_neuron(100)
            .with_epoch(50)
            .with_info("custom_key", "custom_value");

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.network_id, Some("network_1".to_string()));
        assert_eq!(context.layer_index, Some(2));
        assert_eq!(context.neuron_index, Some(100));
        assert_eq!(context.epoch, Some(50));
        assert_eq!(
            context.additional_info.get("custom_key"),
            Some(&"custom_value".to_string())
        );
    }
}
