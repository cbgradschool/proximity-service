use crate::AppState;
use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
    Extension, Json, Router,
};
use hyper::StatusCode;
use sea_query::*;
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, PgPool, Row};
use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiPayload<T> {
    pub payload: T,
}

#[enum_def] // => Generates OwnersIden
#[derive(Deserialize, Serialize)]
pub struct Owners {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateOwner {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateProfile {
    pub name: String,  // NOTE: Is this the appropriate type?
    pub owner_id: i32, // NOTE: Why does u32 not work for making the sql query
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateCredentials {
    pub email: String,
    pub password: String,
    pub owner_id: i32, // NOTE: Why does u32 not work for making the sql query
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateOwnerResponse {
    pub id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Server Error: {}", self.0),
        )
            .into_response()
    }
}

#[tracing::instrument(name = "SELECT a single owner")]
pub async fn select_owner(id: i32, db: &PgPool) -> Result<Owners, sqlx::Error> {
    let (sql, values) = Query::select()
        .columns([
            OwnersIden::Id,
            OwnersIden::Name,
            OwnersIden::Email,
            OwnersIden::Password,
        ])
        .from(OwnersIden::Table)
        .and_where(Expr::col(OwnersIden::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .map(|row: PgRow| Owners {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
        })
        .fetch_one(db)
        .await
        .map_err(|error| {
            tracing::error!("Failed to execute query: {:?}", error);
            error
        })
}

// Pattern: Error handling
#[tracing::instrument(name = "GET a single Owner resource")]
pub async fn get_owner(
    Path(id): Path<i32>,
    state: Extension<Arc<AppState>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match select_owner(id, &state.db).await {
        Ok(record) => Ok((StatusCode::OK, Json(record))),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            format!("Record not found for id: {:?}", id),
        )),
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unknown Error: {:?}", error),
        )),
    }
}

#[tracing::instrument(name = "CREATE a single owner")]
pub async fn create_owner(
    owner: CreateOwner,
    db: &PgPool,
) -> Result<CreateOwnerResponse, sqlx::Error> {
    let (sql, values) = Query::insert()
        .into_table(OwnersIden::Table)
        .columns([OwnersIden::Name, OwnersIden::Email, OwnersIden::Password])
        .values_panic([owner.name.into(), owner.email.into(), owner.password.into()])
        .returning(Query::returning().columns([OwnersIden::Id]))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .map(|row: PgRow| CreateOwnerResponse { id: row.get("id") })
        .fetch_one(db)
        .await
        .map_err(|error| {
            tracing::error!("Failed to execute query: {:?}", error);
            error
        })
}

#[tracing::instrument(name = "POST a single Owner resource")]
pub async fn post_owner(
    state: Extension<Arc<AppState>>,
    Json(req): Json<ApiPayload<CreateOwner>>,
) -> impl IntoResponse {
    let owner = CreateOwner {
        name: req.payload.name,
        email: req.payload.email,
        password: req.payload.password,
    };

    match create_owner(owner, &state.db).await {
        Ok(record) => Ok((
            StatusCode::CREATED,
            Json(CreateOwnerResponse { id: record.id }),
        )),
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unknown Error: {:?}", error),
        )),
    }
}

#[tracing::instrument(name = "Update an Owner's profile")]
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

// TODO: Obfuscate password and possibly email
#[tracing::instrument(name = "Update an Owner's credentials")]
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

#[tracing::instrument(name = "Delete a single owner record")]
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
        .route("/owner", post(post_owner))
        .route("/owner/:id", delete(delete_owner))
        .route("/owner/:id/profile", patch(update_profile))
        .route("/owner/:id/credentials", patch(update_credentials))
}
