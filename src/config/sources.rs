use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SourceConfigs {
    BinanceSpotSource(BinanceSpotSourceConfig),
    FileSource(FileSourceConfig),
    MockSource(MockSourceConfig)
}

impl SourceConfigs {
    pub fn name(&self) -> &str {
        match self {
            SourceConfigs::BinanceSpotSource(cfg) => &cfg.name,
            SourceConfigs::FileSource(cfg) => &cfg.name,
            SourceConfigs::MockSource(cfg) => &cfg.name
        }
    }

    pub fn queue_size(&self) -> usize {
        match self {
            SourceConfigs::BinanceSpotSource(cfg) => cfg.queue_size,
            SourceConfigs::FileSource(cfg) => cfg.queue_size,
            SourceConfigs::MockSource(cfg) => cfg.queue_size
        }
    }

    pub fn downstreams(&self) -> &Vec<String> {
        match self {
            SourceConfigs::BinanceSpotSource(cfg) => &cfg.downstreams,
            SourceConfigs::FileSource(cfg) => &cfg.downstreams,
            SourceConfigs::MockSource(cfg) => &cfg.downstreams
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BinanceSpotSourceConfig {
    pub name: String,
    pub queue_size: usize,
    pub downstreams: Vec<String>,
    pub base_url: String,
    pub snapshot_url: String,
    pub symbols: Vec<String>,
    pub wakeup_interval: u64
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FileSourceConfig {
    pub name: String,
    pub queue_size: usize,
    pub downstreams: Vec<String>,
    pub filename: String
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MockSourceConfig {
    pub name: String,
    pub queue_size: usize,
    pub downstreams: Vec<String>,
    pub sleep_time: u64
}
