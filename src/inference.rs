use anyhow::Result;
use ndarray::Array3;
use once_cell::sync::Lazy;
use ort::execution_providers::CPUExecutionProvider;
use ort::init;
use ort::session::builder::{GraphOptimizationLevel, SessionBuilder};
use ort::session::Session;
use ort::{inputs, value::Tensor};
use prometheus::{register_histogram_vec, HistogramVec};
use tokio::sync::Mutex;

/// Inference engine powered by ONNX Runtime.
/// ONNX Runtime inference engine for sequential features.
/// ONNX Runtime inference engine for sequential features.
/// ONNX Runtime inference engine for sequential features.
pub struct InferenceEngine {
    /// ONNX Runtime session; protected by a mutex for mutable access
    session: Mutex<Session>,
}

/// Histogram for inference latency in seconds.
static INFERENCE_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "inference_latency_seconds",
        "Inference latency in seconds",
        &[]
    )
    .expect("failed to create histogram")
});

impl InferenceEngine {
    /// Create a new inference engine, loading the specified ONNX model.
    pub async fn new(model_path: &str) -> Result<Self> {
        // Commit global environment with CPU execution provider (boolean indicates first init)
        let _ = init()
            .with_execution_providers([CPUExecutionProvider::default().build()])
            .commit()?;
        // Build session with optimizations and load model
        let session = SessionBuilder::new()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .commit_from_file(model_path)?;
        Ok(Self {
            session: tokio::sync::Mutex::new(session),
        })
    }

    /// Run inference on the provided feature tensor, returning (embedding, anomaly_score).
    pub async fn predict(&self, features: Array3<f32>) -> Result<(Vec<f32>, f32)> {
        tracing::info!(
            "[Inference] running ONNX inference, input shape: {:?}",
            features.dim()
        );
        // Flatten array into contiguous Vec and record shape
        let shape: Vec<i64> = features.shape().iter().map(|&d| d as i64).collect();
        let raw_data: Vec<f32> = features.into_raw_vec();
        // Build input tensor
        let input = Tensor::from_array((shape.clone(), raw_data.clone()))?;
        // Run session (positional inputs)
        // Run inference (lock session for mutable access)
        let mut session = self.session.lock().await;
        // Measure inference latency
        let timer = INFERENCE_LATENCY
            .with_label_values(&[] as &[&str])
            .start_timer();
        let outputs = session.run(inputs![input])?;
        timer.observe_duration();

        // Extract embedding (first output) and reconstruction (second)
        let emb_arr = outputs[0].try_extract_array::<f32>()?;
        let rec_arr = outputs[1].try_extract_array::<f32>()?;
        let embedding: Vec<f32> = emb_arr.iter().cloned().collect();
        // Compute anomaly score as mean squared error over reconstruction
        let mut mse = 0.0_f32;
        for (a, b) in raw_data.iter().zip(rec_arr.iter()) {
            mse += (a - b).powi(2);
        }
        if !raw_data.is_empty() {
            mse /= raw_data.len() as f32;
        }
        Ok((embedding, mse))
    }
}
