//! Core of the dual-wire serial protocol shared by compressor drivers.
#![no_std]
#![warn(missing_docs)]

mod command;
mod driver;
mod error;

pub use command::Command;
pub use driver::{Config, Driver};
pub use error::*;
