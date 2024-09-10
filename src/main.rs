use std::sync::{Arc, RwLock};

use axum::{routing::{get, post}, Router};
use storage::{read_store, AppState};
use tower_http::compression::CompressionLayer;
mod storage;
mod views;

#[tokio::main]
async fn main() {
    let (data, counter) = read_store();
    let state = AppState { data, counter };
    let app = Router::new()
        .route("/refresh", get(views::refresh))
        .route("/check", post(views::check))
        .layer(CompressionLayer::new())
        .with_state(Arc::new(RwLock::new(state)));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
