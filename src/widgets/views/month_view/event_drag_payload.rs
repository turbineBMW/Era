use clepsydre::Event;

use crate::utils::Date;

#[derive(Debug, Clone, glib::Boxed)]
#[boxed_type(name = "EventDragPayload")]
pub struct EventDragPayload {
    pub event: Event,
    pub anchor: Date,
}
