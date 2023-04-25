#![allow(dead_code)]

use std::time::SystemTime;

#[derive(Clone)]
pub struct Action {
    nonce: u128,
    created_at: SystemTime,
}

impl Action {
    pub fn execute() {
    }

    pub fn calculate_hash(&self) -> Vec<u8> {
        return Vec::new();
    }
}