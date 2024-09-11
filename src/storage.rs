use std::fs::{read_to_string, write};

use base64::{engine::general_purpose, Engine as _};
use bit_vec::BitVec;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub data: BitVec,
    pub counter: u64,
    pub tx: broadcast::Sender<String>
}

pub fn write_store(data: &BitVec, counter: u64) {
    let u8data = data.to_bytes();
    let b64 = general_purpose::STANDARD.encode(&u8data);
    write("./.store", format!("{counter}\n{b64}")).unwrap();
}

pub fn read_store() -> (BitVec, u64) {
    let Ok(store) = read_to_string("./.store") else {
        return (BitVec::from_elem(10000, false), 0);
    };
    let (counter, data) = store
        .split_once("\n")
        .expect(&format!("No newlines in the store = '{store}'"));

    let u8data = general_purpose::STANDARD
        .decode(data)
        .expect(&format!("Invalid base64 = '{data}'"));

    let data = BitVec::from_bytes(&u8data);
    let counter: u64 = counter
        .parse()
        .expect(&format!("Not a u64, counter = {counter}"));

    (data, counter)
}
