use crate::config::Config;
use crate::objects::Objects;

use flume::Sender;
use rocket::{ self, Build, Ignite, Rocket, http, serde::json::Json };
use serde::{ Deserialize, Serialize };
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

#[macro_export]
macro_rules! inject {
    ($unlocked_component:ident, $txs:ident, $obj:ident, $disabled:ident) => {
        let mut futures = $txs.iter()
                              .enumerate()
                              .filter(|&(idx, _)| !$disabled.contains(&idx))
                              .map(|(_, tx)| tx.send_async($obj.clone()))
                              .collect::<Vec<_>>();

        while !futures.is_empty() {
            match future::select_all(futures).await {
                (Ok(_value), _index, remaining) => futures = remaining,
                (Err(e), index, remaining) => {
                    log::error!("{}: INJECT failed while sending at index {} - {}", $unlocked_component.get_name(), index, e);
                    futures = remaining;
                    $disabled.push(index);
                    if $disabled.len() == $txs.len() {
                        log::error!("{}: all the txs are disabled. Returning...", $unlocked_component.get_name());
                        return;
                    }
                }
            }
        }
    };
}

pub struct ControlState {
    cfg: Config,
    txs: HashMap<String, Sender<Controls>>
}

impl ControlState {
    pub fn new(cfg: Config, txs: HashMap<String, Sender<Controls>>) -> Self {
        Self {
            cfg,
            txs
        }
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    pub fn get(&self, key: &str) -> Option<&Sender<Controls>> {
        self.txs.get(key)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum Controls {
    Inject(Objects),
    SleepTime(u64)
}

#[rocket::get("/health")]
async fn health() -> (http::Status, String) {
    (http::Status::Ok, String::from("OK"))
}

#[derive(Deserialize)]
struct ShutdownRequest<'r> {
    magic_code: &'r str
}

#[rocket::post("/shutdown", data="<request>")]
async fn shutdown(request: Json<ShutdownRequest<'_>>, shutdown: rocket::Shutdown) -> http::Status {
    if request.magic_code == "0xDEADBEEF" {
        shutdown.notify();
        return http::Status::Ok;
    }

    http::Status::BadRequest
}

pub fn build_control(ctrl_state: ControlState) -> Rocket<Build> {
    let mut control_config = rocket::config::Config::default();
    match IpAddr::from_str(&ctrl_state.cfg.ctrl_host) {
        Ok(ip) => control_config.address = ip,
        Err(e) => log::error!("Cannot parse ip address {} - {}", &ctrl_state.cfg.ctrl_host, e)
    };
    control_config.port = ctrl_state.cfg.ctrl_port;

    rocket::custom(control_config)
        .manage(ctrl_state)
        .mount("/Processor", rocket::routes![health])
        .mount("/Processor", rocket::routes![shutdown])
}

pub async fn start_control(ctrl: Rocket<Build>) -> Rocket<Ignite> {
    ctrl.launch().await.expect("Failed to launch control component")
}
