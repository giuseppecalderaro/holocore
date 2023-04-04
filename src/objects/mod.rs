pub mod common;
pub mod mock;
pub mod kline;
pub mod level;
pub mod market;
pub mod new_order;
pub mod orderbook;
pub mod orderbook_update;
pub mod trade;

use serde::{ Deserialize, Serialize };
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Objects {
    MockV1(mock::MockV1),
    KlineV1(kline::KlineV1),
    NewOrderV1(new_order::NewOrderV1),
    OrderbookV1(orderbook::OrderbookV1),
    OrderbookUpdateV1(orderbook_update::OrderbookUpdateV1),
    TradeV1(trade::TradeV1)
}

impl Objects {
    pub fn get_type(&self) -> u64 {
        match self {
            Self::MockV1(obj) => obj.get_type(),
            Self::KlineV1(obj) => obj.get_type(),
            Self::NewOrderV1(obj) => obj.get_type(),
            Self::OrderbookV1(obj) => obj.get_type(),
            Self::OrderbookUpdateV1(obj) => obj.get_type(),
            Self::TradeV1(obj) => obj.get_type()
        }
    }

    pub fn get_version(&self) -> u64 {
        match self {
            Self::MockV1(obj) => obj.get_version(),
            Self::KlineV1(obj) => obj.get_version(),
            Self::NewOrderV1(obj) => obj.get_version(),
            Self::OrderbookV1(obj) => obj.get_version(),
            Self::OrderbookUpdateV1(obj) => obj.get_version(),
            Self::TradeV1(obj) => obj.get_version()
        }
    }

    pub fn get_id(&self) -> Uuid {
        match self {
            Self::MockV1(obj) => obj.get_id(),
            Self::KlineV1(obj) => obj.get_id(),
            Self::NewOrderV1(obj) => obj.get_id(),
            Self::OrderbookV1(obj) => obj.get_id(),
            Self::OrderbookUpdateV1(obj) => obj.get_id(),
            Self::TradeV1(obj) => obj.get_id()
        }
    }

    pub fn get_timestamp(&self) -> i64 {
        match self {
            Self::MockV1(obj) => obj.get_timestamp(),
            Self::KlineV1(obj) => obj.get_timestamp(),
            Self::NewOrderV1(obj) => obj.get_timestamp(),
            Self::OrderbookV1(obj) => obj.get_timestamp(),
            Self::OrderbookUpdateV1(obj) => obj.get_timestamp(),
            Self::TradeV1(obj) => obj.get_timestamp()
        }
    }

    pub fn get_sequence_number(&self) -> u64 {
        match self {
            Self::MockV1(obj) => obj.get_sequence_number(),
            Self::KlineV1(obj) => obj.get_sequence_number(),
            Self::NewOrderV1(obj) => obj.get_sequence_number(),
            Self::OrderbookV1(obj) => obj.get_sequence_number(),
            Self::OrderbookUpdateV1(obj) => obj.get_sequence_number(),
            Self::TradeV1(obj) => obj.get_sequence_number()
        }
    }

    pub fn get_correlation_id(&self) -> Uuid {
        match self {
            Self::MockV1(obj) => obj.get_correlation_id(),
            Self::KlineV1(obj) => obj.get_correlation_id(),
            Self::NewOrderV1(obj) => obj.get_correlation_id(),
            Self::OrderbookV1(obj) => obj.get_correlation_id(),
            Self::OrderbookUpdateV1(obj) => obj.get_correlation_id(),
            Self::TradeV1(obj) => obj.get_correlation_id()
        }
    }
}
