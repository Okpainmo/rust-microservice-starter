use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    pub expires_at_in_milliseconds: i64,
}
