#[derive(Clone, Copy)]
pub struct ControlRegister {
    addr: u8,
    bank: Option<Bank>,
    bloc: Block,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Bank {
    Bank0,
    Bank1,
    Bank2,
    Bank3,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Block {
    Eth,
    Mac,
    Mii,
}

impl ControlRegister {
    const fn global(addr: u8) -> Self {
        Self {
            addr,
            // Global registers have a fixed address within each bank, so it is unnecessary to
            // switch banks when issuing commands.
            bank: None,
            // Global registers are all ETH registers.
            bloc: Block::Eth,
        }
    }

    const fn banked(addr: u8, bank: Bank, bloc: Block) -> Self {
        Self {
            addr,
            bank: Some(bank),
            bloc,
        }
    }

    /// The address of the register. 5-bits wide.
    pub const fn addr(&self) -> u8 {
        self.addr & 0b000_11111
    }

    /// Bank of the register. Note this always returns `None` for global registers like `ESTAT`.
    pub const fn bank(&self) -> Option<Bank> {
        self.bank
    }

    /// Generate the first byte of an SPI command, which contains a 3-bit opcode and 5-bit address.
    pub const fn opcode(&self, op: Op) -> u8 {
        (op as u8) | self.addr()
    }

    /// Reports whether a dummy byte is shifted out of the SO pin when reading the register.
    pub const fn shifts_dummy_byte(&self) -> bool {
        matches!(self.bloc, Block::Mac | Block::Mii)
    }
}

#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
pub enum Op {
    RCR = 0b000_00000,
    RBM = 0b001_00000,
    WCR = 0b010_00000,
    WBM = 0b011_00000,
    BFS = 0b100_00000,
    BFC = 0b101_00000,
}

#[derive(Clone, Copy)]
pub struct PhyRegister {
    addr: u8,
}

impl PhyRegister {
    const fn new(addr: u8) -> Self {
        // PHY registers have 5-bit address
        Self { addr: addr & 0x1f }
    }

    pub const fn addr(&self) -> u8 {
        self.addr
    }
}

const fn bank_from_u8(bank: u8) -> Bank {
    match bank {
        0 => Bank::Bank0,
        1 => Bank::Bank1,
        2 => Bank::Bank2,
        3 => Bank::Bank3,
        _ => panic!("invalid bank number"),
    }
}

#[rustfmt::skip]
control_registers![
    //
    // Global registers
    //
    (EIE,   0x1b),
    (EIR,   0x1c),
    (ESTAT, 0x1d),
    (ECON2, 0x1e),
    (ECON1, 0x1f),

    //
    // Bank 0 registers
    //
    (ERDPTL,   0x00, 0, Eth),
    (ERDPTH,   0x01, 0, Eth),
    (EWRPTL,   0x02, 0, Eth),
    (EWRPTH,   0x03, 0, Eth),
    (ETXSTL,   0x04, 0, Eth),
    (ETXSTH,   0x05, 0, Eth),
    (ETXNDL,   0x06, 0, Eth),
    (ETXNDH,   0x07, 0, Eth),
    (ERXSTL,   0x08, 0, Eth),
    (ERXSTH,   0x09, 0, Eth),
    (ERXNDL,   0x0a, 0, Eth),
    (ERXNDH,   0x0b, 0, Eth),
    (ERXRDPTL, 0x0c, 0, Eth),
    (ERXRDPTH, 0x0d, 0, Eth),
    (ERXWRPTL, 0x0e, 0, Eth),
    (ERXWRPTH, 0x0f, 0, Eth),

    //
    // Bank 1 registers
    //
    (ERXFCON, 0x18, 1, Eth),
    (EPKTCNT, 0x19, 1, Eth),

    //
    // Bank 2 registers
    //
    (MACON1,   0x00, 2, Mac),
    (MACON3,   0x02, 2, Mac),
    (MACON4,   0x03, 2, Mac),
    (MABBIPG,  0x04, 2, Mac),
    (MAIPGL,   0x06, 2, Mac),
    (MAIPGH,   0x07, 2, Mac),
    (MAMXFLL,  0x0a, 2, Mac),
    (MAMXFLH,  0x0b, 2, Mac),
    (MICMD,    0x12, 2, Mii),
    (MIREGADR, 0x14, 2, Mii),
    (MIWRL,    0x16, 2, Mii),
    (MIWRH,    0x17, 2, Mii),
    (MIRDL,    0x18, 2, Mii),
    (MIRDH,    0x19, 2, Mii),

    //
    // Bank 3 registers
    //
    (MAADR5, 0x00, 3, Mac),
    (MAADR6, 0x01, 3, Mac),
    (MAADR3, 0x02, 3, Mac),
    (MAADR4, 0x03, 3, Mac),
    (MAADR1, 0x04, 3, Mac),
    (MAADR2, 0x05, 3, Mac),
    (MISTAT, 0x0a, 3, Mii),
    (EREVID, 0x12, 3, Eth),
];

#[rustfmt::skip]
phy_registers![
    (PHCON1,   0x00),
    (PHSTAT1,  0x01),
    (PHID1,    0x02),
    (PHID2,    0x03),
    (PHCON2,   0x10),
    (PHSTAT2,  0x11),
    (PHIE,     0x12),
    (PHIR,     0x13),
    (PHLCON,   0x14),
];
