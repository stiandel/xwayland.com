use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use askama::Template;
use tower_http::services::ServeDir;

// 1. Your Template Definition
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate;

// 2. The Wrapper (This makes Askama play nice with Axum)
struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", err),
            )
                .into_response(),
        }
    }
}

// 3. The Handler
async fn home() -> impl IntoResponse {
    HtmlTemplate(HomeTemplate)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home))
        .nest_service("/public", ServeDir::new("public"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running at http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
