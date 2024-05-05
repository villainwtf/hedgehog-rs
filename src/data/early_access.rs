use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct EarlyAccessFeature {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) stage: String,
    #[serde(rename = "flagKey")]
    pub(crate) feature_flag: String,
}

impl EarlyAccessFeature {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn stage(&self) -> &str {
        &self.stage
    }

    pub fn feature_flag(&self) -> &str {
        &self.feature_flag
    }
}
