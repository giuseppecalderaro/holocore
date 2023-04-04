use crate::objects::common::{ deserialize_fixed, serialize_fixed };

use fixed::FixedU64;
use serde::{ Deserialize, Serialize };

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Level {
    #[serde(serialize_with = "serialize_fixed", deserialize_with = "deserialize_fixed")]
    pub price: FixedU64<32>,
    #[serde(serialize_with = "serialize_fixed", deserialize_with = "deserialize_fixed")]
    pub quantity: FixedU64<32>
}

impl Level {
    pub fn new(price: FixedU64<32>, quantity: FixedU64<32>) -> Self {
        Self {
            price,
            quantity
        }
    }
}
