#![feature(io_error_more)]
#![feature(thin_box)]
#![feature(unsize)]

#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::style)]
#![warn(clippy::suspicious)]
// followings are from clippy::restriction
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::format_push_string)]
#![warn(clippy::get_unwrap)]

pub mod node;
pub mod connection;

pub use {node::Node};