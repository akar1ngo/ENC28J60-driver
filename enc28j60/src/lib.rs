#![no_std]

#[macro_use]
mod macros;

#[cfg(feature = "simple-network")]
mod adapter;
pub mod register;
mod spi_device;

pub use spi_device::Enc28j60;
