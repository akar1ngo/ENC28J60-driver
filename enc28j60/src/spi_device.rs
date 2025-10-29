use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::{Operation, SpiDevice};

use super::register::*;

pub struct Enc28j60<SPI: SpiDevice, INT: InputPin, RST: OutputPin> {
    /// An SPI device
    spi: SPI,

    #[allow(dead_code)]
    /// Interrupt pin
    int: INT,

    /// Reset pin
    reset: RST,

    /// Current bank,
    current_bank: Bank,
}

impl<SPI, INT, RST> Enc28j60<SPI, INT, RST>
where
    SPI: SpiDevice,
    INT: InputPin,
    RST: OutputPin,
{
    pub fn new(spi: SPI, int: INT, reset: RST) -> Self {
        Enc28j60 {
            spi,
            int,
            reset,
            current_bank: Bank::Bank0,
        }
    }

    pub fn initialize(&mut self) -> Result<(), SPI::Error> {
        // TODO: the proper `delay` method needs access to a timer, but the driver should not take
        // ownership of it. Look into passing a mutable reference to a delay object in the future.
        self.reset_via_spi()?;
        cortex_m::asm::delay(1_000_000);

        let revision = self.read_control(EREVID).unwrap_or(0xff);

        match revision {
            0x00 | 0xff => { /* Chip reset, or read failure */ }
            0b0010 | 0b1000 | 0b0101 | 0b0110 => { /* Hardware bug */ }
            _ => loop {
                let estat = self.read_control(ESTAT)?;
                if (estat & 0x01) != 0 {
                    break;
                }
            },
        }

        self.ensure_autoinc()?;

        //
        // Set up receive and transmit buffers
        //
        {
            const RX_START: u16 = 0x0000;
            const RX_END: u16 = TX_START - 1;
            // It is recommended that:
            // 1. ETXST points to an unused location in memory.
            // 2. the address of ETXST is even.
            const TX_START: u16 = 0x1000;

            // Before receiving any packets, the receive buffer must be initialized by programming
            // the ERXST and ERXND Pointers.
            self.write_u16(ERXSTL, ERXSTH, RX_START)?;
            self.write_u16(ERXNDL, ERXNDH, RX_END)?;
            // For tracking purposes, the ERXRDPT registers should additionally be programmed with
            // the same value.
            self.write_u16(ERXRDPTL, ERXRDPTH, RX_START)?;

            // No explicit action is required to initialize the transmission buffer.
            self.write_u16(ETXSTL, ETXSTH, TX_START)?;
        }

        //
        // MAC initialization
        //
        {
            // Set the MARXEN bit in MACON1 to enable the MAC to receive frames.
            self.write_control(MACON1, 1)?;

            // Configure the PADCFG, TXCRCEN and FULDPX bits of MACON3.
            //
            // In this setup, we are:
            // - enabling full duplex mode
            // - enabling frame length checking
            // - appending a CRC to transmitted frames
            // - padding all short frames to 60 bytes and appending a CRC
            const MACON3_MASK: u8 = 0b00110011;
            self.write_control(MACON3, MACON3_MASK)?;

            // Program the MAMXFL registers with the maximum frame length.
            const MAX_FRAME_LENGTH: u16 = 1518;
            self.write_u16(MAMXFLL, MAMXFLH, MAX_FRAME_LENGTH)?;

            // Configure MABBIPG with recommended value for full-duplex mode.
            self.write_control(MABBIPG, 0x15)?;

            // Configure MAIPGL with recommended value.
            self.write_control(MAIPGL, 0x06)?;

            // Program the local MAC address
            self.write_control(MAADR1, 0xff)?;
            self.write_control(MAADR2, 0xca)?;
            self.write_control(MAADR3, 0xde)?;
            self.write_control(MAADR4, 0xee)?;
            self.write_control(MAADR5, 0xff)?;
            self.write_control(MAADR6, 0xc0)?;
        }

        self.write_control(ERXFCON, 0)?;

        //
        // PHY initialization
        //
        {
            // For proper duplex operation, PHCON1.PDPXMD must also match MACON3.FULDPX.
            self.write_phy(PHCON1, 0x0100)?;

            // We are in full-duplex mode, but for sanitation reasons, we disable PHCON2.HDLDIS.
            self.write_phy(PHCON2, 0x0100)?;
        }

        // Issue interrupts when packets arrive. This allows users to wfi() in a loop to
        // efficiently wait for incoming packets.
        self.write_control(EIE, 0b1100_0000)?;

        // At this point, the receive buffer has been initialized, MAC has been configured, and
        // the default receive filter has been set up. We are ready to enable reception.
        self.write_control(ECON1, 0b0000_0100)?;

        Ok(())
    }

    /// Issues a system reset via the device's reset pin.
    ///
    /// Since the function can run at any time, it may be used to asynchronously reset the device.
    ///
    pub fn reset<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), RST::Error> {
        // Hold the RESET pin low for at least $t_{RSTLOW}$ ns
        self.reset.set_low()?;
        delay.delay_ns(400);
        self.reset.set_high()?;

        // After a System Reset, all PHY registers should not be read or written to until at least
        // 50 μs have passed since the Reset has ended.
        delay.delay_us(50);

        Ok(())
    }

    /// Issues a System Soft Reset via SPI by invoking SRC (System Reset Command).
    ///
    /// # Note
    ///
    /// There may be other SPI commands in progress, so the reset is not immediate. If you need
    /// an immediate reset, use the `reset` function.
    ///
    pub fn reset_via_spi(&mut self) -> Result<(), SPI::Error> {
        // Unlike other SPI commands, the SRC is only a single byte command and does not operate on
        // any register.
        self.spi.write(&[0xFF])
    }

    fn ensure_autoinc(&mut self) -> Result<(), SPI::Error> {
        const AUTOINC_MASK: u8 = 0x80;
        let cmd = [ECON2.opcode(Op::BFS), AUTOINC_MASK];
        self.spi.write(&cmd)
    }

    fn mem_read(&mut self, data: &mut [u8]) -> Result<(), SPI::Error> {
        const RBM_MAGIC: u8 = 0x1a;
        const OPCODE: u8 = (Op::RBM as u8) | RBM_MAGIC;

        let mut ops = [Operation::Write(&[OPCODE]), Operation::Read(data)];
        self.spi.transaction(&mut ops)
    }

    fn mem_write(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        const WBM_MAGIC: u8 = 0x1a;
        const OPCODE: u8 = (Op::WBM as u8) | WBM_MAGIC;

        let mut ops = [Operation::Write(&[OPCODE]), Operation::Write(data)];
        self.spi.transaction(&mut ops)
    }

    pub fn read_control(&mut self, reg: ControlRegister) -> Result<u8, SPI::Error> {
        if let Some(bank) = reg.bank()
            && self.current_bank != bank
        {
            self.set_bank(bank)?;
        }

        let mut buf = [0u8; 3];
        let command = [reg.opcode(Op::RCR), 0u8];

        self.spi.transfer(&mut buf, &command)?;

        if reg.shifts_dummy_byte() {
            Ok(buf[2])
        } else {
            Ok(buf[1])
        }
    }

    pub fn write_control(&mut self, reg: ControlRegister, data: u8) -> Result<(), SPI::Error> {
        if let Some(bank) = reg.bank()
            && self.current_bank != bank
        {
            self.set_bank(bank)?;
        }

        let buf = [reg.opcode(Op::WCR), data];
        self.spi.write(&buf)
    }

    pub fn read_phy(&mut self, reg: PhyRegister) -> Result<u16, SPI::Error> {
        // 1. Write address to MIREGADR
        self.write_control(MIREGADR, reg.addr())?;

        // 2. Set MICMD.MIIRD
        self.write_control(MICMD, 0b01)?;

        // 3. Poll MISTAT.BUSY to be certain that the operation is complete
        loop {
            let mistat = self.read_control(MISTAT)?;
            if (mistat & 0b01) == 0 {
                break;
            }
        }

        // 4. Clear MICMD.MIIRD
        self.write_control(MICMD, 0b00)?;

        // 5. Read data from MIRDL and MIRDH
        self.read_u16(MIRDL, MIRDH)
    }

    pub fn write_phy(&mut self, reg: PhyRegister, data: u16) -> Result<(), SPI::Error> {
        // 1. Write address to MIREGADR
        self.write_control(MIREGADR, reg.addr())?;

        // 2. Write lower 8 bits of data into MIWRL
        // 3. Write upper 8 bits of data into MIWRH
        self.write_u16(MIWRL, MIWRH, data)
    }

    //
    // Network function
    //

    /// Receive a single packet into `buf`. Returns number of bytes written into `buf`.
    pub fn receive(&mut self, buf: &mut [u8]) -> Result<usize, SPI::Error> {
        let packet_count = self.read_control(EPKTCNT)?;
        if packet_count == 0 {
            return Ok(0);
        }

        // Read the receive status vector (6 bytes)
        // Format: [next_packet_ptr(2), byte_count(2), status(2)]
        let mut rsv = [0u8; 6];
        self.mem_read(&mut rsv)?;

        // Extract next packet pointer and byte count (little-endian)
        let next_packet = u16::from_le_bytes([rsv[0], rsv[1]]);
        let byte_count = u16::from_le_bytes([rsv[2], rsv[3]]) as usize;

        // The byte count includes the 4-byte CRC, so subtract it for payload length
        let payload_len = byte_count.saturating_sub(4);
        let copy_len = core::cmp::min(payload_len, buf.len());

        // Read the packet payload into the buffer
        if copy_len > 0 {
            self.mem_read(&mut buf[..copy_len])?;

            // If the packet is larger than our buffer, we need to skip the remaining bytes
            // to properly advance the memory read pointer
            if payload_len > copy_len {
                let mut remaining = payload_len - copy_len;
                let mut dummy = [0u8; 64];
                while remaining > 0 {
                    let chunk_size = core::cmp::min(remaining, dummy.len());
                    self.mem_read(&mut dummy[..chunk_size])?;
                    remaining -= chunk_size;
                }
            }
        }

        // Update ERXRDPT to free the memory used by this packet
        // ERXRDPT should point to the byte before the next packet's start
        let erx_start = self.read_u16(ERXSTL, ERXSTH)?;
        let erx_end = self.read_u16(ERXNDL, ERXNDH)?;

        let new_rdpt = if next_packet == erx_start {
            // Wrap-around case: next packet is at the start, so point to the end
            erx_end
        } else {
            // Normal case: point to the byte before the next packet
            next_packet - 1
        };

        self.write_u16(ERXRDPTL, ERXRDPTH, new_rdpt)?;

        // Decrement the packet count by setting ECON1.PKTDEC
        const PKTDEC_MASK: u8 = 0b0100_0000;
        let cmd = [ECON2.opcode(Op::BFS), PKTDEC_MASK];
        self.spi.write(&cmd)?;

        Ok(copy_len)
    }

    /// Transmit a packet with the given source MAC, destination MAC, and data payload.
    /// The data should include the EtherType/Length field and payload.
    pub fn transmit(
        &mut self,
        dst: &[u8; 6],
        src: &[u8; 6],
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        // 1a. Read current ETXST to know where to write
        let tx_start = self.read_u16(ETXSTL, ETXSTH)?;

        // 1b. Set up write pointer to tx_start
        self.write_u16(EWRPTL, EWRPTH, tx_start)?;

        // 2a. Write the per-packet control byte
        let control = [0u8];
        self.mem_write(&control)?;

        // 2b. Write the Ethernet frame header
        self.mem_write(dst)?;
        self.mem_write(src)?;

        // 2c. Write the data (should include EtherType + payload)
        self.mem_write(data)?;

        // 3. Appropriately program the ETXND Pointer.
        // It should point to the last byte in the data payload.
        let packet_len = control.len() + src.len() + dst.len() + data.len();
        let tx_end = tx_start + (packet_len as u16) - 1;
        self.write_u16(ETXNDL, ETXNDH, tx_end)?;

        // 4. Clear EIR.TXIF. For now, we do not enable interrupts (EIE.TXIE and EIE.INTIE).
        const TXIF_MASK: u8 = 0b0000_1000;
        let cmd = [EIR.opcode(Op::BFC), TXIF_MASK];
        self.spi.write(&cmd)?;

        // 5. Start the transmission process by setting ECON1.TXRTS.
        const TXRTS_MASK: u8 = 0b0000_1000;
        let cmd = [ECON1.opcode(Op::BFS), TXRTS_MASK];
        self.spi.write(&cmd)?;

        // Wait for transmission to complete
        loop {
            let econ1 = self.read_control(ECON1)?;
            if (econ1 & TXRTS_MASK) == 0 {
                break;
            }
        }

        // Check if transmission was successful
        const TXABRT_MASK: u8 = 0b0000_0010;
        let estat = self.read_control(ESTAT)?;
        if (estat & TXABRT_MASK) != 0 {
            // Aborted. Clear flag and log error for now.
            let cmd = [ESTAT.opcode(Op::BFC), TXABRT_MASK];
            self.spi.write(&cmd)?;
            // defmt::error!("transmit: aborted");
        }

        Ok(())
    }

    //
    // Helper function
    //

    fn read_u16(&mut self, lo: ControlRegister, hi: ControlRegister) -> Result<u16, SPI::Error> {
        let lo = self.read_control(lo)? as u16;
        let hi = self.read_control(hi)? as u16;
        Ok(lo | (hi << 8))
    }

    fn write_u16(
        &mut self,
        lo: ControlRegister,
        hi: ControlRegister,
        val: u16,
    ) -> Result<(), SPI::Error> {
        self.write_control(lo, (val & 0xff) as u8)?;
        self.write_control(hi, (val >> 8) as u8)?;
        Ok(())
    }

    fn set_bank(&mut self, bank: Bank) -> Result<(), SPI::Error> {
        let mask = 0b11;
        let command = [ECON1.opcode(Op::BFC), mask];
        self.spi.write(&command)?;

        let command = [ECON1.opcode(Op::BFS), (bank as u8) & mask];
        self.spi.write(&command)?;
        self.current_bank = bank;

        Ok(())
    }
}
