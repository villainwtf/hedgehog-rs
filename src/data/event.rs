use std::collections::HashMap;

use serde_json::Value;

use crate::{client::PosthogClient, error::PosthogError};

use super::{Person, PropertyFilter};

pub struct Event {
    pub(crate) name: String,
    pub(crate) properties: Option<HashMap<String, Value>>,
    pub(crate) is_identify: bool,
}

impl Event {
    pub fn builder() -> EventBuilder {
        EventBuilder {
            name: None,
            properties: HashMap::new(),
            is_identify: false,
        }
    }

    pub async fn capture(
        self,
        person: &Person,
        posthog: &PosthogClient,
    ) -> Result<(), PosthogError> {
        posthog.capture_event(person, self).await
    }

    pub fn enqueue(self, person: &Person, posthog: &PosthogClient) -> Result<(), PosthogError> {
        posthog.enqueue_event(person, self)
    }

    pub(crate) fn build_properties(&self, person: &Person) -> HashMap<String, Value> {
        let mut properties = self.properties.clone().unwrap_or_default();

        let person_event_properties = person.build_properties(
            PropertyFilter::new()
                .include_person_properties(self.is_identify)
                .use_set_syntax(self.is_identify)
                .include_ip(true)
                .include_feature_flags(true),
        );
        properties.extend(person_event_properties);

        properties
    }
}

pub struct EventBuilder {
    name: Option<String>,
    properties: HashMap<String, Value>,
    is_identify: bool,
}

impl EventBuilder {
    pub fn name(mut self, event_name: impl Into<String>) -> Self {
        self.name = Some(event_name.into());

        if self.name.as_ref().unwrap() == "$identify" {
            self.is_identify = true;
        }

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

    pub fn identify(mut self) -> Self {
        self.is_identify = true;
        self
    }

    pub fn build(self) -> Result<Event, PosthogError> {
        let event_name = self.name.ok_or(PosthogError::EventNameRequired)?;

        Ok(Event {
            name: event_name,
            properties: if self.properties.is_empty() {
                None
            } else {
                Some(self.properties)
            },
            is_identify: self.is_identify,
        })
    }
}
