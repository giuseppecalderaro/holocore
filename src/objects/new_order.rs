use fixed::FixedU64;
use serde::{ Deserializer, Deserialize, Serialize };
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    LimitMaker
}

fn deserialize_f64_to_fixed<'de, D: Deserializer<'de>>(deserializer: D) -> Result<FixedU64<32>, D::Error>
{
    match Deserialize::deserialize(deserializer) {
        Ok(value) => Ok(FixedU64::<32>::from_num::<f64>(value)),
        Err(e) => Err(e)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewOrderV1 {
    id: Uuid,
    timestamp: i64,
    sequence_nr: u64,
    correlation_id: Uuid,

    pub symbol: String,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    #[serde(deserialize_with = "deserialize_f64_to_fixed")]
    pub quantity: FixedU64<32>,
    #[serde(deserialize_with = "deserialize_f64_to_fixed")]
    pub price: FixedU64<32>
}

impl NewOrderV1 {
    pub fn new(id: Uuid, timestamp: i64, sequence_nr: u64, correlation_id: Uuid, symbol: &str, order_side: OrderSide, order_type: OrderType, quantity: FixedU64<32>, price: FixedU64<32>) -> Self {
        Self {
            id,
            timestamp,
            sequence_nr,
            correlation_id,
            symbol: String::from(symbol),
            order_side,
            order_type,
            quantity,
            price
        }
    }

    // Getters
    pub fn get_type(&self) -> u64 {
        20
    }

    pub fn get_version(&self) -> u64 {
        1
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_sequence_number(&self) -> u64 {
        self.sequence_nr
    }

    pub fn get_correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
