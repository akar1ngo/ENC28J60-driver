#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_hal_bus::spi::ExclusiveDevice;
use hal::prelude::*;
use panic_probe as _;
use simple_network::SimpleNetwork;
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

    let mut buf = [0u8; 1518];
    loop {
        cortex_m::asm::delay(1_000_000);
        match enc.read_control(register::EPKTCNT) {
            Ok(count) => {
                if orange_led.is_set_high() {
                    orange_led.set_low();
                }
                if count > 0 {
                    blue_led.set_high();
                    analyze_ether_frame(&mut enc, &mut buf);
                    blue_led.set_low();
                }
            }
            Err(_) => orange_led.set_high(),
        }
    }
}

fn analyze_ether_frame(snp: &mut impl SimpleNetwork, buf: &mut [u8]) {
    match snp.receive(buf) {
        Ok(n) => {
            defmt::info!("Received {} bytes", n);

            let dst = &buf[0..6];
            let src = &buf[6..12];
            let typ = &buf[12..14];
            // let dat = &buf[14..n];
            defmt::debug!(
                r#"Frame layout of packet:
     Source MAC: {:#x}
Destination MAC: {:#x}
     Ether Type: {:#x}
    Data Length: {} bytes"#,
                src,
                dst,
                typ,
                n - 14
            );
        }
        Err(_) => {
            defmt::error!("Error receiving packet");
        }
    }
}
