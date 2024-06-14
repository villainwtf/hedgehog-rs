use std::collections::HashMap;

use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::oneshot::channel;

use crate::{
    data::{Event, FeatureFlag, FeatureFlagCollection, Person, PropertyFilter},
    error::PosthogError,
};

use super::{
    queue::{PosthogRequest::EvaluateFeatureFlags, QueuedRequest},
    PosthogClient,
};

impl PosthogClient {
    pub async fn feature_flags(
        &self,
        person: &mut Person,
    ) -> Result<FeatureFlagCollection, PosthogError> {
        let json = json!({
            "api_key": self.api_key,
            "distinct_id": person.distinct_id,
            "person_properties": person.build_properties(PropertyFilter::new().include_person_properties(true).include_ip(true)),
        });

        let (tx, rx) = channel();

        self.queue.offer(QueuedRequest {
            request: EvaluateFeatureFlags { body: json },
            response_tx: Some(tx),
        });

        let json = rx.await.map_err(|_| PosthogError::QueueError)??;
        let json = serde_json::from_value::<PartialFeatureFlagResponse>(json)?;

        if json.error_computing_flags {
            return Err(PosthogError::FeatureFlagError);
        }

        let feature_flags = json
            .feature_flags
            .into_iter()
            .map(|(key, value)| {
                let payload = json.feature_flag_payloads.get(&key);

                Ok((
                    key,
                    FeatureFlag {
                        variant: value.into(),
                        payload: payload
                            .and_then(|p| serde_json::from_str::<Value>(p).ok())
                            .map(Into::into),
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>, PosthogError>>()?;

        let collection = FeatureFlagCollection::new(feature_flags);
        person.stored_feature_flags = Some(collection.clone());

        Ok(collection)
    }

    pub fn enqueue_feature_flag_called_event(
        &self,
        person: &Person,
        feature_flag: impl Into<String>,
        feature_flag_variant: impl Into<String>,
    ) -> Result<(), PosthogError> {
        Event::builder()
            .name("$feature_flag_called")
            .property("$feature_flag", feature_flag.into())
            .property("$feature_flag_response", feature_flag_variant.into())
            .build()?
            .enqueue(person, self)?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct PartialFeatureFlagResponse {
    #[serde(rename = "errorComputingFlags", default)]
    error_computing_flags: bool,
    #[serde(rename = "featureFlags")]
    feature_flags: HashMap<String, Value>,
    #[serde(rename = "featureFlagPayloads")]
    feature_flag_payloads: HashMap<String, String>,
}
