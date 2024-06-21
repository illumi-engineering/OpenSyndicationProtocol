use std::fs;
use clap::{Parser};
use log::{error, info};
use openssl::rsa::Rsa;
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

fn main() {
    let args = Args::parse();

    let key_contents = fs::read_to_string(args.private_key.clone()).expect(format!("Unable to open private key file {}", args.private_key).as_str());
    let key = Rsa::private_key_from_pem(key_contents.as_bytes()).unwrap();

    let mut conn = OutboundConnection::create_with_socket_addr(args.address.parse().unwrap(), key, args.hostname).unwrap();

    match conn.begin() {
        Ok(_) => {
            info!("Connection Finished Successfully")
        },
        Err(e) => {
            error!("Connection finished with error: {e}")
        }
    }
}