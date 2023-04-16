#![allow(dead_code)]
use std::io;
use std::net::{SocketAddr, UdpSocket};
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
    pub fn new(peer_addrs: SocketAddr, self_addr: SocketAddr, blockchain: Blockchain) -> io::Result<Node> {
        let socket = UdpSocket::bind(self_addr)?;
        Ok(Node { peer_addrs, socket, blockchain })
    }

    pub fn send_message(&self, message: &[u8]) -> io::Result<usize> {
        let size = self.socket.send_to(message, self.peer_addrs);
        eprintln!("Sent {} bytes to {}", message.len(), self.peer_addrs);
        return size
    }

    pub fn poll_messages(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        loop {
            match self.socket.recv_from(buf) {
                Ok((size, peer_addr)) => {
                    let msg = String::from_utf8_lossy(&buf[..size]).to_string();

                    if self.blockchain.authenticate(msg.clone()) {
                        eprintln!("Received from {}: {}", peer_addr, &msg);
                    } else {
                        eprintln!("Received any invalid message from {}", peer_addr);
                    }
                    return Ok((size, peer_addr));
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(err) => return Err(err),
            }
        }
    }
}

fn main() -> io::Result<()> {
    let node1_addr = "127.0.0.1:9000".parse().unwrap();
    let node2_addr = "127.0.0.1:9001".parse().unwrap();
    let blockchain = Blockchain::new(835);


    let node1 = Node::new(node2_addr, node1_addr, blockchain.clone())?;
    let node2 = Node::new(node1_addr, node2_addr, blockchain.clone())?;

    thread::sleep(Duration::from_secs(1));
    let message = "Hello world".as_bytes();
    node1.send_message(message)?;

    let mut buffer = [0u8; 1024];
    let (size, addr) = node2.poll_messages(&mut buffer)?;

    println!("Node2 received {} bytes from {}: {:?}", size, addr, &buffer[..size]);

    Ok(())
}