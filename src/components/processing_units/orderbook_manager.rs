use crate::components::processing_units::common::update_orderbook;
use crate::config::pus::OrderbookManagerPUConfig;
use crate::objects::{Objects, market::Market, orderbook_update::OrderbookUpdateV1};

use rocket::{ self, Build, Rocket, http, State };
use std::{sync::Arc, collections::HashMap};
use tokio::sync::RwLock;

pub struct OrderbookManagerPU {
    name: String,
    queue_size: usize,
    processed_objs: u64,
    realtime: bool,
    market: Arc<RwLock<Market>>,
    obj_queues: HashMap<String, Vec<OrderbookUpdateV1>>
}

impl OrderbookManagerPU {
    pub fn new(cfg: &OrderbookManagerPUConfig) -> Self {
        Self {
            name: String::from(&cfg.name),
            queue_size: cfg.queue_size,
            processed_objs: 0,
            realtime: cfg.realtime,
            market: Arc::new(RwLock::new(Market::new("Market"))),
            obj_queues: HashMap::<String, Vec<OrderbookUpdateV1>>::new()
        }
    }

    pub async fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    pub async fn execute(&mut self, mut obj: Objects) -> Result<(Option<Objects>, bool), String> {
        match obj {
            Objects::OrderbookUpdateV1(ref obj) => {
                let mut market = self.market.write().await;
                update_orderbook(&mut market, obj, &self.name, self.realtime, &mut self.obj_queues).await;
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

                let mut market = self.market.write().await;
                market.clear_pending(&new_book.symbol);
                market.set_book(new_book.clone());
            },
            _ => ()
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
        ctrl.manage(self.market.clone())
            .mount(format!("/{}/", self.name), rocket::routes!(get_orderbook))
            .mount(format!("/{}/", self.name), rocket::routes!(delete_orderbook))
    }
}

#[rocket::get("/orderbook/<symbol>")]
async fn get_orderbook(symbol: &str, state: &State<Arc<RwLock<Market>>>) -> (http::Status, String) {
    log::info!("Retrieving orderbook for {}", symbol);

    let market = state.read().await;
    match market.get_book(symbol) {
        Some(book) => {
            match serde_json::to_string(book) {
                Ok(wire) => (http::Status::Ok, wire),
                Err(e) => {
                    log::error!("Error while serializing orderbook for {} - {}", symbol, e);
                    (http::Status::InternalServerError, String::from("{}"))
                }
            }
        },
        None => {
            log::error!("Failed to retrieve orderbook for {}", symbol);
            (http::Status::BadRequest, String::from("{}"))
        }
    }
}

#[rocket::delete("/orderbook/<symbol>")]
async fn delete_orderbook(symbol: &str, state: &State<Arc<RwLock<Market>>>) -> (http::Status, String) {
    log::info!("Deleting orderbook for {}", symbol);

    let mut market = state.write().await;
    market.clear_book(symbol);
    (http::Status::Ok, String::from("Ok"))
}
