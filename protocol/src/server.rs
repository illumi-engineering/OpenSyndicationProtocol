use std::{collections::HashMap, net::{SocketAddr, TcpListener, TcpStream}, sync::{Arc, Mutex, RwLock}};

use crate::{OSPHandshakeIn, OSPHandshakeOut, Protocol};

#[derive(Clone)]
pub struct OSProtocolNode {
    inbound_conn_id: Arc<RwLock<u32>>,
    outbound_conn_id: Arc<RwLock<u32>>,
    bind_addr: SocketAddr,
    inbound_connections: Arc<Mutex<HashMap<u32, InboundConnectionContext>>>,
    outbound_connections: Arc<Mutex<HashMap<u32, OutboundConnectionContext>>>
}

struct InboundConnectionContext {
    protocol: Protocol
}

struct OutboundConnectionContext {
    protocol: Protocol,
}

impl OSProtocolNode {
    fn create(bind_addr: SocketAddr) -> Self {
        Self { 
            inbound_conn_id: Arc::new(RwLock::new(0)),
            outbound_conn_id: Arc::new(RwLock::new(0)),
            bind_addr,
            inbound_connections: Arc::new(Mutex::new(HashMap::new())),
            outbound_connections: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    fn start_server(self) {
        let listener = TcpListener::bind(self.bind_addr).unwrap();
        println!("listening started, ready to accept");
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                println!("Accepting a new connection from {}", 
                    stream
                        .peer_addr()
                        .map(|addr| addr.to_string())
                        .unwrap_or("unknown address".to_string())
                );

                self.clone().start_connection(stream).map_err(|e| eprintln!("[osp] err: {}", e)).unwrap();
            }
        }
    }

    fn start_connection(self, stream: TcpStream) -> std::io::Result<()> {
        let protocol = Protocol::with_stream(stream)?;
    
        let me = self.clone();
        std::thread::spawn({
            let ctx = InboundConnectionContext {
                protocol
            };

            let id = me.inbound_conn_id.read().unwrap();
            me.inbound_connections.lock().unwrap().insert(*id, ctx);
            let new_id = id.clone();

            let mut next_id = self.inbound_conn_id.write().unwrap();
            *next_id += 1;

            move || {
                let _ = handle_inbound_connection(me.inbound_connections.lock().unwrap().get_mut(&new_id).expect(""))
                    .map_err(|e| eprintln!("[osp] err: {}", e));
            }
        });
    
        Ok(())
    }

    fn start_syndication_from(self, addr: SocketAddr) {
        let me = self.clone();

        std::thread::spawn({
            let protocol = Protocol::connect(addr)
                .map_err(|e| eprintln!("[osp] err: {}", e))
                .unwrap();
            let ctx = OutboundConnectionContext {
                protocol,
            };

            let id = me.outbound_conn_id.read().unwrap();
            me.outbound_connections.lock().unwrap().insert(*id, ctx);
            let new_id = id.clone();

            let mut next_id = self.outbound_conn_id.write().unwrap();
            *next_id += 1;

            move || {

            }
        });
    }
}

fn switch_handshake_in(req: OSPHandshakeIn) -> OSPHandshakeOut {
    match req {
        OSPHandshakeIn::Hello {} => {
            OSPHandshakeOut::Acknowledge { ok: true, err: None }
        },
    }
}

fn handle_inbound_connection(ctx: &mut InboundConnectionContext) -> std::io::Result<()> {
    let request = ctx.protocol.read_message::<OSPHandshakeIn>()?;
    let resp = switch_handshake_in(request);
    ctx.protocol.send_message(&resp)
}

fn handle_outbound_connection(ctx: &mut OutboundConnectionContext) -> std::io::Result<()> {
    ctx.protocol.send_message(&OSPHandshakeIn::Hello {  });

    if let OSPHandshakeOut::Acknowledge { ok, err } = ctx.protocol.read_message::<OSPHandshakeOut>()? {
        if ok {
            // todo: next steps
        } else {
            match err {
                Some(msg) => {

                }
                None => {

                }
            }
        }
    } else {
        // todo: error for invalid response
    }
    // todo

    Ok(())
}