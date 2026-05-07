mod handlers;
mod markdown;
mod models;
mod routes;
mod templates;

#[tokio::main]
async fn main() {
    let app = routes::app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running at http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
