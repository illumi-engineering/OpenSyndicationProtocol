// data types



// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
       
//     }
// }

mod protocol;
mod utils;
mod url;
pub mod packet;

pub use {protocol::*, url::OSPUrl, utils::ConnectionType};