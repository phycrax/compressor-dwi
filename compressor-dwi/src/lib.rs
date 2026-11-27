//! Core of the Dual Wire Interface (DWI) protocol shared by compressor drivers.
//!
//! Most users want [`secop-dwi`](https://docs.rs/secop-dwi) or [`embraco-dwi`](https://docs.rs/embraco-dwi) instead.
#![no_std]
#![warn(missing_docs)]

mod command;
mod driver;
mod error;

pub use command::Command;
pub use driver::Driver;
pub use error::{CommunicationError, Error, ResponseError};
