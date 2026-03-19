use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{
    models::{EmailResponse, NewUser, User},
    storage::Store,
};

pub async fn handle_root() -> String {
    "SMTP!, REST server!".to_string()
}

#[axum::debug_handler]
pub async fn handle_get_users(
    State(store): State<Store>, // Remove 'mut' from here
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    // Return a Tuple for proper error response
    let users = store
        .get_users()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}

pub async fn handle_post_user(
    State(store): State<Store>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let email = new_user.email.clone();
    let password = new_user.password_hash.clone();

    let user = store
        .create_user(email, password)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(user))
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    smtp: &'static str,
    db: &'static str,
}

pub async fn handle_health() -> (StatusCode, Json<HealthResponse>) {
    let response = HealthResponse {
        status: "Ok",
        smtp: "running",
        db: "connected",
    };

    (StatusCode::OK, Json(response))
}

#[derive(Deserialize)]
pub struct EmailGetParams {
    pub email: String,
}

pub async fn handle_email_by_user(
    State(store): State<Store>,
    Path(params): Path<EmailGetParams>,
) -> Result<Json<Vec<EmailResponse>>, (StatusCode, String)> {
    let email_details = store
        .get_emails_by_user(params.email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(email_details))
}

#[derive(Deserialize)]
pub struct UserDeleteParams {
    pub id: i32,
}

pub async fn handle_delete_user(
    State(store): State<Store>,
    Path(params): Path<UserDeleteParams>,
) -> Result<Json<usize>, (StatusCode, String)> {
    let user = store
        .delete_user_by_id(params.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(user))
}
