use std::sync::Arc;

use axum::{response::Html, routing::get, Router};
use storage::{read_store, AppState};
use tokio::sync::{broadcast, RwLock};
use tower_http::compression::CompressionLayer;
mod storage;
mod views;

#[tokio::main]
async fn main() {
    let (data, counter) = read_store();
    let (tx, _rx) = broadcast::channel(100);
    let state = AppState { data, counter, tx };
    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(views::websocket_handler))
        .route("/refresh", get(views::refresh))
        .route("/check", get(views::check))
        .layer(CompressionLayer::new())
        .with_state(Arc::new(RwLock::new(state)));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("./html/index.html"))
}
