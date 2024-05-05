use std::sync::Arc;

use reqwest::{Client, Method};
use serde_json::{json, Value};
use tokio::{
    select,
    sync::{
        mpsc::{unbounded_channel, UnboundedSender},
        oneshot::Sender,
    },
    time::{interval, Duration},
};

use crate::error::PosthogError;

#[derive(Debug)]
pub enum PosthogRequest {
    /// Capture an event.
    ///
    /// Endpoint: /capture
    /// Method: POST
    CaptureEvent { body: Value },

    /// Capture multiple events in a single request.
    ///
    /// Endpoint: /batch
    /// Method: POST
    CaptureBatch { body: Value },

    /// Evaluate feature flags.
    ///
    /// Endpoint: /decide?v=3
    /// Method: POST
    EvaluateFeatureFlags { body: Value },

    /// Get early access features.
    ///
    /// Endpoint: /api/early_access_features
    /// Method: GET
    GetEarlyAccessFeatures { api_key: String },

    /// Any other request.
    ///
    /// Endpoint: Any
    /// Method: Any
    Other {
        method: Method,
        endpoint: String,
        json: Value,
    },
}

impl Default for PosthogRequest {
    fn default() -> Self {
        Self::Other {
            method: Method::GET,
            endpoint: String::new(),
            json: Value::Null,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct QueuedRequest {
    pub(crate) request: PosthogRequest,

    pub(crate) immediate: bool,
    pub(crate) response_tx: Option<Sender<Result<Value, PosthogError>>>,
}

#[derive(Debug)]
pub(crate) struct QueueWorker {
    pub(crate) base_url: String,
    pub(crate) client: Client,

    pub(crate) tx: UnboundedSender<QueuedRequest>,
}

impl QueueWorker {
    pub(crate) fn new(base_url: String) -> Arc<Self> {
        let (tx, mut rx) = unbounded_channel::<QueuedRequest>();

        let worker = Self {
            base_url,
            client: Client::new(),
            tx,
        };

        let worker = Arc::new(worker);

        {
            let worker = Arc::clone(&worker);

            tokio::spawn(async move {
                let mut queued_requests = Vec::new();
                let mut flush_timer = interval(Duration::from_secs(1));

                loop {
                    select! {
                        Some(request) = rx.recv() => {
                            if request.immediate {
                                worker.dispatch_request(Arc::clone(&worker), request).await;
                            } else {
                                queued_requests.push(request);
                            }
                        }

                        _ = flush_timer.tick() => {
                            if queued_requests.is_empty() {
                                continue;
                            }

                            let mut requests = Vec::new();
                            std::mem::swap(&mut queued_requests, &mut requests);

                            let mut batch_capture = Vec::new();

                            for request in requests {
                                match request.request {
                                    PosthogRequest::CaptureEvent { body } if request.response_tx.is_none() => {
                                        batch_capture.push(body);
                                    }

                                    _ => {
                                        worker.dispatch_request(Arc::clone(&worker), request).await;
                                    }
                                }
                            }

                            if !batch_capture.is_empty() {
                                // The API key is added by the client to each event, so we can just take it from the first event.
                                let api_key = batch_capture[0]["api_key"].as_str().unwrap();

                                let body = json!({
                                    "api_key": api_key,
                                    "batch": batch_capture,
                                 });

                                let request = QueuedRequest {
                                    request: PosthogRequest::CaptureBatch { body },
                                    immediate: true,
                                    response_tx: None,
                                };

                                worker.dispatch_request(Arc::clone(&worker), request).await;
                            }
                        }
                    }
                }
            });
        }

        worker
    }

    async fn dispatch_request(&self, worker: Arc<Self>, request: QueuedRequest) {
        tokio::spawn(async move {
            worker.handle_request(request).await;
        });
    }

    async fn handle_request(&self, request: QueuedRequest) {
        let (method, endpoint, body) = match request.request {
            PosthogRequest::CaptureEvent { body } => (Method::POST, "capture".to_string(), body),

            PosthogRequest::CaptureBatch { body } => (Method::POST, "batch".to_string(), body),

            PosthogRequest::EvaluateFeatureFlags { body } => {
                (Method::POST, "decide?v=3".to_string(), body)
            }

            PosthogRequest::GetEarlyAccessFeatures { api_key } => (
                Method::GET,
                format!("api/early_access_features?api_key={}", api_key),
                Value::Null,
            ),

            PosthogRequest::Other {
                method,
                endpoint,
                json,
            } => (method, endpoint, json),
        };

        let response = self.send_request(method, endpoint, body).await;

        if let Some(response_tx) = request.response_tx {
            response_tx.send(response).ok();
        }
    }

    async fn send_request(
        &self,
        method: Method,
        endpoint: impl Into<String>,
        json: Value,
    ) -> Result<Value, PosthogError> {
        let response = self
            .client
            .request(method, &format!("{}/{}", self.base_url, endpoint.into()))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&json)
            .send()
            .await
            .map_err(PosthogError::HttpError)?;

        if response.status().is_success() {
            response.json().await.map_err(PosthogError::HttpError)
        } else {
            Err(PosthogError::HttpError(
                response.error_for_status().unwrap_err(),
            ))
        }
    }
}
