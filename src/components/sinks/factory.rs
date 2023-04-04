use crate::components::sinks::{ Sinks, binance::spot::BinanceSpotSink, file::FileSink, mock::MockSink };
use crate::config::sinks::SinkConfigs;

use std::sync::Arc;
use tokio::sync::Mutex;

pub fn make(cfg: &SinkConfigs) -> Sinks {
    match cfg {
        SinkConfigs::BinanceSpotSink(cfg) => Sinks::BinanceSpotSink(Arc::new(Mutex::new(BinanceSpotSink::new(cfg)))),
        SinkConfigs::FileSink(cfg) => Sinks::FileSink(Arc::new(Mutex::new(FileSink::new(cfg)))),
        SinkConfigs::MockSink(cfg) => Sinks::MockSink(Arc::new(Mutex::new(MockSink::new(cfg)))),
        // _ => panic!("Cannot build sink {}", cfg.name())
    }
}
