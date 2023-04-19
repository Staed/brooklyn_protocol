use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

mod funcs;
use funcs::node::Node;
use funcs::node::Blockchain;

fn generate_addr(base_addr: &str, port: i32) -> SocketAddr {
    return (base_addr.to_owned() + &port.to_string()).parse().unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_addr = "127.0.0.1:";
    let mut initial_port = 9000;

    // TODO Node initialization into a loop
    let node1_addr: SocketAddr = generate_addr(base_addr, initial_port);
    initial_port += 1;
    let node2_addr: SocketAddr = generate_addr(base_addr, initial_port);
    initial_port += 1;
    let node3_addr: SocketAddr = generate_addr(base_addr, initial_port);
    initial_port += 1;
    let node4_addr: SocketAddr = generate_addr(base_addr, initial_port);
    let blockchain: Blockchain = Blockchain::new(835);

    let mut node1_peers: Vec<SocketAddr> = Vec::new();
    let mut node2_peers: Vec<SocketAddr> = Vec::new();
    let mut node3_peers: Vec<SocketAddr> = Vec::new();
    let node4_peers: Vec<SocketAddr> = Vec::new();

    // TODO Find a way to not have to clone these addresses constantly
    node1_peers.push(node2_addr.clone());
    node1_peers.push(node3_addr.clone());
    node1_peers.push(node4_addr.clone());

    node2_peers.push(node1_addr.clone());
    node2_peers.push(node3_addr.clone());
    node2_peers.push(node4_addr.clone());

    node3_peers.push(node1_addr.clone());
    node3_peers.push(node2_addr.clone());
    node3_peers.push(node4_addr.clone());

    let mut node1 = Node::new(node1_peers, node1_addr, blockchain.clone(), 0).await?;
    let mut node2 = Node::new(node2_peers, node2_addr, blockchain.clone(), 0).await?;
    let mut node3 = Node::new(node3_peers, node3_addr, blockchain.clone(), 0).await?;

    let mut node4 = Node::new(node4_peers, node4_addr, blockchain.clone(), 0).await?;
    node4.add_peer(node1_addr.clone());
    node4.add_peer(node2_addr.clone());
    node4.add_peer(node3_addr.clone());

    thread::sleep(Duration::from_secs(1));
    let message = "Hello world".as_bytes();
    node1.broadcast_message(message).await?;

    let mut buffer_2 = [0u8; 1024];
    let node2_future = node2.poll_messages(&mut buffer_2);
    let mut buffer_3 = [0u8; 1024];
    let node3_future = node3.poll_messages(&mut buffer_3);
    let mut buffer_4 = [0u8; 1024];
    let node4_future = node4.poll_messages(&mut buffer_4);

    let (size_2, addr_2) = node2_future.await?;
    let (size_3, addr_3) = node3_future.await?;
    let (size_4, addr_4) = node4_future.await?;
    println!("Node2 received {} bytes from {}: {:?}", size_2, addr_2, &buffer_2[..size_2]);
    println!("Node3 received {} bytes from {}: {:?}", size_3, addr_3, &buffer_3[..size_3]);
    println!("Node4 received {} bytes from {}: {:?}", size_4, addr_4, &buffer_4[..size_4]);

    Ok(())
}