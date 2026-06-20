use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_time_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to evaluate time in milliseconds!")
        .as_millis()
}
