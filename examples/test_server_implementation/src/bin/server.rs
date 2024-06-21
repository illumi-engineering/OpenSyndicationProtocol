use std::net::{SocketAddr, SocketAddrV4};
use std::panic;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use clap::Parser;
use osp_server_sdk::OSProtocolNode;
use url::Url;
use log::info;
use osp_protocol::OSPUrl;

static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

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

fn main() {
    colog::init();
    let args = Args::parse();
    let addr = SocketAddrV4::new(args.bind.parse().expect("Invalid bind address"), args.port);
    let node = Arc::new(Mutex::new(
        OSProtocolNode::builder()
            .bind_to(SocketAddr::from(addr))
            .private_key_file(args.private_key)
            .hostname(args.hostname)
            .build()
    ));

    let n = Arc::clone(&node);
    GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
    std::thread::spawn(move || {
        // We need to catch panics to reliably signal exit of a thread
        let result = panic::catch_unwind(move || {
            n.lock().unwrap().listen();
        });
        // process errors
        match result {
            _ => {}
        }
        // signal thread exit
        GLOBAL_THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
    });

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

    while GLOBAL_THREAD_COUNT.load(Ordering::SeqCst) != 0 {
        std::thread::sleep(Duration::from_millis(1));
    }
}
