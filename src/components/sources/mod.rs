pub mod factory;
pub mod binance;
pub mod file;
pub mod mock;

use std::sync::Arc;
use tokio::sync::Mutex;

pub enum Sources {
    BinanceSpotSource(Arc<Mutex<binance::spot::BinanceSpotSource>>),
    FileSource(Arc<Mutex<file::FileSource>>),
    MockSource(Arc<Mutex<mock::MockSource>>)
}
