use crate::AppState;
use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Extension, Json, Router,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiPayload<T> {
    pub payload: T,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateOwner {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateProfile {
    pub name: String,  // NOTE: Is this the appropriate type?
    pub owner_id: i32, // NOTE: Why does u32 not work for making the sql query
}

#[derive(Deserialize, Serialize)]
pub struct UpdateCredentials {
    pub email: String,
    pub password: String,
    pub owner_id: i32, // NOTE: Why does u32 not work for making the sql query
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

#[tracing::instrument(name = "Fetch a single Owner resource")]
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

#[tracing::instrument(name = "Create a single Owner resource")]
pub async fn create_owner(
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

pub async fn update_profile(
    state: Extension<Arc<AppState>>,
    Json(req): Json<ApiPayload<UpdateProfile>>,
) -> impl IntoResponse {
    let _patch = sqlx::query_scalar!(
        r#"UPDATE owners SET name = $1 WHERE id = $2;"#,
        req.payload.name,
        req.payload.owner_id
    )
    .fetch_one(&state.db)
    .await;

    StatusCode::NO_CONTENT
}

pub async fn update_credentials(
    state: Extension<Arc<AppState>>,
    Json(req): Json<ApiPayload<UpdateCredentials>>,
) -> impl IntoResponse {
    let _patch = sqlx::query_scalar!(
        r#"UPDATE owners SET email = $1, password = $2 WHERE id = $3;"#,
        req.payload.email,
        req.payload.password,
        req.payload.owner_id,
    )
    .fetch_one(&state.db)
    .await;

    StatusCode::NO_CONTENT
}

pub async fn delete_owner(
    Path(id): Path<i32>,
    state: Extension<Arc<AppState>>,
) -> impl IntoResponse {
    let _delete = sqlx::query_scalar!(r#"delete from "owners" where id = $1"#, id,)
        .fetch_one(&state.db)
        .await;

    StatusCode::NO_CONTENT
}

pub fn router() -> Router {
    Router::new()
        .route("/owner/:id", get(get_owner))
        .route("/owner", post(create_owner))
        .route("/owner/:id", delete(delete_owner))
        .route("/owner/:id/profile", patch(update_profile))
        .route("/owner/:id/credentials", patch(update_credentials))
}
