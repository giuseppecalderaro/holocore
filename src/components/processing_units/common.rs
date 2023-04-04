use crate::objects::{market::Market, orderbook_update::OrderbookUpdateV1, orderbook::OrderbookStatus};

use reqwest::Client;
use std::collections::HashMap;

pub async fn update_orderbook(market: &mut Market, obj: &OrderbookUpdateV1, name: &str, realtime: bool, obj_queues: &mut HashMap<String, Vec<OrderbookUpdateV1>>) {
    match market.get_mut_book(&obj.symbol) {
        Some(book) => {
            log::trace!("{}: Updating orderbook {}", name, &obj.symbol);
            book.update(obj);
            if book.status == OrderbookStatus::Stale {
                log::error!("Orderbook {} is stale, clearing it", &obj.symbol);
                market.clear_book(&obj.symbol);
            }
        },
        None => {
            if realtime && !market.is_pending(&obj.symbol) {
                log::info!("{}: retrieving orderbook for symbol {}", name, &obj.symbol);
                let complete_url = format!("http://127.0.0.1:8000/{}/orderbook/{}", &obj.source, &obj.symbol);
                let client = Client::new();
                match client.get(complete_url).send().await {
                    Ok(_) => (),
                    Err(e) => log::error!("{}: failed to retrieve orderbook - {}", name, e)
                };

                market.set_pending(&obj.symbol);
            }

            log::debug!("{}: queuing object {} ({}) - {} - {}", name, obj.get_id(), obj.symbol, obj.first_update_id, obj.final_update_id);
            match obj_queues.get_mut(&obj.symbol) {
                Some(queue) => queue.push(obj.clone()),
                None => {
                    let queue = vec![ obj.clone() ];
                    obj_queues.insert(String::from(&obj.symbol), queue);
                }
            };
        }
    };
}
