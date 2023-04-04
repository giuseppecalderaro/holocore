pub mod sources;
pub mod pus;
pub mod sinks;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: u64,
    pub log_level: String,
    pub workers: usize,
    pub stack_size: usize,
    pub ctrl_host: String,
    pub ctrl_port: u16,
    pub discovery_service: Option<String>,
    pub gateway: Option<String>,
    pub sources: Vec<sources::SourceConfigs>,
    pub pus: Option<Vec<pus::PUConfigs>>,
    pub sinks: Vec<sinks::SinkConfigs>,
}
