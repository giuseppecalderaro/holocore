use crate::components::processing_units::common::update_orderbook;
use crate::components::processing_units::strategy;
use crate::config::pus::StrategyPUConfig;
use crate::objects::Objects;
use crate::objects::orderbook_update::OrderbookUpdateV1;
use crate::objects::market::Market;

use rocket::{ self, Build, Rocket };
use std::collections::HashMap;

pub struct StrategyPU {
    name: String,
    queue_size: usize,
    processed_objs: u64,
    realtime: bool,
    market: Market,
    obj_queues: HashMap<String, Vec<OrderbookUpdateV1>>,
    algorithm: strategy::Algorithms
}

impl StrategyPU {
    pub fn new(cfg: &StrategyPUConfig) -> Self {
        let algorithm = strategy::factory::make(&cfg.algorithm);

        Self {
            name: String::from(&cfg.name),
            queue_size: cfg.queue_size,
            processed_objs: 0,
            realtime: cfg.realtime,
            market: Market::new("Market"),
            obj_queues: HashMap::<String, Vec<OrderbookUpdateV1>>::new(),
            algorithm
        }
    }

    pub async fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    pub async fn execute(&mut self, mut obj: Objects) -> Result<(Option<Objects>, bool), String> {
        match obj {
            Objects::OrderbookUpdateV1(ref obj) => {
                update_orderbook(&mut self.market, obj, &self.name, self.realtime, &mut self.obj_queues).await;
            },
            Objects::OrderbookV1(ref mut new_book) => {
                log::info!("{}: got orderbook for {} - {}", self.name, &new_book.symbol, new_book.last_update_id);
                if let Some(queue) = self.obj_queues.get_mut(&new_book.symbol) {
                    for update in queue.iter() {
                        log::debug!("{}: spooling object {} ({}) - {} - {}", self.name, update.get_id(), update.symbol, update.first_update_id, update.final_update_id);
                        new_book.update(update)
                    }
                    queue.clear();
                    log::info!("{}: orderbook {} has been updated", self.name, &new_book.symbol);
                }

                self.market.clear_pending(&new_book.symbol);
                self.market.set_book(new_book.clone());
            },
            _ => ()
        };

        match self.algorithm.process(&obj, &self.market) {
            Ok(_obj) => (),
            Err(e) => log::error!("{}: failed to process obj {} - {}", "TrendFollowing", obj.get_id(), e)
        };

        self.processed_objs += 1;
        Ok((Some(obj), false))
    }

    // Getters
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_queue_size(&self) -> usize {
        self.queue_size
    }

    // Endpoints
    pub fn endpoints(&self, ctrl: Rocket<Build>) -> Rocket<Build> {
        ctrl
    }
}
