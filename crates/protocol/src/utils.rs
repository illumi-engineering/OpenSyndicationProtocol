#[derive(Clone)]
pub enum ConnectionType {
    Unknown = 0,
    Client = 1,
    Server = 2
}

impl ConnectionType {
    pub(crate) fn from_u8(t: u8) -> ConnectionType {
        match t {
            1 => ConnectionType::Client,
            2 => ConnectionType::Server,
            _ => ConnectionType::Unknown
        }
    }
}

impl From<&ConnectionType> for u8 {
    fn from(t: &ConnectionType) -> Self {
        match t {
            ConnectionType::Unknown => 0,
            ConnectionType::Client => 1,
            ConnectionType::Server => 2,
        }
    }
}