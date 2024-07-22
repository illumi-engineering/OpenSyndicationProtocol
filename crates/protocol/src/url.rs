use std::fmt::{Display, Formatter};
use tokio::io;
use url::Url;

#[derive(PartialEq, Eq, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct OSPUrl {
    pub domain: String,
    pub port: u16,
}

impl TryFrom<Url> for OSPUrl {
    type Error = io::Error;

    fn try_from(value: Url) -> io::Result<Self> {
        assert_eq!(value.scheme(), "osp");

        Ok(Self {
            domain: value.domain().expect("Url must have domain").to_string(),
            port: value.port().expect("Url must have port")
        })
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
    fn test_url_parse() -> io::Result<()> {
        let expected = OSPUrl {
            domain: "test-url.com".to_string(),
            port: 42069,
        };

        let test_val = OSPUrl::try_from(
            Url::parse("osp://test-url.com:42069")
                .map_err(|e| { io::Error::new(io::ErrorKind::InvalidData, e.to_string()) })?
        )?;
        assert_eq!(expected, test_val);
        Ok(())
    }
}