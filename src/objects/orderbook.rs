use crate::objects::common::{ serialize_asks, deserialize_asks, serialize_bids, deserialize_bids };
use crate::objects::orderbook_update::OrderbookUpdateV1;
use crate::components::sources::binance::model::BinanceOrderBookEvent;

use fixed::FixedU64;
use serde::{ Deserialize, Serialize };
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum OrderbookStatus {
    Ticking,
    Crossed,
    Error,
    Stale
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderbookV1 {
    id: Uuid,
    timestamp: i64,
    sequence_nr: u64,
    correlation_id: Uuid,
    source: String,
    exchange: String,
    asset_type: String,

    pub symbol: String,
    pub first_update: bool,
    pub last_update_id: u64,

    pub status: OrderbookStatus,

    #[serde(serialize_with = "serialize_bids", deserialize_with = "deserialize_bids")]
    bids: BTreeMap<FixedU64<32>, FixedU64<32>>,

    #[serde(serialize_with = "serialize_asks", deserialize_with = "deserialize_asks")]
    asks: BTreeMap<FixedU64<32>, FixedU64<32>>
}

impl OrderbookV1 {
    pub fn from_binance(id: Uuid,
                        timestamp: i64,
                        sequence_nr: u64,
                        correlation_id: Uuid,
                        source: &str,
                        exchange: &str,
                        asset_type: &str,
                        symbol: &str,
                        event: BinanceOrderBookEvent) -> Self {
        let mut bids = BTreeMap::<FixedU64<32>, FixedU64<32>>::new();
        for bid in event.bids {
            let price = match FixedU64::<32>::from_str(&bid.price) {
                Ok(price) => price,
                Err(e) => {
                    log::error!("OrderbookV1: failed to parse price {} - {}", bid.price, e);
                    continue;
                }
            };
            let volume = match FixedU64::<32>::from_str(&bid.qty) {
                Ok(volume) => volume,
                Err(e) => {
                    log::error!("OrderbookV1: failed to parse volume {} - {}", bid.qty, e);
                    continue;
                }
            };

            bids.insert(price, volume);
        }

        let mut asks = BTreeMap::<FixedU64<32>, FixedU64<32>>::new();
        for ask in event.asks {
            let price = match FixedU64::<32>::from_str(&ask.price) {
                Ok(price) => price,
                Err(e) => {
                    log::error!("OrderbookV1: failed to parse price {} - {}", ask.price, e);
                    continue;
                }
            };
            let volume = match FixedU64::<32>::from_str(&ask.qty) {
                Ok(volume) => volume,
                Err(e) => {
                    log::error!("OrderbookV1: failed to parse volume {} - {}", ask.qty, e);
                    continue;
                }
            };

            asks.insert(price, volume);
        }

        Self {
            id,
            timestamp,
            sequence_nr,
            correlation_id,
            source: String::from(source),
            exchange: String::from(exchange),
            asset_type: String::from(asset_type),
            symbol: String::from(symbol),
            first_update: true,
            last_update_id: event.last_update_id,
            status: OrderbookStatus::Ticking,
            bids,
            asks
        }
    }

    // Getters
    pub fn get_type(&self) -> u64 {
        19
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

    pub fn get_correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    pub fn best_bid(&self) -> (FixedU64<32>, FixedU64<32>) {
        match self.bids.iter().next_back() {
            Some((price, volume)) => (*price, *volume),
            None => {
                log::error!("Orderbook {}: does not have a best bid level, returning zeroes", self.symbol);
                (FixedU64::<32>::from_num(0), FixedU64::<32>::from_num(0))
            }
        }
    }

    pub fn best_ask(&self) -> (FixedU64<32>, FixedU64<32>) {
        match self.asks.iter().next() {
            Some((price, volume)) => (*price, *volume),
            None => {
                log::error!("Orderbook {}: does not have a best ask level, returning zeroes", self.symbol);
                (FixedU64::<32>::from_num(0), FixedU64::<32>::from_num(0))
            }
        }
    }

    // Methods
    pub fn update(&mut self, obj: &OrderbookUpdateV1) {
        // Drop any event where u is <= lastUpdateId in the snapshot.
        if obj.final_update_id <= self.last_update_id {
            if self.last_update_id - obj.final_update_id > 20000 {
                log::error!("Snapshot orderbook {} ({}) has a wrong update id {} - {}", self.symbol, obj.symbol, self.last_update_id, obj.final_update_id);
                self.status = OrderbookStatus::Stale;
                return;
            }

            log::debug!("{}: skipping event {}: {} - {}", self.symbol, obj.get_id(), obj.final_update_id, self.last_update_id);
            return;
        }

        // The first processed event should have U <= lastUpdateId+1 AND u >= lastUpdateId+1.
        if self.first_update {
            if !(obj.first_update_id <= self.last_update_id + 1 && obj.final_update_id > self.last_update_id) {
                log::error!("{}: received wrong FIRST update, book might be incorrect {} - {} - {}", self.symbol, obj.first_update_id, self.last_update_id, obj.final_update_id);
                self.status = OrderbookStatus::Error;
                return;
            }
            self.first_update = false;
        } else if obj.first_update_id != self.last_update_id + 1 {
            log::error!("{}: received wrong update, book might be incorrect - {} - {}", self.symbol, obj.first_update_id, self.last_update_id);
            self.status = OrderbookStatus::Error;
            return;
        }

        for bid in &obj.bids {
            if bid.quantity == 0 {
                self.bids.remove(&bid.price);
            } else {
                self.bids.insert(bid.price, bid.quantity);
            }
        }

        for ask in &obj.asks {
            if ask.quantity == 0 {
                self.asks.remove(&ask.price);
            } else {
                self.asks.insert(ask.price, ask.quantity);
            }
        }

        self.status = OrderbookStatus::Ticking;
        let (bid_price, _bid_volume) = self.best_bid();
        let (ask_price, _ask_volume) = self.best_ask();
        if bid_price >= ask_price {
            log::error!("{}: is crossed {} >= {}", self.symbol, bid_price, ask_price);
            self.status = OrderbookStatus::Crossed;
        }

        self.last_update_id = obj.final_update_id;
        log::trace!("Updated orderbook {} to update_id {}", obj.symbol, self.last_update_id);
    }

    // Spread(s)
    pub fn spread(&self) -> f64 {
        let (bid_price, _bid_volume) = self.best_bid();
        let (ask_price, _ask_volume) = self.best_ask();
        (ask_price - bid_price).to_num()
    }

    pub fn weighted_spread(&self, requested_volume: f64) -> f64 {
        let mut average_bid = FixedU64::<32>::from_num(0);
        let mut average_ask = FixedU64::<32>::from_num(0);

        let mut running = FixedU64::<32>::from_num(requested_volume);
        for (price, volume) in self.bids.iter().rev() {
            if &running >= volume {
                // This level is used completely
                average_bid += price * volume;
                running -= volume;
                continue;
            }

            // We now run out of requested volume
            average_bid += price * running;
            break;
        }

        running = FixedU64::<32>::from_num(requested_volume);
        for (price, volume) in self.asks.iter() {
            if &running >= volume {
                // This level is used completely
                average_ask += price * volume;
                running -= volume;
                continue;
            }

            // We now run out of requested volume
            average_ask += price * running;
            break;
        }

        (average_ask - average_bid).to_num()
    }

    // Mid(s)
    pub fn mid(&self) -> FixedU64<32> {
        let (bid_price, _bid_volume) = self.best_bid();
        let (ask_price, _ask_volume) = self.best_ask();
        (ask_price + bid_price) / 2
    }

    pub fn weighted_mid(&self) -> FixedU64<32> {
        let (bid_price, _bid_volume) = self.best_bid();
        let (ask_price, _ask_volume) = self.best_ask();
        let imbalance = self.imbalance();
        let weighted_mid = imbalance * ask_price + (FixedU64::<32>::from_num(1) - imbalance) * bid_price;
        weighted_mid
    }

    // Microprice
    pub fn microprice(&self) -> FixedU64<32> {
        let mid = self.mid();
        let imbalance = self.imbalance();
        let microprice = mid + (imbalance * 1);

        microprice
    }

    // Imbalance
    fn imbalance(&self) -> FixedU64<32> {
        let (_bid_price, bid_volume) = self.best_bid();
        let (_ask_price, ask_volume) = self.best_ask();
        bid_volume / (bid_volume + ask_volume)
    }
}
