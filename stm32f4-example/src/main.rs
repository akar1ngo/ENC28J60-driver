#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_hal_bus::spi::ExclusiveDevice;
use hal::prelude::*;
use panic_probe as _;
use stm32f4xx_hal::{self as hal, hal_02::spi::MODE_0, rcc::Config, spi::Spi};

use enc28j60::{Enc28j60, register};

#[entry]
fn main() -> ! {
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let dp = hal::pac::Peripherals::take().unwrap();

    // システムクロックの設定
    let mut rcc = dp.RCC.freeze(Config::hsi().sysclk(16.MHz()));

    // GPIO 初期化
    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);
    let gpiod = dp.GPIOD.split(&mut rcc);

    // SPI1 ピン設定
    let sck = gpioa.pa5.into_alternate();
    let miso = gpioa.pa6.into_alternate();
    let mosi = gpioa.pa7.into_alternate();

    // ENC28J60 制御ピン
    let mut cs = gpiob.pb1.into_push_pull_output();
    let mut reset = gpiob.pb0.into_push_pull_output();
    let int = gpioa.pa1.into_pull_up_input();

    // debug LED
    let mut orange_led = gpiod.pd13.into_push_pull_output();
    let mut blue_led = gpiod.pd15.into_push_pull_output();

    let mut spi = Spi::new(
        dp.SPI1,
        (Some(sck), Some(miso), Some(mosi)),
        MODE_0,
        8.MHz(), // 最大20MHz程度までOK
        &mut rcc,
    );

    // --- ENC28J60リセット
    orange_led.set_high();
    blue_led.set_high();
    {
        reset.set_low();
        cortex_m::asm::delay(4_000_000);
        reset.set_high();
        cortex_m::asm::delay(4_000_000);
    }
    blue_led.set_low();
    orange_led.set_low();

    // ---
    let dly = cp.SYST.delay(&rcc.clocks);
    let dev = ExclusiveDevice::new(&mut spi, &mut cs, dly).expect("Set up SpiDevice");
    let mut enc = Enc28j60::new(dev, int, reset);

    let estat_val = enc.read_control(register::ESTAT).unwrap_or(0xFF);
    defmt::info!("ESTAT={:?}", estat_val);

    let mut dly = dp.TIM2.delay_us(&mut rcc);
    enc.initialize(&mut dly).expect("initialize");
    let estat_val = enc.read_control(register::ESTAT).unwrap_or(0xFF);
    defmt::info!("ESTAT={:?}", estat_val);

    loop {
        cortex_m::asm::delay(16_000_000);
    }
}
