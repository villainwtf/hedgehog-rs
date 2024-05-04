use std::collections::HashMap;

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct FeatureFlagCollection {
    pub(crate) flags: HashMap<String, FeatureFlag>,
}

impl FeatureFlagCollection {
    pub(crate) fn new(flags: HashMap<String, FeatureFlag>) -> Self {
        Self { flags }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &FeatureFlag)> {
        self.flags.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn get(&self, key: &str) -> Option<&FeatureFlag> {
        self.flags.get(key)
    }

    pub fn get_str_flag(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|flag| flag.str())
    }

    pub fn get_bool_flag(&self, key: &str) -> bool {
        self.get(key).map_or(false, |flag| flag.variant_as_bool())
    }

    pub fn get_int_flag(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|flag| flag.int())
    }

    pub fn get_json_flag(&self, key: &str) -> Option<&Value> {
        self.get(key).and_then(|flag| flag.json())
    }

    pub fn get_typed_json_flag<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.get_json_flag(key)
            .and_then(|json| serde_json::from_value(json.clone()).ok())
    }
}

impl std::ops::Index<&str> for FeatureFlagCollection {
    type Output = FeatureFlag;

    fn index(&self, key: &str) -> &Self::Output {
        self.get(key).expect("no such feature flag")
    }
}

#[derive(Debug, Clone)]
pub struct FeatureFlag {
    pub(crate) variant: FeatureFlagData,
    pub(crate) payload: Option<FeatureFlagData>,
}

impl FeatureFlag {
    pub fn variant(&self) -> &FeatureFlagData {
        &self.variant
    }

    pub fn variant_as_str(&self) -> String {
        match &self.variant {
            FeatureFlagData::Boolean(b) => b.to_string(),
            FeatureFlagData::Integer(i) => i.to_string(),
            FeatureFlagData::String(s) => s.clone(),
            FeatureFlagData::Json(v) => v.to_string(),
        }
    }

    pub fn variant_as_bool(&self) -> bool {
        match &self.variant {
            FeatureFlagData::Boolean(b) => *b,
            FeatureFlagData::String(s) => s.parse().unwrap_or(false),
            FeatureFlagData::Integer(i) => *i != 0,
            FeatureFlagData::Json(_) => false,
        }
    }

    pub fn payload(&self) -> Option<&FeatureFlagData> {
        self.payload.as_ref()
    }

    pub fn str(&self) -> Option<&str> {
        match &self.payload {
            Some(FeatureFlagData::String(s)) => Some(s),
            _ => None,
        }
    }

    pub fn int(&self) -> Option<i64> {
        match &self.payload {
            Some(FeatureFlagData::Integer(i)) => Some(*i),
            _ => None,
        }
    }

    pub fn bool(&self) -> Option<bool> {
        match &self.payload {
            Some(FeatureFlagData::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn json(&self) -> Option<&Value> {
        match &self.payload {
            Some(FeatureFlagData::Json(v)) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FeatureFlagData {
    Boolean(bool),
    Integer(i64),
    String(String),
    Json(Value),
}

impl From<Value> for FeatureFlagData {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(b) => Self::Boolean(b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Self::Integer(i)
                } else {
                    Self::Json(Value::Number(n))
                }
            }
            Value::String(s) => Self::String(s),
            other => Self::Json(other),
        }
    }
}
