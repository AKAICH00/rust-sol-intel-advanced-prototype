use once_cell::sync::Lazy;
use prometheus::{register_int_counter, IntCounter};

/// Total WebSocket ticks received.
pub static TICKS_RECEIVED: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("ticks_received_total", "Total WebSocket ticks received").unwrap()
});

/// Total trading signals emitted.
pub static SIGNALS_EMITTED: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("signals_emitted_total", "Total trading signals emitted").unwrap()
});
