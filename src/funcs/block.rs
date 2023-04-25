#![allow(dead_code)]

#[path = "action.rs"]
pub mod action;
pub use action::Action;

#[derive(Clone)]
pub struct Block {
    prev_hash: String,
    hash: Option<String>,
    nonce: u128,
    actions: Vec<Action>
}

impl Block {
    fn set_nonce(&mut self, nonce: u128) {
        self.nonce = nonce;
    }

    fn calculate_hash(&self) -> Vec<u8> {
        return Vec::new();
    }

    fn update_hash(&mut self) {
    }

    fn add_action(&mut self, action: Action) {
        self.actions.push(action);
        self.update_hash();
    }

    fn get_action_count(&self) -> usize {
        return self.actions.len();
    }
}