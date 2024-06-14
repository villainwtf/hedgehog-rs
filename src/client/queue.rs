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
    pub(crate) response_tx: Option<Sender<Result<Value, PosthogError>>>,
}

#[derive(Clone, Debug)]
pub(crate) struct QueueWorker {
    client: QueueClient,
    batch_capture_tx: UnboundedSender<Value>,
}

#[derive(Clone, Debug)]
struct QueueClient {
    base_url: String,
    client: Client,
}

impl QueueWorker {
    pub(crate) fn new(base_url: String) -> Self {
        let client = QueueClient {
            base_url: base_url.clone(),
            client: Client::new(),
        };

        let (batch_capture_tx, mut rx) = unbounded_channel::<Value>();

        let worker = Self {
            client,
            batch_capture_tx,
        };

        // Only capture events can be batched, everything else will be executed immediately.
        {
            let worker = worker.clone();

            tokio::spawn(async move {
                let mut events = vec![];
                let mut flush_timer = interval(Duration::from_secs(1));

                loop {
                    select! {
                        Some(event) = rx.recv() => {
                            events.push(event);
                        }

                        _ = flush_timer.tick() => {
                            if events.is_empty() {
                                continue;
                            }

                            let mut batch = vec![];
                            std::mem::swap(&mut events, &mut batch);

                            // The API key is added by the client to each event, so we can just take it from the first event.
                            let api_key = batch[0]["api_key"].as_str().unwrap();

                            let body = json!({
                                "api_key": api_key,
                                "batch": batch,
                            });

                            let request = QueuedRequest {
                                request: PosthogRequest::CaptureBatch { body },
                                response_tx: None,
                            };

                            worker.dispatch_request(request);
                        }
                    }
                }
            });
        }

        worker
    }

    pub fn offer(&self, request: QueuedRequest) {
        match request.request {
            PosthogRequest::CaptureEvent { body } => {
                self.batch_capture_tx.send(body).ok();
            }

            _ => {
                self.dispatch_request(request);
            }
        }
    }

    pub fn dispatch_request(&self, request: QueuedRequest) {
        let client = self.client.clone();
        tokio::spawn(async move {
            QueueWorker::handle_request(client, request).await;
        });
    }

    async fn handle_request(client: QueueClient, request: QueuedRequest) {
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

        let response = QueueWorker::send_request(client, method, endpoint, body).await;

        if let Some(response_tx) = request.response_tx {
            response_tx.send(response).ok();
        }
    }

    async fn send_request(
        client: QueueClient,
        method: Method,
        endpoint: impl Into<String>,
        json: Value,
    ) -> Result<Value, PosthogError> {
        let response = client
            .client
            .request(method, &format!("{}/{}", client.base_url, endpoint.into()))
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
