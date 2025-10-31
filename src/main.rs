mod database;
mod execution;
mod feature_buffer;
mod inference;
mod metrics;
mod questdb;
mod risk_manager;
mod types;
mod vector_store;
mod websocket;

use clap::Parser;
use execution::execute_trade;
use feature_buffer::FeatureBuffer;
use inference::InferenceEngine;
use once_cell::sync::Lazy;
use prometheus::{gather, Encoder, TextEncoder};
use questdb::QuestDBClient;
use std::sync::Arc;
use tracing_subscriber::prelude::*;
use vector_store::VectorStore;
use warp::Filter;
use websocket::stream_jupiter_websocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize OpenTelemetry + tracing subscriber for structured logging & tracing
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("memecoin_trading_engine")
        .install_simple()?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish()
        .with(telemetry)
        .init();

    // CLI options
    #[derive(Parser, Debug)]
    #[command(
        name = "memecoin_trading_engine",
        about = "Optimal Python+Rust memecoin trading"
    )]
    struct Opt {
        /// WebSocket URL for market data
        #[arg(long, default_value = "wss://api.jup.ag/ws/v1/quotes")]
        ws_url: String,
        /// Market symbol (e.g. BONK/SOL)
        #[arg(long, default_value = "BONK/SOL")]
        market: String,
        /// QuestDB HTTP URL
        #[arg(long, default_value = "http://localhost:9000")]
        questdb_url: String,
        /// Qdrant HTTP URL
        #[arg(long, default_value = "http://localhost:6334")]
        qdrant_url: String,
        /// ONNX model path
        #[arg(long, default_value = "lstm_autoencoder.onnx")]
        model_path: String,
        /// Feature rolling window size
        #[arg(long, default_value_t = 50)]
        window_size: usize,
        /// Confidence threshold for signals
        #[arg(long, default_value_t = 0.8)]
        threshold: f32,
        /// HTTP port for metrics & health endpoints
        #[arg(long, default_value_t = 9090)]
        metrics_port: u16,
    }
    let opt = Opt::parse();

    // Channels for ticks and trading signals
    let (tick_tx, mut tick_rx) = tokio::sync::mpsc::unbounded_channel();
    let (signal_tx, mut signal_rx) = tokio::sync::mpsc::unbounded_channel();

    // Initialize clients and engines
    let questdb = QuestDBClient::new(&opt.questdb_url);
    let inference = Arc::new(InferenceEngine::new(&opt.model_path).await?);
    let vector_store = Arc::new(VectorStore::new(&opt.qdrant_url).await?);

    // Task 1: WebSocket ingestion
    // Task 1: WebSocket ingestion
    {
        let tick_tx = tick_tx.clone();
        let url = opt.ws_url.clone();
        let market = opt.market.clone();
        tokio::spawn(async move {
            let _ = stream_jupiter_websocket(url, market, tick_tx).await;
        });
    }

    // Task 2: Feature engineering + Inference
    {
        let questdb = questdb.clone();
        let inference = Arc::clone(&inference);
        let vector_store = Arc::clone(&vector_store);
        let signal_tx = signal_tx.clone();
        tokio::spawn(async move {
            let mut buf = FeatureBuffer::new(opt.window_size);
            while let Some(tick) = tick_rx.recv().await {
                let _ = questdb.insert_tick(&tick).await;
                buf.push(tick.clone());
                if buf.is_ready() {
                    let features = buf.extract_features();
                    if let Ok((embedding, score)) = inference.predict(features).await {
                        let similar = vector_store
                            .find_similar(&embedding, 5)
                            .await
                            .unwrap_or_default();
                        let signal = types::analyze_pattern(&similar, score);
                        if signal.confidence > opt.threshold {
                            let _ = signal_tx.send(signal);
                        }
                        let _ = vector_store
                            .insert_pattern(&embedding, &types::PatternMetadata::from_tick(&tick))
                            .await;
                    }
                }
            }
        });
    }

    // Task 3: Execution engine
    {
        tokio::spawn(async move {
            while let Some(signal) = signal_rx.recv().await {
                let _ = execute_trade(signal).await;
            }
        });
    }

    // Metrics & health endpoints
    let metrics_route = {
        let metrics = warp::path("metrics").map(|| {
            let mut buffer = Vec::new();
            let encoder = TextEncoder::new();
            let mf = prometheus::gather();
            encoder.encode(&mf, &mut buffer).unwrap();
            warp::http::Response::builder()
                .header("content-type", encoder.format_type())
                .body(buffer)
                .unwrap()
        });
        let healthz = warp::path("healthz").map(|| {
            warp::http::Response::builder()
                .status(warp::http::StatusCode::OK)
                .body("OK")
                .unwrap()
        });
        metrics.or(healthz).boxed()
    };
    let metrics_port = opt.metrics_port;
    tokio::spawn(warp::serve(metrics_route).run(([0, 0, 0, 0], metrics_port)));

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    Ok(())
}
