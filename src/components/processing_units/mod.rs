pub mod common;
pub mod factory;
pub mod mock;
pub mod order_manager;
pub mod orderbook_manager;
pub mod strategy;

use std::sync::Arc;
use tokio::sync::Mutex;

pub enum PUs {
    MockPU(Arc<Mutex<mock::MockPU>>),
    OrderManagerPU(Arc<Mutex<order_manager::OrderManagerPU>>),
    OrderbookManagerPU(Arc<Mutex<orderbook_manager::OrderbookManagerPU>>),
    StrategyPU(Arc<Mutex<strategy::base::StrategyPU>>)
}
