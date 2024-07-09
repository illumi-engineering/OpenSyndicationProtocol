use std::fs;

use clap::Parser;

use log::info;

use openssl::rsa::Rsa;

use tokio::io;

use url::Url;

use osp_protocol::OSPUrl;
use osp_server_sdk::connection::outbound::OutboundConnection;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IPv4 address to bind to
    #[arg()]
    url: String,

    /// RSA Private Key for decrypting DNS challenges
    #[arg(long)]
    private_key: String,

    /// Used to identify myself during the handshake
    #[arg(long)]
    hostname: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut clog = colog::default_builder();
    clog.filter(None, log::LevelFilter::Trace);
    clog.init();

    let args = Args::parse();

    let key_contents = fs::read_to_string(args.private_key.clone()).expect(format!("Unable to open private key file {}", args.private_key).as_str());
    let key = Rsa::private_key_from_pem(key_contents.as_bytes()).unwrap();

    let reg_url = Url::parse(args.url.as_str()).unwrap();
    let url = OSPUrl::from(reg_url);

    info!("Starting outbound thread");
    let mut conn = OutboundConnection::create(url, key, args.hostname).await?;
    let mut conn_in_handshake = conn.begin().await?;
    conn_in_handshake.handshake().await
}