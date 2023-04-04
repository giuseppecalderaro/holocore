use crate::components::sinks::binance::api::{ API, Spot };
use crate::components::sinks::binance::client::BinanceSpotClient;
use crate::components::sinks::binance::model::{ BinanceAccountInformation,
                                                BinanceAckOrder,
                                                BinanceBalance,
                                                BinanceNewOrder,
                                                BinanceOrder,
                                                BinanceOrderCanceled };
use crate::config::sinks::BinanceSpotSinkConfig;
use crate::objects::{ Objects, new_order::NewOrderV1 };

use rocket::{ self, http, Build, Rocket, State };
use rocket::serde::json::Json;
use std::collections::BTreeMap;

pub struct BinanceSpotSink {
    name: String,
    sent_objs: u64,
    client: BinanceSpotClient
}

impl BinanceSpotSink {
    pub fn new(cfg: &BinanceSpotSinkConfig) -> Self {
        Self {
            name: String::from(&cfg.name),
            sent_objs: 0,
            client: BinanceSpotClient::new(&cfg.base_url, &cfg.api_key, &cfg.secret_key, &cfg.user_agent, &cfg.content_type)
        }
    }

    pub async fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    pub async fn send(&mut self, _obj: Objects) -> Result<(), String> {
        self.sent_objs += 1;

        // match obj {
        //     _ => self.new_order("BTCUSDT", OrderSide::Buy, OrderType::Market, FixedU64::<32>::from_num(1), FixedU64::<32>::from_num(1)).await
        // }

        Ok(())
    }

    // Getters
    pub fn get_name(&self) -> &str {
        &self.name
    }

    // Endpoints
    pub fn endpoints(&self, ctrl: Rocket<Build>) -> Rocket<Build> {
        ctrl.manage(self.client.clone())
            .mount(format!("/{}/", self.name), rocket::routes!(get_account))
            .mount(format!("/{}/", self.name), rocket::routes!(get_balance))
            .mount(format!("/{}/", self.name), rocket::routes!(get_open_orders))
            .mount(format!("/{}/", self.name), rocket::routes!(post_new_order))
            .mount(format!("/{}/", self.name), rocket::routes!(get_orders))
            .mount(format!("/{}/", self.name), rocket::routes!(delete_cancel_order))
    }
}

// Helpers
async fn account(client: &BinanceSpotClient) -> Result<BinanceAccountInformation, String> {
    let params = BTreeMap::<String, String>::new();
    let request = BinanceSpotClient::build_request(params, None);
    client.get_signed(API::Spot(Spot::Account), request).await
}

async fn balance(client: &BinanceSpotClient, symbol: &str) -> Result<BinanceBalance, String> {
    match account(client).await {
        Ok(account) => {
            for balance in account.balances {
                if balance.asset == symbol {
                    return Ok(balance);
                }
            }

            Err(format!("Cannot find balance for asset {}", symbol))
        },
        Err(e) => Err(e)
    }
}

async fn list_orders(client: &BinanceSpotClient, symbol: &str) -> Result<Vec<BinanceOrder>, String>
{
    let mut params = BTreeMap::<String, String>::new();
    params.insert("symbol".into(), symbol.into());

    let request = BinanceSpotClient::build_request(params, None);
    client.get_signed::<Vec<BinanceOrder>>(API::Spot(Spot::AllOrders), request).await
}

async fn list_open_orders(client: &BinanceSpotClient, symbol: &str) -> Result<Vec<BinanceOrder>, String>
{
    let mut params = BTreeMap::<String, String>::new();
    params.insert("symbol".into(), symbol.into());

    let request = BinanceSpotClient::build_request(params, None);
    client.get_signed::<Vec<BinanceOrder>>(API::Spot(Spot::OpenOrders), request).await
}

async fn new_order(client: &BinanceSpotClient, new_order: BinanceNewOrder) -> Result<BinanceAckOrder, String> {
    let mut params = BTreeMap::<String, String>::new();

    params.insert(String::from("symbol"), new_order.symbol);
    params.insert(String::from("side"), String::from(new_order.order_side));
    params.insert(String::from("type"), String::from(new_order.order_type));
    params.insert(String::from("quantity"), new_order.quantity.to_string());
    params.insert(String::from("price"), new_order.price.to_string());

    let request = BinanceSpotClient::build_request(params, None);
    client.post_signed::<BinanceAckOrder>(API::Spot(Spot::Order), request).await
}

async fn cancel_order(client: &BinanceSpotClient, symbol: &str, order_id: i64) -> Result<BinanceOrderCanceled, String> {
    let mut params = BTreeMap::<String, String>::new();
    params.insert(String::from("symbol"), symbol.to_string());
    params.insert(String::from("orderId"), order_id.to_string());

    let request = BinanceSpotClient::build_request(params, None);
    client.delete_signed::<BinanceOrderCanceled>(API::Spot(Spot::Order), request).await
}

#[rocket::get("/account")]
async fn get_account(state: &State<BinanceSpotClient>) -> (http::Status, String) {
    match account(state).await {
        Ok(account) => {
            match serde_json::to_string(&account) {
                Ok(text) => (http::Status::Ok, text),
                Err(e) => (http::Status::BadRequest, e.to_string())
            }
        },
        Err(e) => (http::Status::BadRequest, e)
    }
}

#[rocket::get("/balance/<symbol>")]
async fn get_balance(symbol: &str, state: &State<BinanceSpotClient>) -> (http::Status, String) {
    match balance(state, symbol).await {
        Ok(balance) => {
            match serde_json::to_string(&balance) {
                Ok(text) => (http::Status::Ok, text),
                Err(e) => (http::Status::BadRequest, e.to_string())
            }
        },
        Err(e) => (http::Status::BadRequest, e)
    }
}

#[rocket::get("/open_orders/<symbol>")]
async fn get_open_orders(symbol: &str, state: &State<BinanceSpotClient>) -> (http::Status, String) {
    match list_open_orders(state, symbol).await {
        Ok(orders) => {
            match serde_json::to_string(&orders) {
                Ok(text) => (http::Status::Ok, text),
                Err(e) => (http::Status::BadRequest, e.to_string())
            }
        },
        Err(e) => (http::Status::BadRequest, e)
    }
}

#[rocket::post("/orders", format = "json", data = "<order>")]
async fn post_new_order(order: Json<NewOrderV1>, state: &State<BinanceSpotClient>) -> (http::Status, String) {
    match new_order(state, BinanceNewOrder::from(order.into_inner())).await {
        Ok(ack) => {
            match serde_json::to_string(&ack) {
                Ok(text) => (http::Status::Ok, text),
                Err(e) => (http::Status::BadRequest, e.to_string())
            }
        },
        Err(e) => (http::Status::BadRequest, e)
    }
}

#[rocket::get("/open/<symbol>")]
async fn get_orders(symbol: &str, state: &State<BinanceSpotClient>) -> (http::Status, String) {
    match list_orders(state, symbol).await {
        Ok(orders) => {
            match serde_json::to_string(&orders) {
                Ok(text) => (http::Status::Ok, text),
                Err(e) => (http::Status::BadRequest, e.to_string())
            }
        },
        Err(e) => (http::Status::BadRequest, e)
    }
}

#[rocket::delete("/orders/<symbol>/<order_id>")]
async fn delete_cancel_order(symbol: &str, order_id: i64, state: &State<BinanceSpotClient>) -> (http::Status, String) {
    match cancel_order(state, symbol, order_id).await {
        Ok(ack) => {
            match serde_json::to_string(&ack) {
                Ok(text) => (http::Status::Ok, text),
                Err(e) => (http::Status::BadRequest, e.to_string())
            }
        },
        Err(e) => (http::Status::BadRequest, e)
    }
}
