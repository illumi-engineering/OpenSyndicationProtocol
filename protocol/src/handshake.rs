use std::io::{self, Read, Write};

use edcert::certificate::Certificate;
use byteorder::{WriteBytesExt, ReadBytesExt};

use super::utils::{DeserializePacket, SerializePacket};

pub enum ConnectionType {
    Unknown = 0,
    Client = 1,
    Server = 2
}

impl ConnectionType {
    fn from_u8(t: u8) -> io::Result<ConnectionType> {
        match t {
            1 => Ok(ConnectionType::Client),
            2 => Ok(ConnectionType::Server),
            _ => io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Connection Type"
            )
        }
    }
}

impl From<&ConnectionType> for u8 {
    fn from(t: &ConnectionType) -> Self {
        match t {
            ConnectionType::Client => 1,
            ConnectionType::Server => 2,
        }
    }
}

pub enum OSPHandshakeIn {
    Hello {
        connection_type: ConnectionType,
    },
    Login {},
}

impl From<&OSPHandshakeIn> for u8 {
    fn from(req: &OSPHandshakeIn) -> Self {
        match req {
            OSPHandshakeIn::Hello { .. } => 1,
            OSPHandshakeIn::Login { .. } => 2,
        }
    }
}

impl SerializePacket for OSPHandshakeIn {
    /// Serialize Request to bytes (to send to server)
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize> {
        buf.write_u8(self.into())?; // Message Type byte
        let mut bytes_written: usize = 1;
        match self {
            OSPHandshakeIn::Hello { connection_type } => {
                buf.write_u8(u8::from(connection_type))?;
                bytes_written += 1
            }
            OSPHandshakeIn::Login {  } => {

            }
            // OSPHandshake::SyncProject { root_dir } => {
            //     // Write the variable length message string, preceded by it's length
            //     let root_dir = root_dir.as_bytes();
            //     buf.write_u16::<NetworkEndian>(root_dir.len() as u16)?;
            //     buf.write_all(&root_dir)?;
            //     bytes_written += 2 + root_dir.len()
            // }
            // OSPHandshake::InstallProject { project_dir, workspace } => {
            //     // Write the variable length message string, preceded by it's length
            //     let project_dir = project_dir.as_bytes();
            //     buf.write_u16::<NetworkEndian>(project_dir.len() as u16)?;
            //     buf.write_all(&project_dir)?;
            //     bytes_written += 2 + project_dir.len();

            //     buf.write_i8(*workspace as i8)?;
            //     bytes_written += 1;
            // }
        }
        Ok(bytes_written)
    }
}

impl DeserializePacket for OSPHandshakeIn {
    type Output = OSPHandshakeIn;

    fn deserialize(mut buf: &mut impl Read) -> io::Result<OSPHandshakeIn> {
        // We'll match the same `u8` that is used to recognize which request type this is
        match buf.read_u8()? {
            1 => Ok(OSPHandshakeIn::Hello {
                connection_type: ConnectionType::from_u8(buf.read_u8()?)?
            }),
            2 => Ok(OSPHandshakeIn::Login {}),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request Type",
            )),
        }
    }
}

pub enum OSPHandshakeOut {
    Acknowledge {
        ok: bool,
        require_login: bool,
        err: Option<String>,
    },
    LoginResponse {
        ok: bool,
        err: Option<String>,
    }
}

impl From<&OSPHandshakeOut> for u8 {
    fn from(req: &OSPHandshakeOut) -> Self {
        match req {
            OSPHandshakeOut::Acknowledge { .. } => 1,
            OSPHandshakeOut::LoginResponse { .. } => 2,
        }
    }
}

impl SerializePacket for OSPHandshakeOut {
    /// Serialize Response to bytes (to send to client)
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize> {
        buf.write_u8(self.into())?; // Message Type byte
        let mut bytes_written: usize = 1;
        match self {
            OSPHandshakeOut::Acknowledge { ok, require_login, err } => {
                buf.write_i8(*ok as i8)?;
                bytes_written += 1;

                buf.write_i8(*require_login as i8)?;
                bytes_written += 1;

                self.write_optional_string(buf, err);
            }
            OSPHandshakeOut::LoginResponse { ok, err } => {
                buf.write_i8(*ok as i8)?;
                bytes_written += 1;

                self.write_optional_string(buf, err);
            }
        }
        Ok(bytes_written)
    }
}

impl DeserializePacket for OSPHandshakeOut {
    type Output = OSPHandshakeOut;

    fn deserialize(buf: &mut impl Read) -> io::Result<OSPHandshakeOut> {
        // We'll match the same `u8` that is used to recognize which response type this is
        match buf.read_u8()? {
            1 => Ok(OSPHandshakeOut::Acknowledge {
                ok: buf.read_i8().unwrap() != 0,
                require_login: buf.read_i8().unwrap() != 0,
                err: Self::read_optional_string(buf),
            }),
            2 => Ok(OSPHandshakeOut::LoginResponse {
                ok: buf.read_i8().unwrap() != 0,
                err: Self::read_optional_string(buf),
            }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request Type",
            )),
        }
    }
}