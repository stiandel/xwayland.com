use askama::Template;
use axum::{http::StatusCode, response::IntoResponse};
use crate::{markdown::markdown_to_html, templates::HtmlTemplate};

#[derive(Template)]
#[template(path = "cv.html")]
struct CvTemplate {
    content: String,
}

pub async fn cv() -> impl IntoResponse {
    let Ok(raw) = std::fs::read_to_string("content/cv.md") else {
        return (StatusCode::NOT_FOUND, "CV not found").into_response();
    };
    HtmlTemplate(CvTemplate {
        content: markdown_to_html(&raw),
    }).into_response()
}
