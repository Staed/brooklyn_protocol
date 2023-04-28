use std::io;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::task;
use futures::StreamExt;
use futures::stream::iter;
use std::sync::Arc;
use num_cpus;
use log;

use std::time::Instant;

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

    pub fn _add_peer(&mut self, new_addr: SocketAddr) {
        // TODO Check for isize::MAX panic
        self.peer_addrs.push(new_addr.clone())
    }

    // TODO Ping every X time and remove peer from peer_addrs if no response
    pub async fn broadcast_message(&mut self, message: Vec<u8>) -> Result<usize, Box<dyn std::error::Error>> {
        let start = Instant::now();

        let total_sent = iter(&self.peer_addrs)
            .map(|&addr| {
                let socket = Arc::clone(&self.socket);
                let msg = message.as_slice();
                async move { match socket.try_send_to(msg, addr) {
                    Ok(sent) => Ok(sent),
                    Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        log::warn!("UDP socket buffer is full; failed to send a message to {}: {}", addr, err);
                        Ok(0)
                    },
                    Err(err) => {
                        log::error!("Failed to send a message to to {}: {}", addr, err);
                        Err(err)
                    }
                }}
            })
            .buffer_unordered(num_cpus::get())
            .fold(0, |acc, sent| async move { acc + sent.unwrap_or(0) })
            .await;

        let duration = start.elapsed();
        println!("Elapsed time in broadcast after sending {:?}: {:?}", String::from_utf8(message), duration);

        self.cur_leader_idx = (self.cur_leader_idx + 1) % self.peer_addrs.len();
        Ok(total_sent)
    }

    pub async fn poll_messages(&mut self) -> Result<(usize, SocketAddr), Box <dyn std::error::Error>> {
        loop {
            let mut buffer = vec![0u8; 1024];
            let (size, peer_addr) = self.socket.recv_from(&mut buffer).await?;
            let msg = String::from_utf8_lossy(&buffer[..size]).to_string();

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
