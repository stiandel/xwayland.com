use askama::Template;
use axum::response::IntoResponse;

use crate::templates::HtmlTemplate;

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate;

pub async fn about() -> impl IntoResponse {
    HtmlTemplate(AboutTemplate).into_response()
}
