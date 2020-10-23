#[allow(dead_code)]
pub enum EventSource {
    /// It is used to differentiate between
    /// the different sources the data came from
    Gist,
    Github,
    Paste
}

pub struct Event {
    /// Represents a data source agnostic event
    ///
    /// # Fields
    ///
    /// * `id`: A unique identifier for this event. Could be of different format based on the source
    /// * `url`: The url from which this file was retrieved
    /// * `size`: The size of the file (in bytes)
    /// * `filename`: The name of the file as retrieved from the source
    /// * `creator`: The username of the creator
    /// * `raw_content`: The entire content of the file as retrieved from the source
    /// * `timestamp`: The time at which this event was received
    /// * `event_type`: The source from which the event was received
    id: String,
    url: String,
    size: u64,
    filename: String,
    creator: String,
    raw_content: String,
    timestamp: String,
    event_source: EventSource
}

#[allow(dead_code)]
impl Event {
    pub fn new(id: String, url: String, size: u64, filename: String,
                creator: String, raw_content: String, timestamp: String,
                event_source: EventSource) -> Self {
        Self {
            id, url, size, filename,
            creator, raw_content, timestamp,
            event_source
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn creator(&self) -> &str {
        &self.creator
    }

    pub fn raw_content(&self) -> &str {
        &self.raw_content
    }

    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    pub fn event_source(&self) -> &EventSource {
        &self.event_source
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_does_not_blow_up() {
        Event::new(
            String::from("test_id"),
            String::from("some_url"),
            5000,
            String::from("password_leaks.txt"),
            String::from("h4x0rZ"),
            String::from("non-password-related-stuff"),
            String::from("2020-02-20 17:18:19"),
            EventSource::Github
        );
    }
}