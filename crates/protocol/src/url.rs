use std::fmt::{Display, Formatter};
use url::Url;

#[derive(PartialEq, Debug)]
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

#[cfg(test)]
mod tests {
    use tokio::io;
    use url::Url;
    use crate::OSPUrl;

    #[test]
    fn test_url_parse() {
        let expected = OSPUrl {
            domain: "test-url.com".to_string(),
            port: 42069,
        };

        let test_val = OSPUrl::from(Url::parse("osp://test-url.com:42069").unwrap());
        assert_eq!(expected, test_val);
    }
}