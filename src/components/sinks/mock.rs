use crate::config::sinks::MockSinkConfig;
use crate::objects::Objects;
use crate::utils::time::now_nanos;

use rocket::{ self, Build, Rocket, http };

pub struct MockSink {
    name: String,
    sent_objs: u64
}

impl MockSink {
    pub fn new(cfg: &MockSinkConfig) -> Self {
        Self {
            name: String::from(&cfg.name),
            sent_objs: 0
        }
    }

    pub async fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    pub async fn send(&mut self, obj: Objects) -> Result<(), String> {
        self.sent_objs += 1;
        let now = now_nanos();
        log::debug!("{}: Received {} messages - msg id: {} - lag {} micros", self.name, self.sent_objs, obj.get_id(), (now - obj.get_timestamp()) / 1_000);

        if let Objects::OrderbookUpdateV1(obj) = obj {
            let now_ms: u64 = now as u64 / 1_000;
            let timestamp: u64 = obj.get_timestamp() as u64 / 1_000;
            log::debug!("{}: lag from event {} - {}  = {} micros", self.name, now_ms, timestamp, now_ms - timestamp);
        }

        Ok(())
    }

    // Getters
    pub fn get_name(&self) -> &str {
        &self.name
    }

    // Endpoints
    pub fn endpoints(&self, ctrl: Rocket<Build>) -> Rocket<Build> {
        ctrl.mount(format!("/{}/", self.name), rocket::routes!(test))
    }
}

#[rocket::get("/test")]
fn test() -> (http::Status, String) {
    (http::Status::Ok, String::from("OK"))
}
