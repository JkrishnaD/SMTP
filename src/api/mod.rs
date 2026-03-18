use axum::{Router, routing::get};

use crate::{api::routes::handle_root, storage::Store};

mod routes;

pub fn create_router(store: Store) -> Router {
    Router::new().route("/", get(handle_root))
}
