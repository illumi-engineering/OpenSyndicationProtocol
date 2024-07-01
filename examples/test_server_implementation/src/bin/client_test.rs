use std::{fs};


use clap::{Parser};
use log::{info};
use openssl::rsa::Rsa;
use tokio::io;
use osp_server_sdk::connection::outbound::OutboundConnection;

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

#[tokio::main]
async fn main() -> io::Result<()> {
    colog::init();
    let args = Args::parse();

    let key_contents = fs::read_to_string(args.private_key.clone()).expect(format!("Unable to open private key file {}", args.private_key).as_str());
    let key = Rsa::private_key_from_pem(key_contents.as_bytes()).unwrap();

    info!("Starting outbound thread");
    let mut conn = OutboundConnection::create_with_socket_addr(args.address.parse().unwrap(), key, args.hostname)?;
    let mut conn_in_handshake = conn.begin().await?;
    conn_in_handshake.handshake().await
}