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

//
// Global Registers
//
pub const EIE: ControlRegister = ControlRegister::global(0x1b);
pub const EIR: ControlRegister = ControlRegister::global(0x1c);
pub const ESTAT: ControlRegister = ControlRegister::global(0x1d);
pub const ECON2: ControlRegister = ControlRegister::global(0x1e);
pub const ECON1: ControlRegister = ControlRegister::global(0x1f);

//
// Bank 0 registers
//
pub const ERDPTL: ControlRegister = ControlRegister::banked(0x00, Bank::Bank0, Block::Eth);
pub const ERDPTH: ControlRegister = ControlRegister::banked(0x01, Bank::Bank0, Block::Eth);
pub const EWRPTL: ControlRegister = ControlRegister::banked(0x02, Bank::Bank0, Block::Eth);
pub const EWRPTH: ControlRegister = ControlRegister::banked(0x03, Bank::Bank0, Block::Eth);
pub const ETXSTL: ControlRegister = ControlRegister::banked(0x04, Bank::Bank0, Block::Eth);
pub const ETXSTH: ControlRegister = ControlRegister::banked(0x05, Bank::Bank0, Block::Eth);
pub const ETXNDL: ControlRegister = ControlRegister::banked(0x06, Bank::Bank0, Block::Eth);
pub const ETXNDH: ControlRegister = ControlRegister::banked(0x07, Bank::Bank0, Block::Eth);
pub const ERXSTL: ControlRegister = ControlRegister::banked(0x08, Bank::Bank0, Block::Eth);
pub const ERXSTH: ControlRegister = ControlRegister::banked(0x09, Bank::Bank0, Block::Eth);
pub const ERXNDL: ControlRegister = ControlRegister::banked(0x0a, Bank::Bank0, Block::Eth);
pub const ERXNDH: ControlRegister = ControlRegister::banked(0x0b, Bank::Bank0, Block::Eth);
pub const ERXRDPTL: ControlRegister = ControlRegister::banked(0x0c, Bank::Bank0, Block::Eth);
pub const ERXRDPTH: ControlRegister = ControlRegister::banked(0x0d, Bank::Bank0, Block::Eth);
pub const ERXWRPTL: ControlRegister = ControlRegister::banked(0x0e, Bank::Bank0, Block::Eth);
pub const ERXWRPTH: ControlRegister = ControlRegister::banked(0x0f, Bank::Bank0, Block::Eth);

//
// Bank 1 registers
//
pub const ERXFCON: ControlRegister = ControlRegister::banked(0x18, Bank::Bank1, Block::Eth);
pub const EPKTCNT: ControlRegister = ControlRegister::banked(0x19, Bank::Bank1, Block::Eth);

//
// Bank 2 registers
//
pub const MACON1: ControlRegister = ControlRegister::banked(0x00, Bank::Bank2, Block::Mac);
pub const MACON3: ControlRegister = ControlRegister::banked(0x02, Bank::Bank2, Block::Mac);
pub const MACON4: ControlRegister = ControlRegister::banked(0x03, Bank::Bank2, Block::Mac);
pub const MABBIPG: ControlRegister = ControlRegister::banked(0x04, Bank::Bank2, Block::Mac);
pub const MAIPGL: ControlRegister = ControlRegister::banked(0x06, Bank::Bank2, Block::Mac);
pub const MAIPGH: ControlRegister = ControlRegister::banked(0x07, Bank::Bank2, Block::Mac);
pub const MAMXFLL: ControlRegister = ControlRegister::banked(0x0a, Bank::Bank2, Block::Mac);
pub const MAMXFLH: ControlRegister = ControlRegister::banked(0x0b, Bank::Bank2, Block::Mac);
pub const MICMD: ControlRegister = ControlRegister::banked(0x12, Bank::Bank2, Block::Mii);
pub const MIREGADR: ControlRegister = ControlRegister::banked(0x14, Bank::Bank2, Block::Mii);
pub const MIWRL: ControlRegister = ControlRegister::banked(0x16, Bank::Bank2, Block::Mii);
pub const MIWRH: ControlRegister = ControlRegister::banked(0x17, Bank::Bank2, Block::Mii);
pub const MIRDL: ControlRegister = ControlRegister::banked(0x18, Bank::Bank2, Block::Mii);
pub const MIRDH: ControlRegister = ControlRegister::banked(0x19, Bank::Bank2, Block::Mii);

//
// Bank 3 registers
//
pub const MAADR5: ControlRegister = ControlRegister::banked(0x00, Bank::Bank3, Block::Mac);
pub const MAADR6: ControlRegister = ControlRegister::banked(0x01, Bank::Bank3, Block::Mac);
pub const MAADR3: ControlRegister = ControlRegister::banked(0x02, Bank::Bank3, Block::Mac);
pub const MAADR4: ControlRegister = ControlRegister::banked(0x03, Bank::Bank3, Block::Mac);
pub const MAADR1: ControlRegister = ControlRegister::banked(0x04, Bank::Bank3, Block::Mac);
pub const MAADR2: ControlRegister = ControlRegister::banked(0x05, Bank::Bank3, Block::Mac);
pub const MISTAT: ControlRegister = ControlRegister::banked(0x0A, Bank::Bank3, Block::Mii);
pub const EREVID: ControlRegister = ControlRegister::banked(0x12, Bank::Bank3, Block::Eth);

//
// PHY Registers
//

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

/// PHY Control Register 1
pub const PHCON1: PhyRegister = PhyRegister::new(0x00);
/// PHY Status Register 1
pub const PHSTAT1: PhyRegister = PhyRegister::new(0x01);
/// PHY Identifier Register 1
pub const PHID1: PhyRegister = PhyRegister::new(0x02);
/// PHY Identifier Register 2
pub const PHID2: PhyRegister = PhyRegister::new(0x03);
/// PHY Control Register 2
pub const PHCON2: PhyRegister = PhyRegister::new(0x10);
/// PHY Status Register 2
pub const PHSTAT2: PhyRegister = PhyRegister::new(0x11);
/// PHY Interrupt Enable Register
pub const PHIE: PhyRegister = PhyRegister::new(0x12);
/// PHY Interrupt Request Register
pub const PHIR: PhyRegister = PhyRegister::new(0x13);
/// PHY LED Control Register
pub const PHLCON: PhyRegister = PhyRegister::new(0x14);
