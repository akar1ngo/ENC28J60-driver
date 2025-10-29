use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiDevice;
use simple_network::{EtherType, MacAddress, ReceiveError, SimpleNetwork, TransmitError};

use crate::Enc28j60;

impl<SPI, INT, RST> SimpleNetwork for Enc28j60<SPI, INT, RST>
where
    SPI: SpiDevice,
    INT: InputPin,
    RST: OutputPin,
{
    fn receive(&mut self, buf: &mut [u8]) -> Result<usize, ReceiveError> {
        self.receive(buf).map_err(|_| ReceiveError::DeviceError)
    }

    fn transmit(
        &mut self,
        dst: &MacAddress,
        src: &MacAddress,
        ether_type: EtherType,
        data: &[u8],
    ) -> Result<(), TransmitError> {
        self.transmit(&dst.octets(), &src.octets(), ether_type.as_u16(), data)
            .map_err(|_| TransmitError::DeviceError)
    }
}
