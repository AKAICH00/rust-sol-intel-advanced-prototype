mod lib;
mod helius_swap;

use helius_swap::helius_swap;

#[tokio::main]
async fn main() {
    helius_swap().await;
}
