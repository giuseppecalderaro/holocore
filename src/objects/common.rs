use fixed::FixedU64;
use serde::{ Deserializer, Serializer, ser::SerializeMap, de::Visitor, de::MapAccess };
use std::collections::BTreeMap;

// Serializers
pub fn serialize_fixed<S>(value: &FixedU64<32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    serializer.serialize_str(&value.to_string())
}

pub fn serialize_bids<S>(levels: &BTreeMap<FixedU64<32>, FixedU64<32>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    let mut map = serializer.serialize_map(Some(levels.len()))?;
    for (k, v) in levels.iter().rev() {
        map.serialize_entry(&k.to_string(), &v.to_string())?;
    }
    map.end()
}

pub fn serialize_asks<S>(levels: &BTreeMap<FixedU64<32>, FixedU64<32>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    let mut map = serializer.serialize_map(Some(levels.len()))?;
    for (k, v) in levels.iter() {
        map.serialize_entry(&k.to_string(), &v.to_string())?;
    }
    map.end()
}

// Deserializers
struct DeserializerFixedVisitor;

impl<'de> Visitor<'de> for DeserializerFixedVisitor {
    type Value = FixedU64<32>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("&str to FixedU64")
    }

    fn visit_str<D>(self, v: &str) -> Result<Self::Value, D>
    where
        D: serde::de::Error
    {
        match FixedU64::<32>::from_str(v) {
            Ok(value) => Ok(value),
            Err(e) => {
                log::error!("DeserializerFixedVisitor: cannot deserialize {} - {}", v, e);
                Ok(FixedU64::<32>::from_num(0))
            }
        }
    }
}

pub fn deserialize_fixed<'de, D>(deserializer: D) -> Result<FixedU64<32>, D::Error>
where
    D: Deserializer<'de>
{
    deserializer.deserialize_str(DeserializerFixedVisitor)
}

struct DeserializerVisitor;

impl<'de> Visitor<'de> for DeserializerVisitor {
    type Value = BTreeMap<FixedU64<32>, FixedU64<32>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Map of Strings to BTreeMap")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = BTreeMap::<FixedU64<32>, FixedU64<32>>::new();

        while let Some((key, value)) = access.next_entry()?
            .map(|(k, v): (String, String)| (FixedU64::<32>::from_str(&k), FixedU64::<32>::from_str(&v))) {
                if let Ok(key) = key {
                    if let Ok(value) = value {
                       values.insert(key, value);
                    }
                }
            }

        Ok(values)
    }
}

pub fn deserialize_bids<'de, D>(deserializer: D) -> Result<BTreeMap<FixedU64<32>, FixedU64<32>>, D::Error>
where
    D: Deserializer<'de>
{
    deserializer.deserialize_map(DeserializerVisitor)
}

pub fn deserialize_asks<'de, D>(deserializer: D) -> Result<BTreeMap<FixedU64<32>, FixedU64<32>>, D::Error>
where
    D: Deserializer<'de>
{
    deserializer.deserialize_map(DeserializerVisitor)
}
