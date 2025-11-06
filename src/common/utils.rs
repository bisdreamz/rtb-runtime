use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current epoch timestamp (in millis)
pub fn epoch_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
