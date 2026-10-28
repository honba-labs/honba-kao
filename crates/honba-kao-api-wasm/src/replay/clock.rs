use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReplayClock {
    pub current_timestamp: i64,
    pub speed_multiplier: f64,
    pub is_running: bool,
    pub step_ms: i64,
}

impl ReplayClock {
    pub fn new(start_timestamp: i64, speed_multiplier: f64) -> Self {
        Self {
            current_timestamp: start_timestamp,
            speed_multiplier: speed_multiplier.clamp(0.1, 50.0),
            is_running: false,
            step_ms: 1000,
        }
    }

    pub fn tick(&mut self) -> i64 {
        if self.is_running {
            let advance = (self.step_ms as f64 * self.speed_multiplier) as i64;
            self.current_timestamp += advance;
        }
        self.current_timestamp
    }

    pub fn play(&mut self) {
        self.is_running = true;
    }

    pub fn pause(&mut self) {
        self.is_running = false;
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed_multiplier = speed.clamp(0.1, 50.0);
    }
}
