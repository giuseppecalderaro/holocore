use crate::components::sources::{ Sources, file::FileSource, mock::MockSource };
use crate::components::sources::binance::spot::BinanceSpotSource;
use crate::config::sources::SourceConfigs;

use std::sync::Arc;
use tokio::sync::Mutex;

pub fn make(cfg: &SourceConfigs) -> Sources {
    match cfg {
        SourceConfigs::BinanceSpotSource(cfg) => Sources::BinanceSpotSource(Arc::new(Mutex::new(BinanceSpotSource::new(cfg)))),
        SourceConfigs::FileSource(cfg) => Sources::FileSource(Arc::new(Mutex::new(FileSource::new(cfg)))),
        SourceConfigs::MockSource(cfg) => Sources::MockSource(Arc::new(Mutex::new(MockSource::new(cfg)))),
        // _ => panic!("Cannot build source {}", cfg.name())
    }
}
