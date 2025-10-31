mod lib;
mod debug_swap;

use debug_swap::debug_swap;

#[tokio::main]
async fn main() {
    debug_swap().await;
}
