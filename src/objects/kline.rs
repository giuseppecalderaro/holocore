use crate::components::sources::binance::model::BinanceKlineEvent;

use serde::{ Deserialize, Serialize };
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KlineV1 {
    id: Uuid,
    timestamp: i64,
    sequence_nr: u64,
    correlation_id: Uuid,
    source: String,
    exchange: String,
    asset_type: String,

    event_time: u64,
    open_time: i64,
    close_time: i64,
    symbol: String,
    interval: String,
    first_trade_id: i64,
    last_trade_id: i64,
    open: String,
    close: String,
    high: String,
    low: String,
    volume: String,
    number_of_trades: i64,
    is_final_bar: bool,
    quote_asset_volume: String,
    taker_buy_base_asset_volume: String,
    taker_buy_quote_asset_volume: String,
    ignore: String
}

impl KlineV1 {
    pub fn from_binance(id: Uuid,
                        timestamp: i64,
                        sequence_nr: u64,
                        correlation_id: Uuid,
                        source: &str,
                        exchange: &str,
                        asset_type: &str,
                        event: BinanceKlineEvent) -> Self {
        Self {
            id,
            timestamp,
            sequence_nr,
            correlation_id,
            source: String::from(source),
            exchange: String::from(exchange),
            asset_type: String::from(asset_type),
            event_time: event.event_time,
            open_time: event.kline.open_time,
            close_time: event.kline.close_time,
            symbol: event.kline.symbol,
            interval: event.kline.interval,
            first_trade_id: event.kline.first_trade_id,
            last_trade_id: event.kline.last_trade_id,
            open: event.kline.open,
            close: event.kline.close,
            high: event.kline.high,
            low: event.kline.low,
            volume: event.kline.volume,
            number_of_trades: event.kline.number_of_trades,
            is_final_bar: event.kline.is_final_bar,
            quote_asset_volume: event.kline.quote_asset_volume,
            taker_buy_base_asset_volume: event.kline.taker_buy_base_asset_volume,
            taker_buy_quote_asset_volume: event.kline.taker_buy_quote_asset_volume,
            ignore: event.kline.ignore
        }
    }

    // Getters
    pub fn get_type(&self) -> u64 {
        16
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
