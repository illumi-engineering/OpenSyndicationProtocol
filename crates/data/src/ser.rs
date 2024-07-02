use bytes::BytesMut;

pub struct DataSerializer<T> {
    pub(crate) output: BytesMut,
    pub(crate) bytes_written: usize,
}

impl<T> DataSerializer<T> {
    pub fn new() -> Self {
        DataSerializer {
            output: BytesMut::new(),
            bytes_written: 0,
        }
    }
}

pub trait SerializeData {
    
}
