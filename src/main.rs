use std::net::SocketAddr;
use std::thread;
use std::time::Duration;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

mod funcs;
use funcs::node::Node;
use funcs::node::Blockchain;

fn generate_addr(base_addr: &str, port: i32) -> SocketAddr {
    return (base_addr.to_owned() + &port.to_string()).parse().unwrap();
}

fn generate_random_message() -> Vec<u8> {
    let msg: String = thread_rng().sample_iter(&Alphanumeric).take(20).map(char::from).collect();
    return msg.into_bytes();
}

async fn generate_nodes(base_addr: &str, base_port: i32, blockchain: Blockchain, num_nodes: i32) -> Vec<Node> {
    // TODO An assert here to make explicit that this is for k-groupings where k ∈ ℕ, k < 10
    let nodes_futures = (0..num_nodes).map(|i| {
        let blockchain_tmp = blockchain.clone();
        async move {
            let peers: Vec<SocketAddr> = (0..num_nodes).filter(|&x| x != i)
                                                       .map(|x| generate_addr(base_addr, base_port+x)).collect();
            let addr = generate_addr(base_addr, base_port + i);

            return Node::new(peers, addr, blockchain_tmp, 0).await;
        }
        })
    .collect::<Vec<_>>();

    return futures::future::try_join_all(nodes_futures).await.unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_addr = "127.0.0.1:";
    let base_port = 9000;
    let blockchain: Blockchain = Blockchain::new(835);
    let n = 5;

    let mut nodes: Vec<Node> = generate_nodes(base_addr, base_port, blockchain, n).await;

    thread::sleep(Duration::from_millis(500));

    println!("Node1 will begin broadcasting to its peers which number {}", n-1);
    let node1 = &mut nodes[0];
    for _ in std::iter::repeat(()).take(5) {
        node1.broadcast_message(generate_random_message()).await?;

        thread::sleep(Duration::from_millis(250));
    }

    Ok(())
}