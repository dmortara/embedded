/*
 * 2025-01-08
 * ADCH and HID for STM32F103C8T6 (a.k.a. The Blue Pill)
 * using RTIC
 */
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]
#![no_main]

use defmt::info;
use rtic::app;
use rtic_monotonics::systick::prelude::*;
use {defmt_rtt as _, panic_probe as _};

use stm32f1xx_hal::gpio::{Analog, Output, PinState, PushPull, PA0, PA1, PC13};
use stm32f1xx_hal::pac::ADC1; // Analog Digital Converter one
use stm32f1xx_hal::{adc, prelude::*};

// System time interrupt every 300 ms
systick_monotonic!(Mono, 300);

//
//  A blinky example using RTIC and defmt
//
#[app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {

    use super::*;

    // Analog pins for potenciometers/Hall effect
    struct AnalogInput {
        pa0: PA0<Analog>, // Analog input (PA0)
        pa1: PA1<Analog>, // Analog input (PA1)
    }

    #[shared] // Shared context
    struct SharedContext {}

    #[local] // Local context
    struct LocalContext {
        led: PC13<Output<PushPull>>, // Has and LED soldered permanently to this pin
        adc1: adc::Adc<ADC1>,        // ADC1
        analog: AnalogInput,         // Analog input
    }

    // Setup and initialization task.
    // Runs before any other task
    // init returns the shared and local context
    #[init]
    fn init(context: init::Context) -> (SharedContext, LocalContext) {
        // Clock setup
        let mut flash = context.device.FLASH.constrain();
        let rcc = context.device.RCC.constrain();

        // Initialize the timer (monothonic)
        Mono::start(context.core.SYST, 36_000_000); // 36MHz

        info!("Hello ADC");

        // Setup
        let clocks = rcc
            .cfgr
            .use_hse(8.MHz()) // High speed clock
            .sysclk(36.MHz()) // System clock
            .pclk1(36.MHz()) // Perifferal clock
            .adcclk(2.MHz()) // ADC clock
            .freeze(&mut flash.acr);

        let mut gpioa = context.device.GPIOA.split(); // To access adc 1

        // Setup ADC, with pa0 as an analog input
        let adc1 = adc::Adc::adc1(context.device.ADC1, clocks);
        let pa0 = gpioa.pa0.into_analog(&mut gpioa.crl);
        let pa1 = gpioa.pa1.into_analog(&mut gpioa.crl);
        //let mut adc1_pa1 = gpioa.pa1.into_analog(&mut gpioa.crl);

        // Init LED GPIO
        let mut gpioc = context.device.GPIOC.split(); // To access pc13 led
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::Low);

        let analog = AnalogInput { pa0, pa1 };

        // Schedule the blinking task
        read_adc0_and_blink::spawn().ok();

        // Return the shared and local context
        (SharedContext {}, LocalContext { led, adc1, analog })
    }

    // Turn the led on and off, show some RTT logs, increase the entropy of the universe ...
    #[task(local = [led, adc1, analog])]
    async fn read_adc0_and_blink(context: read_adc0_and_blink::Context) {
        loop {
            // Read value form ADC
            let x: u16 = context // A 12 bits only
                .local
                .adc1
                .read(&mut context.local.analog.pa0)
                .unwrap();

            let y: u16 = context // A 12 bits only
                .local
                .adc1
                .read(&mut context.local.analog.pa1)
                .unwrap();

            match context.local.led.get_state() {
                PinState::Low => {
                    context.local.led.set_high();
                    info!("LED is On  (x,y): ({},{})", x, y);
                }
                PinState::High => {
                    context.local.led.set_low();
                    info!("LED is Off (x,y): ({},{})", x, y);
                }
            };
            // Wait another 300 ms. Release the CPU to other tasks ...
            Mono::delay(300.millis()).await;
        }
    }

    /// Read ADCs and send hid information to USB
    #[task(binds = USB_HP_CAN_TX)]
    fn write_usb(_: write_usb::Context) {
        info!("USB reade to TX");
    }

    //   #[task(binds = USB_LP_CAN_RX)]
    //    fn read_usb(_: read_usb::Context) {}
}
