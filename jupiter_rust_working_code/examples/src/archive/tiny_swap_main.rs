mod lib;
mod tiny_swap;

use tiny_swap::tiny_swap;

#[tokio::main]
async fn main() {
    tiny_swap().await;
}
