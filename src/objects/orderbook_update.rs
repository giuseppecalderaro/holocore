use crate::components::sources::binance::model::BinanceOrderbookUpdateEvent;
use crate::objects::level::Level;

use fixed::FixedU64;
use serde::{ Deserialize, Serialize };
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderbookUpdateV1 {
    id: Uuid,
    timestamp: i64,
    sequence_nr: u64,
    correlation_id: Uuid,
    pub source: String,
    exchange: String,
    asset_type: String,

    event_type: String,
    pub event_time: u64,
    pub symbol: String,
    pub first_update_id: u64,
    pub final_update_id: u64,
    previous_final_update_id: Option<u64>,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>
}

impl OrderbookUpdateV1 {
    pub fn from_binance(id: Uuid,
                        timestamp: i64,
                        sequence_nr: u64,
                        correlation_id: Uuid,
                        source: &str,
                        exchange: &str,
                        asset_type: &str,
                        event: BinanceOrderbookUpdateEvent) -> Self {
        let mut bids = Vec::<Level>::new();
        for bid in event.bids {
            let price = match FixedU64::<32>::from_str(&bid.price) {
                Ok(price) => price,
                Err(e) => {
                    log::error!("Failed to parse bid price {} - {}", bid.price, e);
                    continue;
                }
            };

            let volume = match FixedU64::<32>::from_str(&bid.qty) {
                Ok(volume) => volume,
                Err(e) => {
                    log::error!("Failed to parse bid volume {} - {}", bid.qty, e);
                    continue;
                }
            };

            let level = Level::new(price, volume);
            bids.push(level);
        }

        let mut asks = Vec::<Level>::new();
        for ask in event.asks {
            let price = match FixedU64::<32>::from_str(&ask.price) {
                Ok(price) => price,
                Err(e) => {
                    log::error!("Failed to parse ask price {} - {}", ask.price, e);
                    continue;
                }
            };

            let volume = match FixedU64::<32>::from_str(&ask.qty) {
                Ok(volume) => volume,
                Err(e) => {
                    log::error!("Failed to parse ask volume {} - {}", ask.qty, e);
                    continue;
                }
            };

            let level = Level::new(price, volume);
            asks.push(level);
        }

        Self {
            id,
            timestamp,
            sequence_nr,
            correlation_id,
            source: String::from(source),
            exchange: String::from(exchange),
            asset_type: String::from(asset_type),
            event_type: event.event_type,
            event_time: event.event_time,
            symbol: event.symbol,
            first_update_id: event.first_update_id,
            final_update_id: event.final_update_id,
            previous_final_update_id: event.previous_final_update_id,
            bids,
            asks
        }
    }

    // Getters
    pub fn get_type(&self) -> u64 {
        18
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
