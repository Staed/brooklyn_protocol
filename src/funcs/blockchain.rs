#![allow(dead_code)]

pub mod block;
pub use block::Block;
use block::Action;

#[derive(Clone)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pending_actions: Vec<Action>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain { blocks: Vec::new(), pending_actions: Vec::new() }
    }
    pub fn authenticate(&self, _message: &String) -> bool {
        true
    }

    pub fn append_block(&mut self, _block: Block) {
    }
}