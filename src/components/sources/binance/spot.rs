use crate::components::control::{ ControlState, Controls };
use crate::components::sources::binance::model::{ BinanceKlineEvent, BinanceOrderBookEvent, BinanceOrderbookUpdateEvent, BinanceTradeEvent };
use crate::config::sources::{BinanceSpotSourceConfig, SourceConfigs};
use crate::objects::{ Objects, kline::KlineV1, orderbook::OrderbookV1, orderbook_update::OrderbookUpdateV1, trade::TradeV1 };
use crate::utils::http::get_name_from_uri;
use crate::utils::{ time, uuid };

use futures::StreamExt;
use reqwest;
use rocket::{ self, http, Build, Rocket, State };
use std::sync::Arc;
use std::sync::atomic::{ AtomicBool, AtomicI64, Ordering };
use tokio::net::TcpStream;
use tokio::time::{ Duration, timeout };
use tokio_tungstenite::{ MaybeTlsStream, WebSocketStream, connect_async };
use url::Url;

pub struct BinanceSpotSource {
    name: String,
    queue_size: usize,
    sequence_number: u64,
    active: Arc<AtomicBool>,
    last_timestamp: Arc<AtomicI64>,
    wakeup_interval: u64,
    base_url: String,
    symbols: Vec<String>,
    socket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>
}

impl BinanceSpotSource {
    pub fn new(cfg: &BinanceSpotSourceConfig) -> Self {
        let mut symbols = Vec::<String>::new();
        for sym in &cfg.symbols {
            symbols.push(String::from(sym));
        }

        Self {
            name: String::from(&cfg.name),
            queue_size: cfg.queue_size,
            sequence_number: 0,
            active: Arc::new(AtomicBool::new(false)),
            last_timestamp: Arc::<AtomicI64>::new(AtomicI64::new(0)),
            wakeup_interval: cfg.wakeup_interval,
            base_url: String::from(&cfg.base_url),
            symbols,
            socket: None
        }
    }

    pub async fn init(&mut self) -> Result<(), String> {
        let mut complete_url = String::from(&self.base_url);
        for sym in &self.symbols {
            complete_url += sym;
        }

        log::info!("{}: connecting to {}", self.name, complete_url);
        let url: Url = match Url::parse(&complete_url) {
            Ok(url) => url,
            Err(e) => return Err(format!("{}: cannot parse url {} - {}", self.name, complete_url, e))
        };

        match connect_async(url).await {
            Ok((ws, _response)) => {
                log::info!("{}: Websocket connected", self.name);
                self.socket = Some(ws);
                self.active.store(true, Ordering::Release);
                Ok(())
            },
            Err(e) => {
                let err_msg = format!("{} Error during handshake - {}", self.name, e);
                log::error!("{}", &err_msg);
                Err(err_msg)
            }
        }
    }

    pub async fn recv(&mut self) -> Result<(Option<Objects>, bool), String> {
        if !self.active.load(Ordering::Acquire) {
            return Ok((None, true));
        }

        match &mut self.socket {
            Some(socket) => {
                match timeout(Duration::from_millis(self.wakeup_interval), socket.next()).await {
                    Ok(data) => {
                        match data {
                            Some(data) => match data {
                                Ok(data) => {
                                    let wire_timestamp = time::now_nanos();
                                    self.last_timestamp.store(wire_timestamp, Ordering::Release);

                                    match data.into_text() {
                                        Ok(msg) => {
                                            match serde_json::from_str::<serde_json::Value>(&msg) {
                                                Ok(event) => {
                                                    self.sequence_number += 1;
                                                    match event["data"]["e"].as_str() {
                                                        Some("depthUpdate") => {
                                                            match serde_json::from_value::<BinanceOrderbookUpdateEvent>(event["data"].to_owned()) {
                                                                Ok(binance_event) => {
                                                                    let obj = OrderbookUpdateV1::from_binance(uuid::uuid(),
                                                                                                              wire_timestamp,
                                                                                                              self.sequence_number,
                                                                                                              uuid::uuid(),
                                                                                                              &self.name,
                                                                                                              "Binance",
                                                                                                              "Spot",
                                                                                                              binance_event);
                                                                    Ok((Some(Objects::OrderbookUpdateV1(obj)), false))
                                                                },
                                                                Err(e) => Err(format!("{}: cannot decode orderbook update - {}", self.name, e))
                                                            }
                                                        },
                                                        Some("kline") => {
                                                            match serde_json::from_value::<BinanceKlineEvent>(event["data"].to_owned()) {
                                                                Ok(binance_event) => {
                                                                    let obj = KlineV1::from_binance(uuid::uuid(),
                                                                                                    wire_timestamp,
                                                                                                    self.sequence_number,
                                                                                                    uuid::uuid(),
                                                                                                    &self.name,
                                                                                                    "Binance",
                                                                                                    "Spot",
                                                                                                    binance_event);
                                                                    Ok((Some(Objects::KlineV1(obj)), false))
                                                                },
                                                                Err(e) => Err(format!("{}: cannot decode kline - {}", self.name, e))
                                                            }
                                                        },
                                                        Some("trade") => {
                                                            match serde_json::from_value::<BinanceTradeEvent>(event["data"].to_owned()) {
                                                                Ok(binance_event) => {
                                                                    let obj = TradeV1::from_binance(uuid::uuid(),
                                                                                                    wire_timestamp,
                                                                                                    self.sequence_number,
                                                                                                    uuid::uuid(),
                                                                                                    &self.name,
                                                                                                    "Binance",
                                                                                                    "Spot",
                                                                                                    binance_event);
                                                                    Ok((Some(Objects::TradeV1(obj)), false))
                                                                },
                                                                Err(e) => Err(format!("{}: cannot decode trade - {}", self.name, e))
                                                            }
                                                        },
                                                        _ => {
                                                            Ok((None, false))
                                                        }
                                                    }
                                                },
                                                Err(e) => Err(format!("{}: cannot decode json - {}", self.name, e))
                                            }
                                        },
                                        Err(e) => Err(format!("{}: cannot decode data as text - {}", self.name, e))
                                    }
                                },
                                Err(e) => Err(format!("{}: websocket has returned an error - {}", self.name, e))
                            },
                            None => Err(format!("{}: websocket has returned no data", self.name))
                        }
                    },
                    Err(e) => {
                        self.active.store(false, Ordering::Release);
                        let err_msg = format!("{}: websocket has not received any data in the last {} ms - {}", self.name, self.wakeup_interval, e);
                        log::error!("{}", &err_msg);
                        Err(err_msg)
                    }
                }
            },
            None => {
                let err_msg = format!("{}: Socket is uninitialized", self.name);
                log::error!("{}", &err_msg);
                Err(err_msg)
            }
        }
    }

    // Getters
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_queue_size(&self) -> usize {
        self.queue_size
}

    // Setters
    pub fn set_sleep_time(&mut self, _sleep_time: u64) {}

    // Endpoints
    pub fn endpoints(&self, ctrl: Rocket<Build>) -> Rocket<Build> {
        ctrl.mount(format!("/{}/", self.name), rocket::routes!(orderbook))
    }
}

#[rocket::get("/orderbook/<symbol>")]
async fn orderbook(symbol: &str, uri: &http::uri::Origin<'_>, state: &State<ControlState>) -> http::Status {
    let name = get_name_from_uri(uri);
    let snapshot_url = match state.config().sources.iter().find(|src| src.name() == name) {
        Some(cfg) => {
            if let SourceConfigs::BinanceSpotSource(cfg) = cfg {
                &cfg.snapshot_url
            } else {
                panic!("BinanceSource has got a wrong config format");
            }
        },
        None => {
            log::error!("{}: cannot find source config", name);
            return http::Status::InternalServerError;
        }
    };

    let complete_url = format!("{}{}&limit=1000", snapshot_url, symbol);

    let client = reqwest::Client::new();
    match client.get(complete_url).send().await {
        Ok(res) => match res.status() {
            reqwest::StatusCode::OK => {
                match res.text().await {
                    Ok(data) => {
                        match serde_json::from_str::<BinanceOrderBookEvent>(&data) {
                            Ok(binance_event) => {
                                let orderbook = OrderbookV1::from_binance(uuid::uuid(),
                                                                          time::now_nanos(),
                                                                          0,
                                                                          uuid::uuid(),
                                                                          &name,
                                                                          "Binance",
                                                                          "Spot",
                                                                          symbol,
                                                                          binance_event);

                                match state.get(&name) {
                                    Some(tx) => {
                                        match tx.send_async(Controls::Inject(Objects::OrderbookV1(orderbook))).await {
                                            Ok(()) => http::Status::Ok,
                                            Err(e) => {
                                                log::error!("Cannot send {} - {}", name, e);
                                                http::Status::BadRequest
                                            }
                                        }
                                    },
                                    None => http::Status::BadRequest
                                }
                            },
                            Err(e) => {
                                log::error!("Failed parsing orderbook for symbol {} - {}", symbol, e);
                                http::Status::InternalServerError
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("Failed receiving orderbook for symbol {} - {}", symbol, e);
                        http::Status::BadRequest
                    }
                }
            },
            _ => {
                log::error!("Received error while retrieving orderbook for symbol {}", symbol);
                http::Status::BadRequest
            }
        },
        Err(_e) => {
            log::error!("Failed requesting orderbook for symbol {}", symbol);
            http::Status::BadRequest
        }
    }
}
