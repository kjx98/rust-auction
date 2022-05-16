#![allow(clippy::integer_arithmetic)]
pub mod macros;
pub mod measure;

pub use crate::measure::Measure;

#[cfg(target_arch = "x86_64")]
pub use crate::measure::MeasureTsc;
