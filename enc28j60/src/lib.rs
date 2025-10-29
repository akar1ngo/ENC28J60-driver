#![no_std]

#[macro_use]
mod macros;

pub mod register;
mod spi_device;

pub use spi_device::Enc28j60;
