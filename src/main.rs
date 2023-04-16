use std::net::{SocketAddr, UdpSocket};
use std::io;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
enum Message {
    Login(String),
    Logout(String)
}

#[derive(Clone)]
struct Blockchain {
    nonce: u64,
}

impl Blockchain {
    fn new(nonce: u64) -> Self {
        Self { nonce: nonce, }
    }

    fn authenticate(&self, message: &[u8]) -> bool {
        true
    }
}

struct Node {
    id: u64,
    addr: SocketAddr,
    blockchain: Blockchain,
}

impl Node {
    fn new(id: u64, addr: SocketAddr, blockchain: Blockchain) -> Self {
        Node { id, addr, blockchain }
    }

    fn send_message(&self, message: Message) -> io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        let serialized_message = serde_json::to_string(&message).unwrap();
        socket.send_to(serialized_message.as_bytes(), &self.addr)?;
        Ok(())
    }

    fn receive_message(&self) -> io::Result<Option<Message>> {
        let socket = UdpSocket::bind(self.addr)?;
        socket.set_nonblocking(true)?;

        let mut buffer = [0u8, 255];
        match socket.recv_from(&mut buffer) {
            Ok((bytes_read, _)) => {
                let message = &buffer[..bytes_read];
                if self.blockchain.authenticate(message) {
                    let deserialized_message: Message = serde_json::from_slice(message).unwrap();
                    Ok(Some(deserialized_message))
                } else {
                    println!("Received an invalid message from {}", self.addr);
                    Ok(None)
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}

fn main() -> io::Result<()> {
    let blockchain: Blockchain = Blockchain::new(835);
    let node1_address: SocketAddr = "127.0.0.1:9001".parse().unwrap();
    let node2_address: SocketAddr = "127.0.0.1:9002".parse().unwrap();

    let node1 = Node::new(1, node2_address, blockchain.clone());
    let node2 = Node::new(2, node1_address, blockchain.clone());

    let message = Message::Login("Hello world".to_owned());
    node1.send_message(message.clone())?;

    loop {
        if let Some(received_message) = node2.receive_message()? {
            match received_message {
                Message::Login(text) => {
                    println!("Received login message: {}", text);
                }
                Message::Logout(text) => {
                    println!("Received logout message: {}", text);
                }
            }
            break;
        }
    }

    Ok(())
}