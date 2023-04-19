use std::io;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::task;
use std::sync::Arc;

#[path = "blockchain.rs"]
pub mod blockchain;
pub use blockchain::Blockchain;

pub struct Node {
    peer_addrs: Vec<SocketAddr>,
    socket: Arc<UdpSocket>,
    blockchain: Blockchain,

    // TODO Find a way to allow this to be mut without making the whole struct mut
    cur_leader_idx: usize,
}
impl Node {
    pub async fn new(peer_addrs: Vec<SocketAddr>, self_addr: SocketAddr, blockchain: Blockchain, cur_leader_idx: usize) -> io::Result<Node> {
        let socket = Arc::new(UdpSocket::bind(self_addr).await?);
        Ok(Node { peer_addrs, socket, blockchain, cur_leader_idx })
    }
    pub fn add_peer(&mut self, new_addr: SocketAddr) {
        // TODO Check for isize::MAX panic
        self.peer_addrs.push(new_addr.clone())
    }
    // TODO Ping every X time and remove peer from peer_addrs if no response
    pub async fn broadcast_message(&mut self, message: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        // TODO Parallelize this
        let mut total_sent = 0;
        for addr in self.peer_addrs.iter() {
            let size = self.socket.send_to(message, addr).await?;
            eprintln!("Sent {} bytes to {}", message.len(), addr);
            total_sent += size;
        }

        // Rotate leader after broadcasting
        if self.cur_leader_idx < self.peer_addrs.len() {
            self.cur_leader_idx += 1
        } else {
            self.cur_leader_idx = 0;
        }

        Ok(total_sent)
    }
    pub async fn poll_messages(&mut self, buf: &mut [u8]) -> Result<(usize, SocketAddr), Box <dyn std::error::Error>> {
        loop {
            let (size, peer_addr) = self.socket.recv_from(buf).await?;
            let msg = String::from_utf8_lossy(&buf[..size]).to_string();

            if msg == "Ok" {
                return Ok((0, peer_addr));
            }

            if self.blockchain.authenticate(msg.clone()) {
                eprintln!("Received from {}: {}", peer_addr, &msg);

                self.acknowledge(peer_addr.clone());
            } else {
                eprintln!("Received an invalid message from {}", peer_addr);
            }
            return Ok((size, peer_addr));
        }
    }

    // Use to confirm the hash + leader information received via broadcast looks good
    fn acknowledge(&self, original_sender: SocketAddr) -> () {
        let sock = Arc::clone(&self.socket);
        task::spawn(async move {
            let _ = sock.send_to("Ok".as_bytes(), original_sender).await;
        });
    }
}
