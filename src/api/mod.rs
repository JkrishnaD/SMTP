use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    api::routes::{
        handle_email_by_user, handle_get_users, handle_health, handle_post_user, handle_root,
    },
    storage::Store,
};

mod routes;

/// All the routes for the API
/// GET /users - get all users
/// POST /user - create a new user
/// GET emails/:user - get all emails for a user
/// GET emails/:id - get a specific email by id
/// DELETE /email/:id - delete a specific email by id
/// POST /send - send an email
/// GET /health - get the health of the server
pub fn create_router(store: Store) -> Router {
    Router::new()
        .route("/", get(handle_root))
        .route("/users", get(handle_get_users))
        .route("/user", post(handle_post_user))
        .route("/health", get(handle_health))
        .route("/email/{email}", get(handle_email_by_user))
        .with_state(store)
}
