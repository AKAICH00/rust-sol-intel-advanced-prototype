use jup_ag_sdk::{JupiterClient, client};

pub async fn token_balances() {
    let client = JupiterClient::new("https://lite-api.jup.ag");

    let address = "EXBdeRCdiNChKyD7akt64n9HgSXEpUtpPEhmbnm4L6iH";

    let token_balances = client
        .get_token_balances(address)
        .await
        .expect("Failed to get token balances");

    let sol_balance = token_balances
        .get("SOL")
        .expect("provided address does not have SOL balance");

    println!("SOL balance for {}: {:?}", address, sol_balance.ui_amount);

    let usdc_balance = token_balances
        .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
        .expect("provided address does not have USDC balance");

    println!("USDC balance for {}: {:?}", address, usdc_balance.ui_amount);
}

pub async fn token_price() {
    let client = JupiterClient::new("https://lite-api.jup.ag");

    let token_mints = vec![
        "So11111111111111111111111111111111111111112".to_string(),
        "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".to_string(),
    ];

    let price = client
        .get_tokens_price(&token_mints)
        .await
        .expect("Failed to get token price");

    let sol_price = price
        .get(token_mints[0].as_str())
        .expect("SOL price not found");

    println!("1 SOL price in USDC: {}", sol_price.usd_price);

    let jup_price = price
        .get(token_mints[1].as_str())
        .expect("Jup Token price not found");

    println!("1 JUP price USDC:  {}", jup_price.usd_price);
}

pub async fn token_info() {
    let client = JupiterClient::new("https://lite-api.jup.ag");

    let token_mints = vec![
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
        "3nHtBZqn1kAZ3fHMZnwfhkX6r8MtSKX1nN9wuWVspump".to_string(), // some shit coin from pump.fun
    ];

    let info = client
        .shield(&token_mints)
        .await
        .expect("failed the token info");

    let usdc_warnings = info
        .warnings
        .get(token_mints[0].as_str())
        .expect("USDC not found");

    println!("USDC warnings: {:?}", usdc_warnings);

    let random_shitter = info
        .warnings
        .get(token_mints[1].as_str())
        .expect("random_shitter not found");

    println!("random_shitter warnings: {:?}", random_shitter);
}

pub async fn get_tokens_from_tags() {
    let tags = vec![String::from("lst")];

    let client = JupiterClient::new("https:://lite-api.jup.ag");

    let mints = client
        .get_mints_by_tags(&tags)
        .await
        .expect("failed to get mint");

    println!("mints: {}", mints.len())
}

pub async fn get_trending_tokens() {
    let client = JupiterClient::new("https:://lite-api.jup.ag");

    let tokens = client
        .get_tokens_by_category(
            jup_ag_sdk::types::Category::TopTrending,
            jup_ag_sdk::types::Interval::TwentyFourHours,
            Some(10),
        )
        .await
        .expect("failed to get trending tokens");

    println!("trending tokens: {:?}", tokens.len());
}
