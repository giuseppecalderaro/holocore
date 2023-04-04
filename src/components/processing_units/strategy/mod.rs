pub mod base;
pub mod factory;
pub mod trend_following;

use crate::objects::{Objects, market::Market};

pub enum Algorithms {
    TrendFollowing(trend_following::TrendFollowing)
}

impl Algorithms {
    pub fn process(&mut self, obj: &Objects, market: &Market) -> Result<(Option<Objects>, bool), String> {
        match self {
            Algorithms::TrendFollowing(algo) => algo.process(obj, market)
        }
    }
}
