use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReplayEvent {
    pub timestamp: i64,
    pub event_type: String,
    pub payload_json: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ReplayJournal {
    pub events: Vec<ReplayEvent>,
    pub cursor: usize,
}

impl ReplayJournal {
    pub fn new(events: Vec<ReplayEvent>) -> Self {
        Self { events, cursor: 0 }
    }

    pub fn next_event_until(&mut self, timestamp: i64) -> Option<&ReplayEvent> {
        if self.cursor < self.events.len() && self.events[self.cursor].timestamp <= timestamp {
            let ev = &self.events[self.cursor];
            self.cursor += 1;
            Some(ev)
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }
}
