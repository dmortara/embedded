/*
 * 2025-07-01
 * ADC and HID for STM32F103C8T6 (a.k.a. Blue Pill)
 * Using embassy
 */
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::adc::Adc;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rcc;
use embassy_stm32::rcc::{ADCPrescaler, AHBPrescaler, APBPrescaler, HseMode, Sysclk};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/**
 * A blinky example using embassy and defmt
 */
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello ADC");
    // Setup clocks. Not required here but a good exercise.
    let mut config = Config::default();

    // High Speed External cristal oscillator
    config.rcc.hse = Some(rcc::Hse {
        freq: Hertz(8_000_000),
        mode: HseMode::Oscillator,
    });

    // @TODO: Review configuration to get same frequencies as with rtic-hid example
    config.rcc.sys = Sysclk::HSE;
    config.rcc.ahb_pre = AHBPrescaler::DIV2;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.adc_pre = ADCPrescaler::DIV4;

    let mut periferal = embassy_stm32::init(config);

    let mut led = Output::new(periferal.PC13, Level::High, Speed::Low);
    // pa0 and pa1 are wired to ADC1 (STM32F1)
    let mut adc = Adc::new(periferal.ADC1);

    loop {
        let x = adc.read(&mut periferal.PA0).await;
        let y = adc.read(&mut periferal.PA1).await;
        match led.get_output_level() {
            Level::Low => {
                led.set_high();
                info!("LED is On  (x,y): ({},{})", x, y);
            }
            Level::High => {
                led.set_low();
                info!("LED is Off (x,y): ({},{})", x, y);
            }
        };
        Timer::after_millis(300).await; // Block (wait)
    }
}
