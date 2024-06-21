use std::{fs, panic};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use clap::{Parser};
use log::{error, info};
use openssl::rsa::Rsa;
use osp_server_sdk::connection::outbound::OutboundConnection;


static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IPv4 address to bind to
    #[arg()]
    address: String,

    /// RSA Private Key for decrypting DNS challenges
    #[arg(long)]
    private_key: String,

    /// Used to identify myself during the handshake
    #[arg(long)]
    hostname: String,
}

fn main() {
    let args = Args::parse();

    let key_contents = fs::read_to_string(args.private_key.clone()).expect(format!("Unable to open private key file {}", args.private_key).as_str());
    let key = Rsa::private_key_from_pem(key_contents.as_bytes())?;

    GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
    std::thread::spawn(move || {
        // We need to catch panics to reliably signal exit of a thread
        let result = panic::catch_unwind(move || {
            info!("Starting outbound thread");
            let mut conn = OutboundConnection::create_with_socket_addr(args.address.parse().unwrap(), key, args.hostname)?;
            let mut conn_in_handshake = conn.begin()?;
            conn_in_handshake.handshake()
        });
        // process errors
        match result {
            _ => {}
        }
        // signal thread exit
        GLOBAL_THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
    });

    while GLOBAL_THREAD_COUNT.load(Ordering::SeqCst) != 0 {
        std::thread::sleep(Duration::from_millis(1));
    }
}