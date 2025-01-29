/*
 * 2024-01-08
 * Blinky for STM32F103C8T6 (a.k.a. Blue Pill)
 * Using embassy
 */
#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/**
 * A blinky example using embassy and defmt
 */
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let periferal = embassy_stm32::init(Default::default());

    info!("Hello blinky");

    let mut led = Output::new(periferal.PC13, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await; // Block (wait) for 300 ms

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}
