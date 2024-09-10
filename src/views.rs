use std::sync::{Arc, RwLock};

use axum::{extract::{Query, State}, response::IntoResponse, Form};
use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;

use crate::storage::{write_store, AppState};

#[derive(Deserialize)]
pub struct CheckReq {
    pub id: usize,
    // pub check: bool
}

pub async fn refresh(State(state): State<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let data = &state.read().unwrap().data;
    let u8data = data.to_bytes();
    general_purpose::STANDARD.encode(&u8data)
}

// pub async fn check(State(state): State<Arc<RwLock<AppState>>>, Form(q): Form<CheckReq>) {
pub async fn check(q: Query<CheckReq>, State(state): State<Arc<RwLock<AppState>>>) {
    let state = &mut state.write().unwrap();
    let old_val = state.data.get(q.id).unwrap();
    state.data.set(q.id, !old_val);
    state.counter += 1;
    println!("counter = {}", state.counter);
    if state.counter % 10 == 0 {
        write_store(&state.data, state.counter);
    }
}
