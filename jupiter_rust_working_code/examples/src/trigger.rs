use jup::sign_transaction;
use jup_ag_sdk::{
    JupiterClient,
    types::{
        CreateTriggerOrder,
        ExecuteTriggerOrder,
        GetTriggerOrders,
        OrderStatus,
        // CancelTriggerOrder,
    },
};

pub async fn trigger() {
    // Initialize the Jupiter client
    let client = JupiterClient::new("https://lite-api.jup.ag");

    // User's wallet address
    let user_address = "your wallet address"; // Replace with your actual wallet address

    // Create a trigger order to swap 10 USDC for 20 JUP
    let input_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC
    let output_mint = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"; // JUP
    let input_amount = 10_000_000; // 10 USDC
    let output_amount = 20_000_000; // 20 JUP

    // Construct the trigger order request
    let create_order_request = CreateTriggerOrder::new(
        input_mint,
        output_mint,
        user_address,
        user_address,
        input_amount,
        output_amount,
    );

    // Get the unsigned transaction and request ID for creating the order
    let create_response = client
        .create_trigger_order(&create_order_request)
        .await
        .expect("Failed to create trigger order");

    println!("📦 Created Trigger Order: {:?}", create_response);

    // Sign the transaction locally
    let signed_tx_base64 = sign_transaction(create_response.transaction);

    // Send the signed transaction for execution
    let execute_request = ExecuteTriggerOrder {
        request_id: create_response.request_id,
        signed_transaction: signed_tx_base64,
    };

    let execute_response = client
        .execute_trigger_order(&execute_request)
        .await
        .expect("Failed to execute trigger order");

    println!(
        "🚀 Executed Trigger Order, singnature: {}",
        execute_response.signature
    );

    // Fetch the list of trigger orders for the user
    let get_orders_params = GetTriggerOrders {
        user: user_address.to_string(),
        order_status: OrderStatus::History,
        input_mint: None,
        output_mint: None,
        include_failed_tx: None,
        page: None,
    };

    let order_history = client
        .get_trigger_orders(&get_orders_params)
        .await
        .expect("Failed to get trigger orders");

    println!("📖 Trigger Order History: {:?}", order_history);

    // ---  Cancel a trigger order ---
    /*
    let cancel_request = CancelTriggerOrder::new(
        user_address,
        "HeyWQcYd9t6BFGDfwh3w13F9KmiSNyPJuRPm49kiynFs", // Order ID to cancel
    );

    let cancel_response = client
        .cancel_trigger_order(&cancel_request)
        .await
        .expect("Failed to cancel trigger order");

    println!("❌ Cancelled Trigger Order: {:?}", cancel_response);

    // Sign and execute cancel transaction same as above
    */
}
