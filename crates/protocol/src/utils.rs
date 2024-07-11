use std::fmt::{Display, Formatter};

#[derive(Clone, PartialEq)]
pub enum ConnectionType {
    Unknown = 0,
    Client = 1,
    Server = 2
}

impl From<u8> for ConnectionType {
    fn from(t: u8) -> ConnectionType {
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

#[derive(Clone, PartialEq)]
pub enum ConnectionIntent {
    Subscribe,
    TransferData,
    Unknown,
}

impl From<u8> for ConnectionIntent {
    fn from(i: u8) -> Self {
        match i {
            1 => ConnectionIntent::Subscribe,
            2 => ConnectionIntent::TransferData,
            _ => ConnectionIntent::Unknown,
        }
    }
}

impl From<&ConnectionIntent> for u8 {
    fn from(i: &ConnectionIntent) -> Self {
        match i {
            ConnectionIntent::Unknown => 0,
            ConnectionIntent::Subscribe => 1,
            ConnectionIntent::TransferData => 2,
        }
    }
}

impl Display for ConnectionIntent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionIntent::Subscribe => {
                write!(f, "subscribe")
            }
            ConnectionIntent::TransferData => {
                write!(f, "transfer-data")
            }
            ConnectionIntent::Unknown => {
                write!(f, "unknown")
            }
        }
    }
}