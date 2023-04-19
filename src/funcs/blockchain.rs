#![allow(dead_code)]
#[derive(Clone)]
pub struct Blockchain {
    nonce: u64,
}
impl Blockchain {
    pub fn new(nonce: u64) -> Self {
        Self { nonce }
    }
    pub fn authenticate(&self, _message: String) -> bool {
        true
    }
}