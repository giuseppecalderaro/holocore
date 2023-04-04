use serde::{ Deserialize, Serialize };

use crate::objects::new_order::{ NewOrderV1, OrderSide, OrderType };

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum BinanceOrderSide {
    Buy,
    Sell
}

impl From<BinanceOrderSide> for String {
    fn from(item: BinanceOrderSide) -> Self {
        match item {
            BinanceOrderSide::Buy => String::from("BUY"),
            BinanceOrderSide::Sell => String::from("SELL"),
        }
    }
}

impl From<OrderSide> for BinanceOrderSide {
    fn from(item: OrderSide) -> Self {
        match item {
            OrderSide::Buy => BinanceOrderSide::Buy,
            OrderSide::Sell => BinanceOrderSide::Sell
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum BinanceOrderType {
    Limit,
    Market,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    LimitMaker
}

impl From<BinanceOrderType> for String {
    fn from(item: BinanceOrderType) -> Self {
        match item {
            BinanceOrderType::Limit => String::from("LIMIT"),
            BinanceOrderType::Market => String::from("MARKET"),
            BinanceOrderType::StopLoss => String::from("STOP_LOSS"),
            BinanceOrderType::StopLossLimit => String::from("STOP_LOSS_LIMIT"),
            BinanceOrderType::TakeProfit => String::from("TAKE_PROFIT"),
            BinanceOrderType::TakeProfitLimit => String::from("TAKE_PROFIT_LIMIT"),
            BinanceOrderType::LimitMaker => String::from("LIMIT_MAKER")
        }
    }
}

impl From<OrderType> for BinanceOrderType {
    fn from(item: OrderType) -> Self {
        match item {
            OrderType::Limit => BinanceOrderType::Limit,
            OrderType::Market => BinanceOrderType::Market,
            OrderType::StopLoss => BinanceOrderType::StopLoss,
            OrderType::StopLossLimit => BinanceOrderType::StopLossLimit,
            OrderType::TakeProfit => BinanceOrderType::TakeProfit,
            OrderType::TakeProfitLimit => BinanceOrderType::TakeProfitLimit,
            OrderType::LimitMaker => BinanceOrderType::LimitMaker
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceAccountInformation {
    pub maker_commission: f32,
    pub taker_commission: f32,
    pub buyer_commission: f32,
    pub seller_commission: f32,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub balances: Vec<BinanceBalance>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceOrder {
    pub symbol: String,
    pub order_id: u64,
    pub order_list_id: i64,
    pub client_order_id: String,
    pub price: String,
    pub orig_qty: String,
    pub executed_qty: String,
    pub cummulative_quote_qty: String,
    pub status: String,
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub side: String,
    pub stop_price: String,
    pub iceberg_qty: String,
    pub time: u64,
    pub update_time: u64,
    pub is_working: bool,
    pub orig_quote_order_qty: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BinanceNewOrder {
    pub symbol: String,
    pub order_side: BinanceOrderSide,
    pub order_type: BinanceOrderType,
    pub quantity: String,
    pub price: String
}

impl From<NewOrderV1> for BinanceNewOrder {
    fn from(item: NewOrderV1) -> Self {
        Self {
            symbol: item.symbol,
            order_side: BinanceOrderSide::from(item.order_side),
            order_type: BinanceOrderType::from(item.order_type),
            quantity: item.quantity.to_string(),
            price: item.price.to_string()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceAckOrder {
    pub symbol: String,
    pub order_id: u64,
    pub order_list_id: i64,
    pub client_order_id: String,
    pub transact_time: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceOrderCanceled {
    pub symbol: String,
    pub orig_client_order_id: Option<String>,
    pub order_id: Option<u64>,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceContentError {
    pub code: i16,
    pub msg: String,
}
