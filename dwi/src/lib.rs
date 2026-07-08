//! DWI inverter driver.
#![no_std]
#![warn(missing_docs)]

mod frame;
mod status;

pub use status::{Faults, OperationStatus};
