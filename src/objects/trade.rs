use crate::components::sources::binance::model::BinanceTradeEvent;

use serde::{ Deserialize, Serialize };
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TradeV1 {
    id: Uuid,
    timestamp: i64,
    sequence_nr: u64,
    correlation_id: Uuid,
    source: String,
    exchange: String,
    asset_type: String,

    event_time: u64,
    symbol: String,
    trade_id: u64,
    price: String,
    quantity: String,
    buyer_order_id: u64,
    seller_order_id: u64,
    trade_order_time: u64,
    is_buyer_maker: bool,
    ignore: bool
}

impl TradeV1 {
    pub fn from_binance(id: Uuid,
                        timestamp: i64,
                        sequence_nr: u64,
                        correlation_id: Uuid,
                        source: &str,
                        exchange: &str,
                        asset_type: &str,
                        event: BinanceTradeEvent) -> Self {
        Self {
            id,
            timestamp,
            sequence_nr,
            correlation_id,
            source: String::from(source),
            exchange: String::from(exchange),
            asset_type: String::from(asset_type),
            event_time: event.event_time,
            symbol: event.symbol,
            trade_id: event.trade_id,
            price: event.price,
            quantity: event.qty,
            buyer_order_id: event.buyer_order_id,
            seller_order_id: event.seller_order_id,
            trade_order_time: event.trade_order_time,
            is_buyer_maker: event.is_buyer_maker,
            ignore: event.m_ignore
        }
    }

    // Getters
    pub fn get_type(&self) -> u64 {
        17
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

    pub  fn get_correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
