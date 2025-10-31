mod lib;
mod raw_test;

use raw_test::raw_test;

#[tokio::main]
async fn main() {
    raw_test().await;
}
