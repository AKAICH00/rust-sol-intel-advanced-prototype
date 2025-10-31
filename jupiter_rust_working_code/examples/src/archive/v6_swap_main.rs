mod v6_swap;

#[tokio::main]
async fn main() {
    v6_swap::v6_swap().await;
}
