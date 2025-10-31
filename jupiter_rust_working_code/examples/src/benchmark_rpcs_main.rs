mod lib;
mod benchmark_rpcs;

use benchmark_rpcs::benchmark_rpcs;

#[tokio::main]
async fn main() {
    benchmark_rpcs().await;
}
