/*
 * 2025-07-01
 * ADC and HID for STM32F103C8T6 (a.k.a. Blue Pill)
 * Using embassy
 *
 * NOTE: 2026-03-27: The Hall effect I am using does not work propelly with a hight speed (12Mhz
 *                   ADC bus). Currently running a 9Mhz ADC bus (72MHz / 8) and it seems to work
 *                   ok. I have not tried a lower speed yet.
 */
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::rcc;
use embassy_stm32::rcc::{ADCPrescaler, AHBPrescaler, APBPrescaler, HseMode, Sysclk};
use embassy_stm32::time::Hertz;
use embassy_stm32::{adc, bind_interrupts};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
});

/**
 * A blinky example using embassy and defmt
 */
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Setup clocks. Not required here but a good exercise.
    let mut config = Config::default();

    // High Speed External cristal oscillator (instead of the interal RC HSI)
    config.rcc.hse = Some(rcc::Hse {
        freq: Hertz(8_000_000), // 8MHz cristal
        mode: HseMode::Oscillator,
    });

    // Reset and Clock Control configuration
    config.rcc.pll = Some(rcc::Pll {
        // Phase Locked Loop configuration
        src: rcc::PllSource::HSE,     // 8MHz from config.rcc.hse
        prediv: rcc::PllPreDiv::DIV1, // HSE -> 8 / 1 = 8MHz
        mul: rcc::PllMul::MUL9,       // 8MHz * 9 = 72MHz
    });
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.ahb_pre = AHBPrescaler::DIV1; // AHB = 72MHz (max 72MHz)
    config.rcc.apb1_pre = APBPrescaler::DIV2; // APB1 = 72 / 2 = 36MHz (max 36MHz on STM32F1)
    config.rcc.apb2_pre = APBPrescaler::DIV1; // APB2 = 72MHz (max 72MHz)
    config.rcc.adc_pre = ADCPrescaler::DIV8; // ADC = 72 / 8 = 9MHz (max 14MHz)

    let mut periferal = embassy_stm32::init(config);

    info!("Hello ADC");

    let mut led = Output::new(periferal.PC13, Level::High, Speed::Low);
    // pa0 and pa1 are wired to ADC1 (STM32F1)
    let mut adc = Adc::new(periferal.ADC1);

    loop {
        let x = adc.read(&mut periferal.PA0, SampleTime::CYCLES13_5).await;
        let y = adc.read(&mut periferal.PA1, SampleTime::CYCLES13_5).await;
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
