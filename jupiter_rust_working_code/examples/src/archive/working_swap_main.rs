mod lib;
mod working_swap;

use working_swap::working_swap;

#[tokio::main]
async fn main() {
    working_swap().await;
}
