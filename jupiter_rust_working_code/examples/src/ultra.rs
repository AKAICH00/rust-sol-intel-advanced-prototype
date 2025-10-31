use jup::sign_transaction;
use jup_ag_sdk::{
    JupiterClient,
    types::{UltraExecuteOrderRequest, UltraOrderRequest},
};

pub async fn ultra() {
    // Initialize the Jupiter client with the Lite API endpoint
    let client = JupiterClient::new("https://lite-api.jup.ag");

    // Define the swap: 10 USDC (6 decimals) to SOL
    let input_token = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC
    let output_token = "So11111111111111111111111111111111111111112"; // SOL
    let amount = 10_000_000; // 10 USDC (in micro units)

    // Create an order request and specify the taker's wallet address
    let order_request = UltraOrderRequest::new(input_token, output_token, amount)
        .add_taker("your taker wallet address");

    // Fetch the unsigned transaction and request ID from Jupiter
    let order_response = client
        .get_ultra_order(&order_request)
        .await
        .expect("Failed to fetch ultra order");

    let unsigned_tx_base64 = order_response
        .transaction
        .expect("No transaction found in ultra order response");

    // Sign the base64-encoded transaction locally
    let signed_tx_base64 = sign_transaction(unsigned_tx_base64);

    // Prepare the execution request with the signed transaction
    let execute_request = UltraExecuteOrderRequest {
        signed_transaction: signed_tx_base64,
        request_id: order_response.request_id,
    };

    // Send the signed transaction to Jupiter for execution
    let execute_response = client
        .ultra_execute_order(&execute_request)
        .await
        .expect("Failed to execute transaction");

    // Print the transaction signature
    let tx_signature = execute_response
        .signature
        .expect("No signature found in execution response");

    println!("✅ Transaction submitted: {}", tx_signature);
}

pub async fn ultra_token_search() {
    let client = JupiterClient::new("https:://lite-api.jup.ag");

    let mints = vec![String::from("JUP")];

    let data = client
        .ultra_token_search(&mints)
        .await
        .expect("failed to get token info");

    println!("data: {:?}", data);
}
