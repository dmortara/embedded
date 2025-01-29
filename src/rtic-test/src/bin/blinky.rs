/*
 * 2024-01-08
 * Blinky for STM32F103C8T6 (a.k.a. The Blue Pill)
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

use stm32f1xx_hal::gpio::{Output, PinState, PushPull, PC13};
use stm32f1xx_hal::prelude::*;

// System time interrupt every 300 ms
systick_monotonic!(Mono, 300);

// LED is on or off. It does not handle superposition :-)
enum LedState {
    On,  // gpio is high
    Off, // gpio is low
}

//
//  A blinky example using RTIC and defmt
//
#[app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {

    use super::*;

    #[shared] // Shared context
    struct SharedContext {}

    #[local] // Local context
    struct LocalContext {
        led: PC13<Output<PushPull>>, // Has and LED soldered permanently to this pin
        led_state: LedState,
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

        info!("Hello blinky");

        // Setup
        let _clocks = rcc
            .cfgr
            .use_hse(8.MHz()) // High speed clock
            .sysclk(36.MHz()) // System clock
            .pclk1(36.MHz()) // Perifferal clock
            .freeze(&mut flash.acr);

        // Init LED GPIO
        let mut gpioc = context.device.GPIOC.split();
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::Low);

        // Schedule the blinking task
        blink::spawn().ok();

        // Return the shared and local context
        (
            SharedContext {},
            LocalContext {
                led,
                led_state: LedState::Off,
            },
        )
    }

    // Turn the led on and off, show some RTT logs, encrease the entropy of the universe ...
    #[task(local = [led, led_state])]
    async fn blink(context: blink::Context) {
        loop {
            match *context.local.led_state {
                LedState::Off => {
                    context.local.led.set_high();
                    *context.local.led_state = LedState::On;
                    info!("LED is On");
                }
                LedState::On => {
                    context.local.led.set_low();
                    *context.local.led_state = LedState::Off;
                    info!("LED is Off");
                }
            };
            // Wait another 300 ms. Release the CPU to other tasks ...
            Mono::delay(300.millis()).await;
        }
    }
}
