use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickData {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
}

#[derive(Clone, Debug)]
pub struct PatternMetadata {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
}

impl PatternMetadata {
    pub fn from_tick(tick: &TickData) -> Self {
        PatternMetadata {
            symbol: tick.symbol.clone(),
            price: tick.price,
            volume: tick.volume,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Signal {
    pub confidence: f32,
}

pub fn analyze_pattern(_similar: &Vec<(Vec<f32>, f32)>, _score: f32) -> Signal {
    // Compute confidence based on average similarity scores and anomaly score.
    let avg_sim: f32 = if !_similar.is_empty() {
        _similar.iter().map(|(_, s)| *s).sum::<f32>() / _similar.len() as f32
    } else {
        0.0
    };
    // Higher anomaly score -> lower confidence, so invert score (assuming normalized <=1)
    let anomaly_factor = (1.0_f32 - _score).max(0.0);
    let confidence = (avg_sim * anomaly_factor).clamp(0.0, 1.0);
    crate::metrics::SIGNALS_EMITTED.inc_by((confidence * 1_000_000.0) as u64);
    Signal { confidence }
}
