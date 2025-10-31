mod recurring;
mod swap;
mod token;
mod trigger;
mod ultra;

#[tokio::main]
async fn main() {
    ultra::ultra().await;
}
