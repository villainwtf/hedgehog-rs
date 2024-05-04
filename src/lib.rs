//!
//! This library provides a client to interact with the Posthog API.
//!
//! # Example
//! ```no_run
//! use hedgehog::client::PosthogClient;
//! use hedgehog::data::{Event, Person};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new Posthog client
//!     let client = PosthogClient::builder()
//!         .base_url("https://app.posthog.com")
//!         .api_key("your-api-key")
//!         .build()?;
//!
//!     // Create a new person
//!     let mut person = Person::builder()
//!         .distinct_id("12345")
//!         .property("name", "John Doe")
//!         .build()?;
//!
//!     // Identify the person
//!     client.enqueue_identify(&person)?;
//!
//!     // Capture an event
//!     Event::builder()
//!         .name("test event")
//!         .property("key", "value")
//!         .build()?
//!         .enqueue(&person, &client)?;
//!
//!     // Record a page view
//!     client.enqueue_page_view_event(&person, "Test Page")?;
//!
//!     // Record a screen view
//!     client.enqueue_screen_view_event(&person, "Test Screen")?;
//!
//!     // Evaluate feature flags
//!     // Note: we use a mutable reference here so the retrieved feature flags can be stored in the person object,
//!     //       which the client will automatically include in future events, and so the feature flags can be used
//!     //       in the application without needing to be fetched again.
//!     let feature_flags = client.feature_flags(&mut person).await?;
//!
//!     // Test a feature flag
//!     if feature_flags.get_bool_flag("test_feature_flag") {
//!         println!("Feature flag is enabled");
//!     } else {
//!         println!("Feature flag is disabled");
//!     }
//!
//!     // Print a feature flag
//!     let json_flag = feature_flags.get_json_flag("json_feature_flag");
//!     if let Some(json_flag) = json_flag {
//!         println!("JSON feature flag: {:?}", json_flag);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod data;
pub mod error;
