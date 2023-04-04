pub mod factory;
pub mod binance;
pub mod file;
pub mod mock;

use std::sync::Arc;
use tokio::sync::Mutex;

pub enum Sinks {
    BinanceSpotSink(Arc<Mutex<binance::spot::BinanceSpotSink>>),
    FileSink(Arc<Mutex<file::FileSink>>),
    MockSink(Arc<Mutex<mock::MockSink>>)
}
