use std::net::{SocketAddr, SocketAddrV4};
use std::{io};
use clap::Parser;
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
    //
    // /// Servers to open outbound connections to
    // #[arg(long)]
    // push_to: Vec<String>
}
#[tokio::main]
async fn main() -> io::Result<()> {
    let mut clog = colog::default_builder();
    clog.filter(None, log::LevelFilter::Trace);
    clog.init();

    let args = Args::parse();
    let addr = SocketAddrV4::new(args.bind.parse().expect("Invalid bind address"), args.port);
    let node = OSProtocolNode::builder()
        .bind_to(SocketAddr::from(addr))
        .private_key_file(args.private_key)
        .hostname(args.hostname)
        .build();

    node.listen().await

    // for uri in args.push_to {
    //     let osp_url = OSPUrl::from(Url::parse(uri.as_str()).unwrap());
    //     info!("url: {osp_url}");
    //     let n = Arc::clone(&node);
    //     GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
    //     std::thread::spawn(move || {
    //         // We need to catch panics to reliably signal exit of a thread
    //         let result = panic::catch_unwind(move || {
    //             info!("Starting outbound thread");
    //             n.lock().unwrap().test_outbound(osp_url);
    //         });
    //         // process errors
    //         match result {
    //             _ => {}
    //         }
    //         // signal thread exit
    //         GLOBAL_THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
    //     });
    // }

}
