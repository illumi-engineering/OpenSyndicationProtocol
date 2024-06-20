use std::fmt::{Display, Formatter};
use url::Url;

pub struct OSPUrl {
    pub domain: String,
    pub port: u16,
}

impl From<Url> for OSPUrl {
    fn from(value: Url) -> Self {
        assert_eq!(value.scheme(), "osp");

        OSPUrl {
            domain: value.domain().unwrap().to_string(),
            port: value.port().unwrap()
        }
    }
}

impl Display for OSPUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("osp://{}:{}", self.domain, self.port).as_str())
    }
}