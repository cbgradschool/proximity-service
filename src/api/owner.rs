use crate::AppState;
use axum::{
    extract::Path,
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

#[derive(Deserialize, Serialize)]
pub struct CreateOwnerResponse {
    pub id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct Owner {
    pub id: i32,
    pub name: String,
    pub email: String,
}

pub async fn get_owner(Path(id): Path<i32>, state: Extension<Arc<AppState>>) -> impl IntoResponse {
    let owner = sqlx::query!(r#"select id, name, email from "owners" where id = $1;"#, id,)
        .fetch_one(&state.db)
        .await
        .unwrap();

    (
        StatusCode::OK,
        Json(Owner {
            id: owner.id,
            name: owner.name,
            email: owner.email,
        }),
    )
}

pub async fn post_owner(
    state: Extension<Arc<AppState>>,
    Json(req): Json<ApiPayload<CreateOwner>>,
) -> impl IntoResponse {
    let insert_id = sqlx::query_scalar!(
        r#"insert into "owners" (name, email, password) values ($1, $2, $3) returning id;"#,
        req.payload.name,
        req.payload.email,
        req.payload.password,
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    let response = CreateOwnerResponse { id: insert_id };

    (StatusCode::CREATED, Json(response))
}

pub fn router() -> Router {
    Router::new()
        .route("/owner/:id", get(get_owner))
        .route("/owner", post(post_owner))
}
