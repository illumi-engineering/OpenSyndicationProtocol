use std::io::{self, Read, Write};
use byteorder::{NetworkEndian, WriteBytesExt, ReadBytesExt};
use super::utils::{DeserializePacket, extract_string, SerializePacket};

pub enum OSPHandshakeIn {
    Hello {},
}

impl From<&OSPHandshakeIn> for u8 {
    fn from(req: &OSPHandshakeIn) -> Self {
        match req {
            OSPHandshakeIn::Hello { .. } => 1,
            // OSPHandshake::InstallProject { .. } => 2,
        }
    }
}

impl SerializePacket for OSPHandshakeIn {
    /// Serialize Request to bytes (to send to server)
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize> {
        buf.write_u8(self.into())?; // Message Type byte
        let mut bytes_written: usize = 1;
        match self {
            OSPHandshakeIn::Hello {} => {
                // NODATA
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
            1 => Ok(OSPHandshakeIn::Hello {}),
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
        err: Option<String>,
    }
}

impl From<&OSPHandshakeOut> for u8 {
    fn from(req: &OSPHandshakeOut) -> Self {
        match req {
            OSPHandshakeOut::Acknowledge { .. } => 1,
        }
    }
}

impl SerializePacket for OSPHandshakeOut {
    /// Serialize Response to bytes (to send to client)
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize> {
        buf.write_u8(self.into())?; // Message Type byte
        let mut bytes_written: usize = 1;
        match self {
            OSPHandshakeOut::Acknowledge { ok, err } => {
                buf.write_i8(*ok as i8)?;
                bytes_written += 1;

                if let Some(msg) = err {
                    buf.write_i8(1)?; // write true to indicate that this will have a message
                    bytes_written += 1;

                    let message = msg.as_bytes();
                    buf.write_u16::<NetworkEndian>(message.len() as u16)?;
                    buf.write_all(&message)?;
                    bytes_written += 2 + message.len();
                } else {
                    buf.write_i8(0)?; // write false for no err
                    bytes_written += 1;
                }
            }
            // OSPHandshakeOut::SyncProject { ok, changed } => {
            //     buf.write_i8(*ok as i8)?;
            //     bytes_written += 1;

            //     buf.write_i8(*changed as i8)?;
            //     bytes_written += 1;

            //     // let message = message.as_bytes();
            //     // buf.write_u16::<NetworkEndian>(message.len() as u16)?;
            //     // buf.write_all(&message)?;
            //     // bytes_written += 2 + message.len();
            // }
            // OSPHandshakeOut::InstallProject { ok } => {
            //     buf.write_i8(*ok as i8)?;
            //     bytes_written += 1;
            // }
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
                err: if buf.read_i8().unwrap() != 0 { // if the boolean is set read the optional value
                    Some(extract_string(buf)?)
                } else { None } // otherwise None
            }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request Type",
            )),
        }
    }
}