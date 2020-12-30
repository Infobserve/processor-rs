use crate::entities::{FlatMatch, Event};

#[derive(Debug)]
pub struct ProcessedEvent(pub Event, pub Vec<FlatMatch>);