use axum::{routing::get, Router};

use crate::handlers::about::about;

pub fn router() -> Router {
    Router::new().route("/about", get(about))
}
