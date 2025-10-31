use super::OrderStatus;
use serde::{Deserialize, Serialize};

/// Represents a request to create a recurring order, either time-based or price-based.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRecurringOrderRequest {
    /// The wallet address initiating the order.
    pub user: String,
    /// The mint address of the input SPL token.
    pub input_mint: String,
    /// The mint address of the output SPL token.
    pub output_mint: String,
    /// Parameters for the recurring order, either time-based or price-based.
    pub params: OrderParams,
}

/// Enum wrapper for the two types of recurring order strategies:
/// - `TimeWrapper`: Splits funds over time.
/// - `PriceWrapper`: Splits funds based on price increments.
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum OrderParams {
    /// Time-based recurring order parameters.
    TimeWrapper { time: TimeParams },
    /// Price-based recurring order parameters.
    PriceWrapper { price: PriceParams },
}

/// Parameters for creating a time-based recurring order.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimeParams {
    /// Total input token amount to be split across orders.
    pub in_amount: u64,
    /// Number of orders to place.
    pub number_of_orders: u64,
    /// Time interval (in seconds) between each order.
    pub interval: u64,
    /// Optional minimum price threshold for executing an order.
    pub min_price: Option<f64>,
    /// Optional maximum price threshold for executing an order.
    pub max_price: Option<f64>,
    /// Optional Unix timestamp to schedule when the order starts.
    pub start_at: Option<u64>,
}

/// Parameters for creating a price-based recurring order.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PriceParams {
    /// Total amount to be deposited for the strategy.
    pub deposit_amount: u64,
    /// USDC value increment that triggers each order.
    pub increment_usdc_value: u64,
    /// Minimum time interval (in seconds) between each order execution.
    pub interval: u64,
    /// Optional Unix timestamp to schedule when the order starts.
    pub start_at: Option<u64>,
}

impl CreateRecurringOrderRequest {
    /// Creates a new time-based recurring order.
    ///
    /// # Arguments
    ///
    /// * `user` - The user wallet address.
    /// * `input_mint` - SPL token mint address for the input token.
    /// * `output_mint` - SPL token mint address for the output token.
    /// * `in_amount` - Total input amount to be divided.
    /// * `number_of_orders` - How many times the order should be placed.
    /// * `interval` - Time gap between each order (in seconds).
    pub fn new_time_order(
        user: impl Into<String>,
        input_mint: impl Into<String>,
        output_mint: impl Into<String>,
        in_amount: u64,
        number_of_orders: u64,
        interval: u64,
    ) -> Self {
        let params = TimeParams {
            in_amount,
            number_of_orders,
            interval,
            min_price: None,
            max_price: None,
            start_at: None,
        };
        Self {
            user: user.into(),
            input_mint: input_mint.into(),
            output_mint: output_mint.into(),
            params: OrderParams::TimeWrapper { time: params },
        }
    }

    /// Creates a new price-based recurring order.
    ///
    /// # Arguments
    ///
    /// * `user` - The user wallet address.
    /// * `input_mint` - SPL token mint address for the input token.
    /// * `output_mint` - SPL token mint address for the output token.
    /// * `deposit_amount` - Total amount to deposit.
    /// * `increment_usdc_value` - Price increment in USDC for each order trigger.
    /// * `interval` - Minimum interval between executions (in seconds).
    pub fn new_price_order(
        user: impl Into<String>,
        input_mint: impl Into<String>,
        output_mint: impl Into<String>,
        deposit_amount: u64,
        increment_usdc_value: u64,
        interval: u64,
    ) -> Self {
        let params = PriceParams {
            deposit_amount,
            increment_usdc_value,
            interval,
            start_at: None,
        };

        Self {
            user: user.into(),
            input_mint: input_mint.into(),
            output_mint: output_mint.into(),
            params: OrderParams::PriceWrapper { price: params },
        }
    }

    /// Sets the `start_at` Unix timestamp to delay the start of the recurring order.
    pub fn with_start_at(mut self, start_at: u64) -> Self {
        match &mut self.params {
            OrderParams::TimeWrapper { time } => time.start_at = Some(start_at),
            OrderParams::PriceWrapper { price } => price.start_at = Some(start_at),
        }
        self
    }

    /// Sets the optional `min_price` threshold for a time-based order.
    pub fn with_min_price(mut self, price: f64) -> Self {
        if let OrderParams::TimeWrapper { time } = &mut self.params {
            time.min_price = Some(price);
        }
        self
    }

    /// Sets the optional `max_price` threshold for a time-based order.
    pub fn with_max_price(mut self, price: f64) -> Self {
        if let OrderParams::TimeWrapper { time } = &mut self.params {
            time.max_price = Some(price);
        }
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelRecurringOrderRequest {
    pub order: String,

    pub recurring_type: RecurringOrderType,

    pub user: String,
}

impl CancelRecurringOrderRequest {
    pub fn new(
        order: impl Into<String>,
        recurring_type: RecurringOrderType,
        user: impl Into<String>,
    ) -> Self {
        Self {
            order: order.into(),
            recurring_type,
            user: user.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecurringOrderType {
    Time,
    Price,
    /// All type is to only be used to get all recurring orders not a actual order type
    All,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceDeposit {
    pub amount: u64,

    pub order: String,

    pub user: String,
}

impl PriceDeposit {
    /// # Arguments
    ///
    /// * `amount` - The amount to deposit
    /// * `order` - The recurring order account address
    /// * `user` - The user account address
    pub fn new(amount: u64, order: impl Into<String>, user: impl Into<String>) -> Self {
        Self {
            amount,
            order: order.into(),
            user: user.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceWithdraw {
    /// If no amount is provided, it will withdraw the entire amount
    pub amount: u64,

    pub order: String,

    pub user: String,

    /// Possible values: [In, Out]
    pub input_or_output: String,
}

impl PriceWithdraw {
    /// # Arguments
    ///
    /// * `amount` - The amount to withdraw
    /// * `order` - The recurring order account address
    /// * `user` - The user account address
    /// * `input_or_output` - The withdrawal direction ("In" or "Out")
    pub fn new(
        amount: u64,
        order: impl Into<String>,
        user: impl Into<String>,
        input_or_output: impl Into<String>,
    ) -> Self {
        Self {
            amount,
            order: order.into(),
            user: user.into(),
            input_or_output: input_or_output.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringResponse {
    pub request_id: String,

    /// Unsigned base-64 encoded transaction
    pub transaction: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRecurringRequest {
    pub request_id: String,

    pub signed_transaction: String,
}

impl ExecuteRecurringRequest {
    pub fn new(request_id: impl Into<String>, signed_transaction: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            signed_transaction: signed_transaction.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRecurringResponse {
    pub signature: String,

    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRecurringOrders {
    pub recurring_type: RecurringOrderType,
    pub order_status: OrderStatus,
    pub user: String,
    pub page: u64,
    pub mint: Option<String>,
    pub include_failed_tx: bool,
}

impl GetRecurringOrders {
    /// Basic constructor
    pub fn new(
        recurring_type: RecurringOrderType,
        order_status: OrderStatus,
        user: impl Into<String>,
    ) -> Self {
        Self {
            recurring_type,
            order_status,
            user: user.into(),
            page: 1,
            mint: None,
            include_failed_tx: false,
        }
    }

    /// Customize page number
    pub fn with_page(mut self, page: u64) -> Self {
        self.page = page;
        self
    }

    /// Filter by a specific mint
    pub fn with_mint(mut self, mint: impl Into<String>) -> Self {
        self.mint = Some(mint.into());
        self
    }

    /// Include failed transactions
    pub fn include_failed(mut self) -> Self {
        self.include_failed_tx = true;
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringOrders {
    pub order_status: OrderStatus,
    pub page: u64,
    pub total_pages: u64,
    pub user: String,
    #[serde(default)]
    pub time: Option<Vec<TimeOrder>>,
    #[serde(default)]
    pub price: Option<Vec<PriceOrder>>,
    #[serde(default)]
    pub all: Option<Vec<Order>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Order {
    Time(TimeOrder),
    Price(PriceOrder),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceOrder {
    pub close_tx: String,
    pub closed_by: String,
    pub created_at: String,
    pub estimated_usdc_value_spent: String,
    pub in_deposited: String,
    pub in_left: String,
    pub in_used: String,
    pub in_withdrawn: String,
    pub incremental_usd_value: String,
    pub input_mint: String,
    pub open_tx: String,
    pub order_interval: String,
    pub order_key: String,
    pub out_received: String,
    pub out_withdrawn: String,
    pub output_mint: String,
    pub raw_estimated_usdc_value_spent: String,
    pub raw_in_deposited: String,
    pub raw_in_left: String,
    pub raw_in_used: String,
    pub raw_in_withdrawn: String,
    pub raw_incremental_usd_value: String,
    pub raw_out_received: String,
    pub raw_out_withdrawn: String,
    pub raw_supposed_usd_value: String,
    pub start_at: String,
    pub status: String,
    pub supposed_usd_value: String,
    pub trades: Vec<Trade>,
    pub updated_at: String,
    pub user_pubkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeOrder {
    pub close_tx: String,
    pub created_at: String,
    pub cycle_frequency: String,
    pub in_amount_per_cycle: String,
    pub in_deposited: String,
    pub in_used: String,
    pub in_withdrawn: String,
    pub input_mint: String,
    pub max_out_amount: String,
    pub min_out_amount: String,
    pub open_tx: String,
    pub order_key: String,
    pub out_received: String,
    pub out_withdrawn: String,
    pub output_mint: String,
    pub raw_in_amount_per_cycle: String,
    pub raw_in_deposited: String,
    pub raw_in_used: String,
    pub raw_in_withdrawn: String,
    pub raw_max_out_amount: String,
    pub raw_min_out_amount: String,
    pub raw_out_received: String,
    pub raw_out_withdrawn: String,
    pub trades: Vec<Trade>,
    pub updated_at: String,
    pub user_closed: bool,
    pub user_pubkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub action: String,
    pub confirmed_at: String,
    pub fee_amount: String,
    pub fee_mint: String,
    pub input_amount: String,
    pub input_mint: String,
    pub keeper: String,
    pub order_key: String,
    pub output_amount: String,
    pub output_mint: String,
    #[serde(default)]
    pub product_meta: Option<ProductMeta>,
    pub raw_fee_amount: String,
    pub raw_input_amount: String,
    pub raw_output_amount: String,
    pub tx_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProductMeta {
    pub new_actual_usdc_value: String,
    pub value: String,
}
