#![no_std]

pub mod register;
mod spi_device;

pub use spi_device::Enc28j60;
