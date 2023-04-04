use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PUConfigs {
    EnginePU(EnginePUConfig),
    MockPU(MockPUConfig),
    OrderManagerPU(OrderManagerPUConfig),
    OrderbookManagerPU(OrderbookManagerPUConfig),
    StrategyPU(StrategyPUConfig)
}

impl PUConfigs {
    pub fn name(&self) -> &str {
        match self {
            PUConfigs::EnginePU(cfg) => &cfg.name,
            PUConfigs::MockPU(cfg) => &cfg.name,
            PUConfigs::OrderManagerPU(cfg) => &cfg.name,
            PUConfigs::OrderbookManagerPU(cfg) => &cfg.name,
            PUConfigs::StrategyPU(cfg) => &cfg.name
        }
    }

    pub fn queue_size(&self) -> usize {
        match self {
            PUConfigs::EnginePU(cfg) => cfg.queue_size,
            PUConfigs::MockPU(cfg) => cfg.queue_size,
            PUConfigs::OrderManagerPU(cfg) => cfg.queue_size,
            PUConfigs::OrderbookManagerPU(cfg) => cfg.queue_size,
            PUConfigs::StrategyPU(cfg) => cfg.queue_size
        }
    }

    pub fn downstreams(&self) -> &Vec<String> {
        match self {
            PUConfigs::EnginePU(cfg) => &cfg.downstreams,
            PUConfigs::MockPU(cfg) => &cfg.downstreams,
            PUConfigs::OrderManagerPU(cfg) => &cfg.downstreams,
            PUConfigs::OrderbookManagerPU(cfg) => &cfg.downstreams,
            PUConfigs::StrategyPU(cfg) => &cfg.downstreams
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EnginePUConfig {
    pub name: String,
    pub queue_size: usize,
    pub downstreams: Vec<String>
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MockPUConfig {
    pub name: String,
    pub queue_size: usize,
    pub downstreams: Vec<String>
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OrderManagerPUConfig {
    pub name: String,
    pub queue_size: usize,
    pub downstreams: Vec<String>
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OrderbookManagerPUConfig {
    pub name: String,
    pub queue_size: usize,
    pub downstreams: Vec<String>,
    pub realtime: bool
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StrategyPUConfig {
    pub name: String,
    pub queue_size: usize,
    pub downstreams: Vec<String>,
    pub realtime: bool,
    pub algorithm: String
}
