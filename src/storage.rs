use std::{fs::{read_to_string, write}, u8};

use base64::{engine::general_purpose, Engine as _};
use bit_vec::BitVec;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub data: Vec<u8>,
    pub counter: u64,
    pub tx: broadcast::Sender<String>
}

pub fn write_store(data: &Vec<u8>, counter: u64) {
    let b64 = general_purpose::STANDARD.encode(&data);
    write("./.store", format!("{counter}\n{b64}")).unwrap();
}

pub fn read_store() -> (Vec<u8>, u64) {
    let Ok(store) = read_to_string("./.store") else {
        return (vec![u8::MAX; 5000], 0);
    };
    let (counter, data) = store
        .split_once("\n")
        .expect(&format!("No newlines in the store = '{store}'"));

    let data = general_purpose::STANDARD
        .decode(data)
        .expect(&format!("Invalid base64 = '{data}'"));
    
    let counter: u64 = counter
        .parse()
        .expect(&format!("Not a u64, counter = {counter}"));

    (data, counter)
}
