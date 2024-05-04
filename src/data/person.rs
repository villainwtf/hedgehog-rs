use std::collections::HashMap;

use serde_json::Value;

use crate::error::PosthogError;

use super::FeatureFlagCollection;

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

    pub(crate) fn build_properties(
        &self,
        include_person_properties: bool,
        include_ip: bool,
        include_feature_flags: bool,
    ) -> HashMap<String, Value> {
        let mut properties = HashMap::new();

        if include_person_properties {
            if let Some(person_properties) = &self.properties {
                properties.extend(person_properties.clone());
            }
        }

        if include_ip {
            if let Some(client_ip) = &self.client_ip {
                properties.insert("$ip".to_string(), Value::String(client_ip.clone()));
            }
        }

        if include_feature_flags {
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
