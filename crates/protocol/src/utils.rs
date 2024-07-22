use std::fmt::{Display, Formatter};

#[derive(Clone, PartialEq, Eq)]
pub enum ConnectionType {
    Unknown = 0,
    Client = 1,
    Server = 2
}

impl From<u8> for ConnectionType {
    fn from(t: u8) -> Self {
        match t {
            1 => Self::Client,
            2 => Self::Server,
            _ => Self::Unknown
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

#[derive(Clone, PartialEq, Eq)]
pub enum ConnectionIntent {
    Subscribe,
    TransferData,
    Unknown,
}

impl From<u8> for ConnectionIntent {
    fn from(i: u8) -> Self {
        match i {
            1 => Self::Subscribe,
            2 => Self::TransferData,
            _ => Self::Unknown,
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
            Self::Subscribe => {
                write!(f, "subscribe")
            }
            Self::TransferData => {
                write!(f, "transfer-data")
            }
            Self::Unknown => {
                write!(f, "unknown")
            }
        }
    }
}