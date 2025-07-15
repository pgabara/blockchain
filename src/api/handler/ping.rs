use crate::api::response::Pong;
use crate::network::server::ServerError;

pub async fn handle_request() -> Result<Pong, ServerError> {
    tracing::debug!("Received Ping request");
    Ok(Pong)
}
