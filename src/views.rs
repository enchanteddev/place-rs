use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use base64::{engine::general_purpose, Engine as _};
use futures::{stream::StreamExt, SinkExt};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::storage::{write_store, AppState};

#[derive(Deserialize)]
pub struct CheckReq {
    pub id: usize,
    // pub check: bool
}

pub async fn refresh(State(state): State<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let data = &state.read().await.data;
    let u8data = data.to_bytes();
    general_purpose::STANDARD.encode(&u8data)
}

// pub async fn check(State(state): State<Arc<RwLock<AppState>>>, Form(q): Form<CheckReq>) {
pub async fn check(q: Query<CheckReq>, State(state): State<Arc<RwLock<AppState>>>) {
    let state = &mut state.write().await;
    let old_val = state.data.get(q.id).unwrap();
    state.data.set(q.id, !old_val);
    state.counter += 1;
    println!("counter = {}", state.counter);
    if state.counter % 10 == 0 {
        write_store(&state.data, state.counter);
    }
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| ws_transmitter(socket, state))
}

pub async fn ws_transmitter(socket: WebSocket, state: Arc<RwLock<AppState>>) {
    let (mut sender, mut receiver) = socket.split();

    let read_state = state.read().await;
    let mut rx = read_state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            println!("send_task: {msg}");
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = read_state.tx.clone();
    drop(read_state);
    let nstate = state.clone();
    // This second task will receive messages from client and print them on server console
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let write_state = &mut nstate.write().await;

            let index: usize = text.parse().unwrap();
            let old_val = write_state.data.get(index).unwrap();
            println!("index = {index}, set_check = {}", !old_val);

            write_state.counter += 1;
            println!("counter = {}", write_state.counter);

            write_state.data.set(index, !old_val);
            if write_state.counter % 10 == 0 {
                write_store(&write_state.data, write_state.counter);
            }
            
            let _ = tx.send(format!("{text} {} {}", !old_val, tx.receiver_count()));
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };
}
