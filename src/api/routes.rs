use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::{
    models::{NewUser, User},
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
