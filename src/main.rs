use axum::{Router, routing::get};
#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(async || "Hello, World!"));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
        .await
        .unwrap();
    println!("Axum server running on http://127.0.0.1:4000");
    axum::serve(listener, app).await.unwrap();
}
