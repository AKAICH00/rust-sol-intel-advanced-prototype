use jup::sign_transaction;
use jup_ag_sdk::{
    JupiterClient,
    types::{CreateRecurringOrderRequest, ExecuteRecurringRequest},
};

async fn recurring() {
    let client = JupiterClient::new("https://lite-api.jup.ag");

    // create a recurring time based order to swap 100 USDC for SOL every 10 days, with a duration
    // of 86400 seconds (1 day)
    let recurring_order_request = CreateRecurringOrderRequest::new_time_order(
        "your wallet address",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "So11111111111111111111111111111111111111112",
        1_000_000_000,
        10,
        86400,
    );

    let create_order_response = client
        .create_recurring_order(&recurring_order_request)
        .await
        .expect("Failed to create recurring order");

    let base64_signed_tx = sign_transaction(create_order_response.transaction);

    let execute_request = ExecuteRecurringRequest {
        request_id: create_order_response.request_id,
        signed_transaction: base64_signed_tx,
    };

    let execute_res = client
        .execute_recurring_order(&execute_request)
        .await
        .expect("Failed to execute recurring order");

    println!("signature: {}", execute_res.signature)

    // TODO: add other api calls here
}
