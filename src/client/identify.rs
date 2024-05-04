use crate::{
    data::{Event, Person},
    error::PosthogError,
};

use super::PosthogClient;

impl PosthogClient {
    pub fn enqueue_identify(&self, person: &Person) -> Result<(), PosthogError> {
        Event::builder()
            .name("$identify")
            .build()?
            .enqueue(person, self)?;

        Ok(())
    }
}
