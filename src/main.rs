#![allow(dead_code)]
use std::io;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
struct Blockchain {
    nonce: u64,
}

impl Blockchain {
    fn new(nonce: u64) -> Self {
        Self { nonce }
    }

    fn authenticate(&self, _message: String) -> bool {
        true
    }
}

struct Node {
    peer_addrs: SocketAddr,
    socket: UdpSocket,
    blockchain: Blockchain,
}

impl Node {
    pub async fn new(peer_addrs: SocketAddr, self_addr: SocketAddr, blockchain: Blockchain) -> io::Result<Node> {
        let socket = UdpSocket::bind(self_addr).await?;
        Ok(Node { peer_addrs, socket, blockchain })
    }

    pub async fn send_message(&self, message: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        let size = self.socket.send_to(message, self.peer_addrs).await?;
        eprintln!("Sent {} bytes to {}", message.len(), self.peer_addrs);
        Ok(size)
    }

    pub async fn poll_messages(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), Box <dyn std::error::Error>> {
        loop {
            let (size, peer_addr) = self.socket.recv_from(buf).await?;
            let msg = String::from_utf8_lossy(&buf[..size]).to_string();

            if self.blockchain.authenticate(msg.clone()) {
                eprintln!("Received from {}: {}", peer_addr, &msg);
            } else {
                eprintln!("Received any invalid message from {}", peer_addr);
            }
            return Ok((size, peer_addr));
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node1_addr = "127.0.0.1:9000".parse().unwrap();
    let node2_addr = "127.0.0.1:9001".parse().unwrap();
    let blockchain = Blockchain::new(835);


    let node1 = Node::new(node2_addr, node1_addr, blockchain.clone()).await?;
    let node2 = Node::new(node1_addr, node2_addr, blockchain.clone()).await?;

    thread::sleep(Duration::from_secs(1));
    let message = "Hello world".as_bytes();
    node1.send_message(message).await?;

    let mut buffer = [0u8; 1024];
    let (size, addr) = node2.poll_messages(&mut buffer).await?;

    println!("Node2 received {} bytes from {}: {:?}", size, addr, &buffer[..size]);

    Ok(())
}