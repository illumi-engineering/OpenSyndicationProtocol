use std::io::{self, Read, Write};
use byteorder::{NetworkEndian, WriteBytesExt, ReadBytesExt};
use super::utils::{DeserializePacket, extract_string, SerializePacket};

pub enum OSPResponse {
    Acknowledge {
        ok: bool,
        err: Option<String>,
    }
}

impl From<&OSPResponse> for u8 {
    fn from(req: &OSPResponse) -> Self {
        match req {
            OSPResponse::Acknowledge { .. } => 1,
        }
    }
}

impl SerializePacket for OSPResponse {
    /// Serialize Response to bytes (to send to client)
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize> {
        buf.write_u8(self.into())?; // Message Type byte
        let mut bytes_written: usize = 1;
        match self {
            OSPResponse::Acknowledge { ok, err } => {
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
            // OSPResponse::SyncProject { ok, changed } => {
            //     buf.write_i8(*ok as i8)?;
            //     bytes_written += 1;

            //     buf.write_i8(*changed as i8)?;
            //     bytes_written += 1;

            //     // let message = message.as_bytes();
            //     // buf.write_u16::<NetworkEndian>(message.len() as u16)?;
            //     // buf.write_all(&message)?;
            //     // bytes_written += 2 + message.len();
            // }
            // OSPResponse::InstallProject { ok } => {
            //     buf.write_i8(*ok as i8)?;
            //     bytes_written += 1;
            // }
        }
        Ok(bytes_written)
    }
}

impl DeserializePacket for OSPResponse {
    type Output = OSPResponse;

    fn deserialize(buf: &mut impl Read) -> io::Result<OSPResponse> {
        // We'll match the same `u8` that is used to recognize which response type this is
        match buf.read_u8()? {
            1 => Ok(OSPResponse::Acknowledge {
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