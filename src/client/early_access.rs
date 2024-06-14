use serde::Deserialize;
use serde_json::json;
use tokio::sync::oneshot::channel;

use crate::{
    data::{EarlyAccessFeature, Event, Person},
    error::PosthogError,
};

use super::{
    queue::{PosthogRequest::GetEarlyAccessFeatures, QueuedRequest},
    PosthogClient,
};

impl PosthogClient {
    pub fn enqueue_early_access_feature_enrollment(
        &self,
        person: &Person,
        feature: impl Into<String>,
        is_enrolled: bool,
    ) -> Result<(), PosthogError> {
        let feature = feature.into();

        Event::builder()
            .name("$feature_enrollment_update")
            .property("$feature_flag", feature.clone())
            .property("$feature_enrollment", is_enrolled)
            .property(
                "$set",
                json!({
                    format!("$feature_enrollment/{}", feature): is_enrolled
                }),
            )
            .build()?
            .enqueue(person, self)?;

        Ok(())
    }

    pub async fn early_access_features(&self) -> Result<Vec<EarlyAccessFeature>, PosthogError> {
        let (tx, rx) = channel();

        self.queue
            .offer(QueuedRequest {
                request: GetEarlyAccessFeatures {
                    api_key: self.api_key.clone(),
                },
                response_tx: Some(tx),
            })
            .await;

        let json = rx.await.map_err(|_| PosthogError::QueueError)??;
        let json = serde_json::from_value::<PartialEarlyAccessFeaturesResponse>(json)?;

        Ok(json.early_access_features)
    }
}

#[derive(Deserialize)]
struct PartialEarlyAccessFeaturesResponse {
    #[serde(rename = "earlyAccessFeatures")]
    early_access_features: Vec<EarlyAccessFeature>,
}
