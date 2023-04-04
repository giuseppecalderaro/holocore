use crate::config::pus::MockPUConfig;
use crate::objects::Objects;

use rocket::{ self, Build, Rocket, http };

pub struct MockPU {
    name: String,
    queue_size: usize,
    processed_objs: u64
}

impl MockPU {
    pub fn new(cfg: &MockPUConfig) -> Self {
        Self {
            name: String::from(&cfg.name),
            queue_size: cfg.queue_size,
            processed_objs: 0
        }
    }

    pub async fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    pub async fn execute(&mut self, obj: Objects) -> Result<(Option<Objects>, bool), String> {
        self.processed_objs += 1;
        log::info!("{} - {}: Processing message {} - {} - {} - {}",
                   self.name,
                   self.processed_objs,
                   obj.get_id(),
                   obj.get_timestamp(),
                   obj.get_sequence_number(),
                   obj.get_correlation_id());

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
        ctrl.mount(format!("/{}/", self.name), rocket::routes!(test))
    }
}

#[rocket::get("/test")]
fn test() -> (http::Status, String) {
    (http::Status::Ok, String::from("OK"))
}
