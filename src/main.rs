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
    peer_addrs: Vec<SocketAddr>,
    socket: UdpSocket,
    blockchain: Blockchain,
}

impl Node {
    pub async fn new(peer_addrs: Vec<SocketAddr>, self_addr: SocketAddr, blockchain: Blockchain) -> io::Result<Node> {
        let socket = UdpSocket::bind(self_addr).await?;
        Ok(Node { peer_addrs, socket, blockchain })
    }

    pub fn add_peer(mut self, new_addr: SocketAddr) {
        // TODO Check for isize::MAX panic
        self.peer_addrs.push(new_addr.clone())
    }

    // TODO Ping every X time and remove peer from peer_addrs if no response

    pub async fn broadcast_message(&self, message: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        // TODO Parallelize this
        let mut total_sent = 0;
        for addr in self.peer_addrs.iter() {
            let size = self.socket.send_to(message, addr).await?;
            eprintln!("Sent {} bytes to {}", message.len(), addr);
            total_sent += size;
        }
        Ok(total_sent)
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
    let node1_addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    let node2_addr: SocketAddr = "127.0.0.1:9001".parse().unwrap();
    let node3_addr: SocketAddr = "127.0.0.1:9002".parse().unwrap();
    let blockchain: Blockchain = Blockchain::new(835);

    let mut node1_peers: Vec<SocketAddr> = Vec::new();
    let mut node2_peers: Vec<SocketAddr> = Vec::new();
    let mut node3_peers: Vec<SocketAddr> = Vec::new();

    // TODO Find a way to not have to clone these addresses constantly
    node1_peers.push(node2_addr.clone());
    node1_peers.push(node3_addr.clone());

    node2_peers.push(node1_addr.clone());
    node2_peers.push(node3_addr.clone());

    node3_peers.push(node1_addr.clone());
    node3_peers.push(node2_addr.clone());

    let node1 = Node::new(node1_peers, node1_addr, blockchain.clone()).await?;
    let node2 = Node::new(node2_peers, node2_addr, blockchain.clone()).await?;
    let node3 = Node::new(node3_peers, node3_addr, blockchain.clone()).await?;

    thread::sleep(Duration::from_secs(1));
    let message = "Hello world".as_bytes();
    node1.broadcast_message(message).await?;

    let mut buffer_2 = [0u8; 1024];
    let node2_future = node2.poll_messages(&mut buffer_2);
    let mut buffer_3 = [0u8; 1024];
    let node3_future = node3.poll_messages(&mut buffer_3);

    let (size_2, addr_2) = node2_future.await?;
    let (size_3, addr_3) = node3_future.await?;
    println!("Node2 received {} bytes from {}: {:?}", size_2, addr_2, &buffer_2[..size_2]);
    println!("Node3 received {} bytes from {}: {:?}", size_3, addr_3, &buffer_3[..size_3]);

    Ok(())
}