#[derive(thiserror::Error, Debug)]
pub enum PosthogError {
    #[error("Base URL is required")]
    BaseUrlRequired,
    #[error("API key is required")]
    ApiKeyRequired,

    #[error("Distinct ID is required")]
    DistinctIdRequired,
    #[error("Event name is required")]
    EventNameRequired,

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Server failed to compute feature flags")]
    FeatureFlagError,

    #[error("Failed to enqueue request")]
    QueueError,
}
