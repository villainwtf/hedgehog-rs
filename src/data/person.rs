use std::collections::HashMap;

use serde_json::{json, Value};

use crate::error::PosthogError;

use super::FeatureFlagCollection;

#[derive(Default, Debug, Clone)]
pub struct PropertyFilter {
    pub(crate) include_person_properties: bool,
    pub(crate) use_set_syntax: bool,
    pub(crate) include_ip: bool,
    pub(crate) include_feature_flags: bool,
}

impl PropertyFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include_person_properties(mut self, include_person_properties: bool) -> Self {
        self.include_person_properties = include_person_properties;
        self
    }

    pub fn use_set_syntax(mut self, use_set_syntax: bool) -> Self {
        self.use_set_syntax = use_set_syntax;
        self
    }

    pub fn include_ip(mut self, include_ip: bool) -> Self {
        self.include_ip = include_ip;
        self
    }

    pub fn include_feature_flags(mut self, include_feature_flags: bool) -> Self {
        self.include_feature_flags = include_feature_flags;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Person {
    pub(crate) distinct_id: String,
    pub(crate) properties: Option<HashMap<String, Value>>,
    pub(crate) stored_feature_flags: Option<FeatureFlagCollection>,
    pub(crate) client_ip: Option<String>,
}

impl Person {
    pub fn builder() -> PersonBuilder {
        PersonBuilder {
            distinct_id: None,
            properties: HashMap::new(),
            client_ip: None,
        }
    }

    pub fn distinct_id(&self) -> &str {
        &self.distinct_id
    }

    pub fn set_client_ip(&mut self, client_ip: impl Into<String>) {
        self.client_ip = Some(client_ip.into());
    }

    pub fn stored_feature_flags(&self) -> Option<&FeatureFlagCollection> {
        self.stored_feature_flags.as_ref()
    }

    pub(crate) fn build_properties(&self, filter: PropertyFilter) -> HashMap<String, Value> {
        let mut properties = HashMap::new();

        if filter.include_person_properties {
            if let Some(person_properties) = &self.properties {
                if filter.use_set_syntax {
                    properties.insert("$set".to_string(), json!(person_properties));
                } else {
                    properties.extend(person_properties.clone());
                }
            }
        }

        if filter.include_ip {
            if let Some(client_ip) = &self.client_ip {
                properties.insert("$ip".to_string(), Value::String(client_ip.clone()));
            }
        }

        if filter.include_feature_flags {
            if let Some(active_feature_flags) = &self.stored_feature_flags {
                for (key, value) in active_feature_flags.iter() {
                    properties.insert(format!("$feature/{}", key), value.variant_as_str().into());
                }
            }
        }

        properties
    }
}

pub struct PersonBuilder {
    distinct_id: Option<String>,
    properties: HashMap<String, Value>,
    client_ip: Option<String>,
}

impl PersonBuilder {
    pub fn distinct_id(mut self, distinct_id: impl Into<String>) -> Self {
        self.distinct_id = Some(distinct_id.into());
        self
    }

    pub fn property(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    pub fn properties(mut self, properties: HashMap<String, Value>) -> Self {
        self.properties = properties;
        self
    }

    pub fn client_ip(mut self, client_ip: impl Into<String>) -> Self {
        self.client_ip = Some(client_ip.into());
        self
    }

    pub fn build(self) -> Result<Person, PosthogError> {
        let distinct_id = self.distinct_id.ok_or(PosthogError::DistinctIdRequired)?;

        Ok(Person {
            distinct_id,
            properties: if self.properties.is_empty() {
                None
            } else {
                Some(self.properties)
            },
            stored_feature_flags: None,
            client_ip: self.client_ip,
        })
    }
}
