mod builder;
mod early_access;
mod event;
mod feature_flag;
mod identify;
mod queue;
mod view;

pub use builder::PosthogClientBuilder;

use self::queue::QueueWorker;

#[derive(Debug, Clone)]
pub struct PosthogClient {
    pub(crate) api_key: String,
    pub(crate) queue: QueueWorker,
}

impl PosthogClient {
    pub fn builder() -> PosthogClientBuilder {
        PosthogClientBuilder::new()
    }

    pub(crate) fn new(base_url: String, api_key: String) -> Self {
        Self {
            api_key,
            queue: QueueWorker::new(base_url),
        }
    }
}
