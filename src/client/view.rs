use crate::{
    data::{Event, Person},
    error::PosthogError,
};

use super::PosthogClient;

impl PosthogClient {
    pub fn enqueue_page_view_event(
        &self,
        person: &Person,
        title: impl Into<String>,
    ) -> Result<(), PosthogError> {
        Event::builder()
            .name("$pageview")
            .property("title", title.into())
            .build()?
            .enqueue(person, self)?;

        Ok(())
    }

    pub fn enqueue_screen_view_event(
        &self,
        person: &Person,
        screen_name: impl Into<String>,
    ) -> Result<(), PosthogError> {
        Event::builder()
            .name("$screen")
            .property("$screen_name", screen_name.into())
            .build()?
            .enqueue(person, self)?;

        Ok(())
    }
}
