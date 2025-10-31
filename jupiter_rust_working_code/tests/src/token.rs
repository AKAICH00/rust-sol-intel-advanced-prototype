#[cfg(test)]
mod token_tests {
    use crate::common::{JUP_MINT, SOL_MINT, USDC_MINT, create_test_client};

    #[tokio::test]
    async fn test_get_token_balances() {
        let client = create_test_client();
        let tokens = client
            .get_token_balances("372sKPyyiwU5zYASHzqvYY48Sv4ihEujfN5rGFKhVQ9j")
            .await
            .expect("failed to get token balances");

        assert_eq!(
            tokens
                .get("2zMMhcVQEXDtdE6vsFS7S7D5oUodfJHE8vd1gnBouauv")
                .expect("pengu token not found")
                .amount,
            516176755.to_string(),
        )
    }

    #[tokio::test]
    async fn test_get_token_prices() {
        let client = create_test_client();
        let token_mints = vec![SOL_MINT.to_string(), USDC_MINT.to_string()];

        let res = client
            .get_tokens_price(&token_mints)
            .await
            .expect("failed to get token prices");

        let usdc_price: f64 = res.get(USDC_MINT).expect("usdc price not found").usd_price;

        assert!(
            (0.9..=1.1).contains(&usdc_price),
            "USDC price {} is out of range (0.9 to 1.1)",
            usdc_price
        );
    }

    #[tokio::test]
    pub async fn test_token_info() {
        let client = create_test_client();

        let info = client
            .token_search(&[JUP_MINT.to_string()])
            .await
            .expect("failed to get token info");

        assert_eq!(info[0].decimals, 6, "JUP decimals should be 6");

        assert_eq!(info[0].symbol, "JUP")
    }

    #[tokio::test]
    pub async fn test_get_mints_by_tags() {
        let client = create_test_client();

        let tags = vec![String::from("lst")];

        let mints = client
            .get_mints_by_tags(&tags)
            .await
            .expect("failed to get mints by tags");

        assert!(mints.len() > 1000, "there are more that 1000 lst");
    }
}
