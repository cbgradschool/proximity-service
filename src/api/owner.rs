use crate::AppState;
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize)]
pub struct ApiPayload<T> {
    pub payload: T,
}

#[derive(Deserialize, Serialize)]
pub struct CreateOwner {
    pub name: String,
    pub email: String,
    pub password: String,
}

pub async fn get_owner() {
    // implement me
}

pub async fn post_owner(
    state: Extension<Arc<AppState>>,
    Json(req): Json<ApiPayload<CreateOwner>>,
) -> impl IntoResponse {
    let _owner_id = sqlx::query_scalar!(
        r#"insert into "owners" (name, email, password) values ($1, $2, $3)"#,
        req.payload.name,
        req.payload.email,
        req.payload.password,
    )
    .fetch_one(&state.db)
    .await;

    let response = CreateOwner {
        name: req.payload.name,
        email: req.payload.email,
        password: req.payload.password,
    };

    (StatusCode::CREATED, Json(response))
}

pub fn router() -> Router {
    Router::new()
        .route("/owner", get(get_owner))
        .route("/owner", post(post_owner))
}
