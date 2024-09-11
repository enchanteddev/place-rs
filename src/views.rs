use std::{sync::Arc};

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

#[derive(Deserialize, Clone, Copy, Debug)]
pub enum Color {
    RED,
    ORANGE,
    YELLOW,
    GREEN,
    LGREEN,
    BLUE,
    LBLUE,
    CYAN,
    PURPLE,
    LPURPLE,
    PINK,
    BROWN,
    BLACK,
    GRAY,
    LGRAY,
    WHITE,
}

impl TryFrom<&str> for Color {
    fn try_from(value: &str) -> Result<Self, ()> {
        let value = value.to_uppercase();
        let value = value.as_str();
        match value {
            "RED" => Ok(Self::RED),
            "ORANGE" => Ok(Self::ORANGE),
            "YELLOW" => Ok(Self::YELLOW),
            "GREEN" => Ok(Self::GREEN),
            "LGREEN" => Ok(Self::LGREEN),
            "BLUE" => Ok(Self::BLUE),
            "LBLUE" => Ok(Self::LBLUE),
            "CYAN" => Ok(Self::CYAN),
            "PURPLE" => Ok(Self::PURPLE),
            "LPURPLE" => Ok(Self::LPURPLE),
            "PINK" => Ok(Self::PINK),
            "BROWN" => Ok(Self::BROWN),
            "BLACK" => Ok(Self::BLACK),
            "GRAY" => Ok(Self::GRAY),
            "LGRAY" => Ok(Self::LGRAY),
            "WHITE" => Ok(Self::WHITE),
            _ => Err(())
        }
    }
    
    type Error = ();
}

#[derive(Deserialize)]
pub struct CheckReq {
    pub id: usize,
    pub color: Color,
}

pub async fn refresh(State(state): State<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let data = &state.read().await.data;
    general_purpose::STANDARD.encode(&data)
}

// pub async fn check(State(state): State<Arc<RwLock<AppState>>>, Form(q): Form<CheckReq>) {
pub async fn check(q: Query<CheckReq>, State(state): State<Arc<RwLock<AppState>>>) {
    let state = &mut state.write().await;
    let old = state.data[q.id / 2];
    let newval = if q.id % 2 == 0 {
        let last4 = (old << 4) >> 4;
        let first4 = q.color as u8;
        first4 << 4 + last4
    } else {
        let first4 = old >> 4;
        let last4 = q.color as u8;
        first4 << 4 + last4
    };
    state.data[q.id / 2] = newval;
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

            let (index_str, color_str) = text.split_once(" ").unwrap();

            let index: usize = index_str.parse().unwrap();
            let color: Color = color_str.try_into().unwrap();
            println!("index = {index}, set_check = {color:?}");

            write_state.counter += 1;
            println!("counter = {}", write_state.counter);
            let old = write_state.data[index / 2];
            let newval = if index % 2 == 0 {
                let last4 = (old << 4) >> 4;
                let first4 = color as u8;
                println!("color as u8 = {first4}, last4 = {last4}");
                (first4 << 4) + last4
            } else {
                let first4 = old >> 4;
                let last4 = color as u8;
                println!("first4 = {first4}, color as u8 = {last4}");
                (first4 << 4) + last4
            };
            write_state.data[index / 2] = newval;
            if write_state.counter % 10 == 0 {
                write_store(&write_state.data, write_state.counter);
            }

            let _ = tx.send(format!("{text} {}", tx.receiver_count()));
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };
}
