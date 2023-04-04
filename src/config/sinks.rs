use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SinkConfigs {
    BinanceSpotSink(BinanceSpotSinkConfig),
    FileSink(FileSinkConfig),
    MockSink(MockSinkConfig)
}

impl SinkConfigs {
    pub fn name(&self) -> &str {
        match self {
            SinkConfigs::BinanceSpotSink(cfg) => &cfg.name,
            SinkConfigs::FileSink(cfg) => &cfg.name,
            SinkConfigs::MockSink(cfg) => &cfg.name
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BinanceSpotSinkConfig {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub secret_key: String,
    pub user_agent: String,
    pub content_type: String
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MockSinkConfig {
    pub name: String
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FileSinkConfig {
    pub name: String,
    pub filename: String
}
