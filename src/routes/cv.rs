use axum::{routing::get, Router};

use crate::handlers::cv::cv;

pub fn router() -> Router {
    Router::new().route("/cv", get(cv))
}
