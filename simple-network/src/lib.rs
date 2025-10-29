#![no_std]

/// A trait that defines a minimal interface for a network driver.
///
/// This trait is intended to be implemented by network drivers. Higher-level networking code
/// (e.g. Internet Protocol) can use implementations of this trait to send and receive packets.
///
pub trait SimpleNetwork {
    /// Receive a packet from the receive buffer of the network interface.
    /// Returns number of bytes written into `buf`.
    fn receive(&mut self, buf: &mut [u8]) -> Result<usize, ReceiveError>;

    /// Send a packet to the transmit buffer of the network interface.
    fn transmit(
        &mut self,
        dst: &MacAddress,
        src: &MacAddress,
        ether_type: EtherType,
        data: &[u8],
    ) -> Result<(), TransmitError>;
}

/// An error that can occur when receiving a packet.
#[derive(Debug)]
pub enum ReceiveError {
    /// The user-provided buffer was too small to store the received packet.
    /// The contained `usize` is the required buffer size.
    BufferTooSmall(usize),
    /// An error occurred with the device.
    DeviceError,
    /// The network interface is not initialized.
    NotInitialized,
    /// The operation timed out.
    Timeout,
}

/// An error that can occur when transmitting a packet.
#[derive(Debug)]
pub enum TransmitError {
    /// The network interface aborted the transmission.
    Aborted,
    /// An error occurred with the device.
    DeviceError,
    /// An invalid parameter was provided.
    InvalidParameter,
    /// The network interface is not initialized.
    NotInitialized,
    /// The transmission timed out.
    Timeout,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
/// Zero-cost representation of a MAC address.
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    /// Returns the octets of the MAC address.
    #[must_use]
    pub const fn octets(self) -> [u8; 6] {
        self.0
    }
}

impl From<[u8; 6]> for MacAddress {
    #[inline]
    fn from(octets: [u8; 6]) -> Self {
        MacAddress(octets)
    }
}

impl From<MacAddress> for [u8; 6] {
    #[inline]
    fn from(mac: MacAddress) -> Self {
        mac.0
    }
}

impl AsRef<[u8; 6]> for MacAddress {
    #[inline]
    fn as_ref(&self) -> &[u8; 6] {
        &self.0
    }
}

impl AsMut<[u8; 6]> for MacAddress {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8; 6] {
        &mut self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
/// Zero-cost representation of the EtherType field in an Ethernet frame.
pub struct EtherType(pub u16);

impl EtherType {
    /// IEEE 802.3 Length field (should be interpreted as length, not EtherType).
    pub const IEEE_802_3: EtherType = EtherType(0x0000);
    /// IPv4 packet (RFC 894).
    pub const IPV4: EtherType = EtherType(0x0800);
    /// ARP packet.
    pub const ARP: EtherType = EtherType(0x0806);
    /// Wake-on-LAN.
    pub const WAKE_ON_LAN: EtherType = EtherType(0x0842);
    /// VLAN-tagged frame (IEEE 802.1Q).
    pub const VLAN: EtherType = EtherType(0x8100);
    /// IPv6 packet.
    pub const IPV6: EtherType = EtherType(0x86DD);

    /// Create a new EtherType from a raw u16.
    #[inline]
    pub const fn new(raw: u16) -> Self {
        EtherType(raw)
    }

    /// Get the inner u16 value of this EtherType.
    #[inline]
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    /// Construct an EtherType from two network-byte-order bytes.
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 2]) -> Self {
        EtherType(u16::from_be_bytes(bytes))
    }

    /// Convert the EtherType to network-byte-order bytes.
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 2] {
        self.0.to_be_bytes()
    }
}
