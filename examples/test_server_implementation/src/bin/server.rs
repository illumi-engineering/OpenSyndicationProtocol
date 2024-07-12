use std::net::{SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};

use clap::Parser;
use log::info;

use tokio::io;

use url::Url;

use osp_protocol::OSPUrl;
use osp_server_sdk::OSProtocolNode;

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
    let mut node = OSProtocolNode::new();
    node.set_addr(SocketAddr::from(addr));
    node.set_private_key_file(args.private_key);
    node.set_hostname(args.hostname);

    let mut connection_node = Arc::new(Mutex::new(node.init()));

    let node_rc_listen = connection_node.clone();
    tokio::spawn(async move {
        let mut node = node_rc_listen.lock().unwrap().to_owned();
        node.listen(|connection| async move {

            Ok(())
        }).await
    });


    let node_rc_subscribe = connection_node.clone();
    for uri in args.subscribe_to {
        let osp_url = OSPUrl::from(Url::parse(uri.as_str()).unwrap());
        info!("Subscribing to server: {osp_url}");
        let mut node = node_rc_subscribe.lock().unwrap().to_owned();
        node.subscribe_to(osp_url).await?;
    }

    Ok(())
}
