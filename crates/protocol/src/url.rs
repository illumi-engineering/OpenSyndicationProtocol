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