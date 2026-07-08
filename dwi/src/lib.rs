//! DWI inverter driver.
#![no_std]
#![warn(missing_docs)]

pub mod error;
mod frame;
mod status;

pub use status::{Faults, OperationStatus};
