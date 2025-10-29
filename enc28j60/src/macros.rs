macro_rules! control_registers {
    () => {};

    ( ($name:ident, $addr:expr), $($rest:tt)* ) => {
        control_register!($name, $addr, None, Eth);
        control_registers!($($rest)*);
    };

    ( ($name:ident, $addr:expr, $bank:literal, $block:ident), $($rest:tt)* ) => {
        control_register!($name, $addr, $bank, $block);
        control_registers!($($rest)*);
    };
}

macro_rules! control_register {
    ($name:ident,$addr:expr, None, $block:ident) => {
        #[allow(clippy::upper_case_acronyms)]
        pub const $name: ControlRegister = ControlRegister::global($addr);
    };

    ($name:ident, $addr:expr, $bank:literal, $block:ident) => {
        #[allow(clippy::upper_case_acronyms)]
        #[rustfmt::skip]
        pub const $name: ControlRegister = ControlRegister::banked($addr, bank_from_u8($bank), Block::$block);
    };
}

macro_rules! phy_registers {
    ($(($name:ident, $addr:expr)),* $(,)?) => {
        $( phy_register!($name, $addr); )*
    };
}

macro_rules! phy_register {
    ($name:ident, $addr:expr) => {
        #[allow(clippy::upper_case_acronyms)]
        pub const $name: PhyRegister = PhyRegister::new($addr);
    };
}
