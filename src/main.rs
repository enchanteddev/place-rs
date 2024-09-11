use std::{env::args, sync::Arc};

use axum::{response::Html, routing::get, Router};
use local_ip_address::local_ip;
use storage::{read_store, AppState};
use tokio::sync::{broadcast, RwLock};
use tower_http::compression::CompressionLayer;
mod storage;
mod views;

#[tokio::main]
async fn main() {
    let args: Vec<String> = args().collect();
    let network = !(args.len() < 2 || args[1] != "host");
    let port = {
        if args.len() < 2 {
            5000
        } else {
            let port: u32 = args[1].parse().unwrap_or_else(|_| {
                let Some(arg2) = args.get(2) else {
                    return 5000;
                };
                arg2.parse().unwrap_or(5000)
            });
            port
        }
    };
    let address = if network { "0.0.0.0" } else { "127.0.0.1" };

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", address, port))
        .await
        .unwrap();

    let base_url = if network {
        format!("http://{}:{}", local_ip().unwrap(), port)
    } else {
        format!("http://{}", listener.local_addr().unwrap())
    };

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

    // let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    //     .await
    //     .unwrap();

    // println!("listening on http://{}", listener.local_addr().unwrap());
    if network {
        println!("listening on http://{}", listener.local_addr().unwrap());
    }
    println!("listening on {}", base_url);
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("./html/index.html"))
}
