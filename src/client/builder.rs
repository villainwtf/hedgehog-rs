use crate::error::PosthogError;

use super::PosthogClient;

pub struct PosthogClientBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
}

impl PosthogClientBuilder {
    pub(crate) fn new() -> Self {
        Self {
            base_url: None,
            api_key: None,
        }
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn build(self) -> Result<PosthogClient, PosthogError> {
        let base_url = self.base_url.ok_or(PosthogError::BaseUrlRequired)?;
        let api_key = self.api_key.ok_or(PosthogError::ApiKeyRequired)?;

        Ok(PosthogClient::new(base_url, api_key))
    }
}
