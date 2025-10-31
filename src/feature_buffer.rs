use crate::types::TickData;

/// Simple rolling buffer to collect ticks for feature extraction.
pub struct FeatureBuffer {
    window_size: usize,
    data: Vec<TickData>,
}

impl FeatureBuffer {
    pub fn new(window_size: usize) -> Self {
        FeatureBuffer {
            window_size,
            data: Vec::with_capacity(window_size),
        }
    }

    pub fn push(&mut self, tick: TickData) {
        self.data.push(tick);
        if self.data.len() > self.window_size {
            self.data.remove(0);
        }
    }

    pub fn is_ready(&self) -> bool {
        self.data.len() == self.window_size
    }

    /// Extracts a simple feature tensor of shape (1, window_size, 3):
    /// [price, price_diff, volume] per tick.
    pub fn extract_features(&self) -> ndarray::Array3<f32> {
        let mut arr = ndarray::Array3::<f32>::zeros((1, self.window_size, 3));
        for (i, tick) in self.data.iter().enumerate() {
            let price = tick.price as f32;
            let volume = tick.volume as f32;
            let diff = if i > 0 {
                price - (self.data[i - 1].price as f32)
            } else {
                0.0
            };
            arr[[0, i, 0]] = price;
            arr[[0, i, 1]] = diff;
            arr[[0, i, 2]] = volume;
        }
        arr
    }
}
