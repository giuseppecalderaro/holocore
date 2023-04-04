use crate::components::control::{ControlState, Controls};
use crate::config::sources::MockSourceConfig;
use crate::objects::{ Objects, mock::MockV1 };
use crate::utils::{ http::get_name_from_uri, time, uuid };

use rocket::{ self, Build, Rocket, http, State, serde::json::Json };
use serde::Deserialize;
use tokio::time::sleep;

pub struct MockSource {
    name: String,
    queue_size: usize,
    sequence_number: u64,
    sleep_time: u64,
}

impl MockSource {
    pub fn new(cfg: &MockSourceConfig) -> Self {
        Self {
            name: String::from(&cfg.name),
            queue_size: cfg.queue_size,
            sequence_number: 0,
            sleep_time: cfg.sleep_time
        }
    }

    pub async fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<(Option<Objects>, bool), String> {
        if self.sleep_time != 0 {
            sleep(std::time::Duration::from_millis(self.sleep_time)).await;
        }

        self.sequence_number += 1;
        let obj = Objects::MockV1(MockV1::new(uuid::uuid(), time::now_nanos(), self.sequence_number, uuid::uuid()));
        log::info!("{}: Received {} messages", self.name, self.sequence_number);
        Ok((Some(obj), false))
    }

    // Getters
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_queue_size(&self) -> usize {
        self.queue_size
    }

    // Setters
    pub fn set_sleep_time(&mut self, sleep_time: u64) {
        self.sleep_time = sleep_time;
    }

    // Endpoints
    pub fn endpoints(&self, ctrl: Rocket<Build>) -> Rocket<Build> {
        ctrl.mount(format!("/{}/", self.name), rocket::routes!(test))
            .mount(format!("/{}/", self.name), rocket::routes!(inject))
            .mount(format!("/{}/", self.name), rocket::routes!(sleep_time))
    }
}

#[rocket::get("/test")]
fn test() -> (http::Status, String) {
    (http::Status::Ok, String::from("OK"))
}

#[rocket::post("/inject")]
async fn inject(uri: &http::uri::Origin<'_>, state: &State<ControlState>) -> http::Status {
    let name = get_name_from_uri(uri);

    match state.get(&name) {
        Some(tx) => {
            let obj = Objects::MockV1(MockV1::new(uuid::uuid(), time::now_nanos(), 0, uuid::uuid()));
            match tx.send_async(Controls::Inject(obj)).await {
                Ok(()) => return http::Status::Ok,
                Err(e) => log::error!("Cannot inject - {}", e)
            }
        },
        None => return http::Status::BadRequest
    }

    http::Status::Ok
}

#[derive(Deserialize)]
struct SleepTimeRequest {
    sleep_time: u64
}

#[rocket::post("/sleep_time", data="<request>")]
async fn sleep_time(uri: &http::uri::Origin<'_>, request: Json<SleepTimeRequest>, state: &State<ControlState>) -> http::Status {
    let name = get_name_from_uri(uri);

    if request.sleep_time > 0 {
        match state.get(&name) {
            Some(tx) => {
                match tx.send_async(Controls::SleepTime(request.sleep_time)).await {
                    Ok(()) => return http::Status::Ok,
                    Err(e) => log::error!("Cannot set sleep_time for {} - {}", name, e)
                }
            },
            None => return http::Status::BadRequest
        }
    }

    http::Status::BadRequest
}
