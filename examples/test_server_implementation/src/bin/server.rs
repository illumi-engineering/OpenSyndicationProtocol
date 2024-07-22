use std::net::{SocketAddr, SocketAddrV4};
use std::sync::Arc;

use clap::Parser;
use log::info;

use tokio::io;
use tokio::sync::Mutex;

use url::Url;

use osp_protocol::OSPUrl;
use osp_server_sdk::Node;

/// Test implementation of an Open Syndication Protocol server node
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IPv4 address to bind to
    #[arg(short, long)]
    bind: String,

    /// TCP port to bind to
    #[arg(short, long, default_value_t = 42069)]
    port: u16,

    /// RSA Private Key for decrypting DNS challenges
    #[arg(long)]
    private_key: String,

    /// Used to identify myself during the handshake
    #[arg(long)]
    hostname: String,

    /// Servers to subscribe to data updates from
    #[arg(long)]
    subscribe_to: Vec<String>
}
#[tokio::main]
async fn main() -> io::Result<()> {
    let mut clog = colog::default_builder();
    clog.filter(None, log::LevelFilter::Trace);
    clog.init();

    let args = Args::parse();
    let addr = SocketAddrV4::new(args.bind.parse().expect("Invalid bind address"), args.port);
    let mut node = Node::new();
    node.set_addr(SocketAddr::from(addr));
    node.set_private_key_file(args.private_key).await?;
    node.set_hostname(args.hostname);


    let connection_node = Arc::new(Mutex::new(node.init().await));

    let node_rc_listen = connection_node.clone();
    tokio::spawn(async move {
        let node = node_rc_listen.lock().await;
        node.listen().await
    });


    for uri in args.subscribe_to {
        let node_rc_subscribe = connection_node.clone();
        let osp_url = OSPUrl::from(Url::parse(uri.as_str()).unwrap());
        info!("Subscribing to server: {osp_url}");
        let node = node_rc_subscribe.lock().await;
        node.clone().subscribe_to(osp_url).await?;
    }

    Ok(())
}
