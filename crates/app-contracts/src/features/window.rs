use guinea::prelude::Event;
use serde::Deserialize;

#[derive(Clone, Debug, Event, Deserialize, guinea::Remote)]
#[remote(event)]
pub struct PressedAway;
