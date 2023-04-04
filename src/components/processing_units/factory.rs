use crate::components::processing_units::{ PUs,
                                           mock::MockPU,
                                           order_manager::OrderManagerPU,
                                           orderbook_manager::OrderbookManagerPU,
                                           strategy::base::StrategyPU
                                         };
use crate::config::pus::PUConfigs;

use std::sync::Arc;
use tokio::sync::Mutex;

pub fn make(cfg: &PUConfigs) -> PUs {
    match cfg {
        PUConfigs::MockPU(cfg) => PUs::MockPU(Arc::new(Mutex::new(MockPU::new(cfg)))),
        PUConfigs::OrderManagerPU(cfg) => PUs::OrderManagerPU(Arc::new(Mutex::new(OrderManagerPU::new(cfg)))),
        PUConfigs::OrderbookManagerPU(cfg) => PUs::OrderbookManagerPU(Arc::new(Mutex::new(OrderbookManagerPU::new(cfg)))),
        PUConfigs::StrategyPU(cfg) => PUs::StrategyPU(Arc::new(Mutex::new(StrategyPU::new(cfg)))),
        _ => panic!("Cannot build PU {}", cfg.name())
    }
}
