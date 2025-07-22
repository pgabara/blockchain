use crate::api::response::Pong;

pub async fn handle_request() -> Pong {
    tracing::debug!("Received Ping request");
    Pong
}
