use axum::{routing::get, Router};

#[tracing::instrument(name = "Health Check")]
pub async fn get_health_check() -> Result<(), ()> {
    // Returns an empty response
    Ok(())
}

pub fn router() -> Router {
    Router::new().route("/health_check", get(get_health_check))
}
