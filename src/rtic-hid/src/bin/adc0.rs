/*
 *  Copyright (c) 2026 Mortara Ltd
 *  Daniel Mortara <daniel.mortara@gmail.com>
 *  2026-06-10
 *
 * ADC + USB HID Joystick for STM32F103C8T6 (Blue Pill)
 * Samples all 10 ADC channels (PA0-PA7, PB0, PB1) and reports them as a
 * standard USB joystick (Generic Desktop, Joystick) at 1 kHz for XPlane 12.
 *
 * ADC clock: 9 MHz, sample time T_239 (~28 µs/ch, ~280 µs for 10 ch)
 * USB HID poll: 1 ms (maximum rate for Full Speed USB)
 * LED / console log: every 300 reports ≈ 300 ms (human-readable)
 *
 * Axis map (all 12-bit, range 0–4095):
 *   X   = PA0  |  Y      = PA1  |  Z      = PA2
 *   Rx  = PA3  |  Ry     = PA4  |  Rz     = PA5
 *   Slider= PA6|  Dial   = PA7  |  Wheel  = PB0  |  Slider2 = PB1
 */
#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m::asm::delay;
use defmt::info;
use rtic::app;
use rtic_monotonics::systick::prelude::*;
use {defmt_rtt as _, panic_probe as _};

use stm32f1xx_hal::{
    adc,
    gpio::{
        Analog, Output, PinState, PushPull, PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PB0, PB1, PC13,
    },
    pac::ADC1,
    prelude::*,
    usb::{Peripheral, UsbBus, UsbBusType},
};
use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_hid::{descriptor::generator_prelude::*, hid_class::HIDClass};

// 1 kHz tick → 1 ms minimum delay, matching the USB HID polling interval.
systick_monotonic!(Mono, 1000);

// Standard USB HID Joystick — Generic Desktop page (0x01), Joystick usage (0x04).
// Ten u16 axes; logical range 0–65535 is derived from the type by the macro.
// ADC values (12-bit, 0–4095) are shifted left 4 to fill the full u16 range.
//
// Each axis gets its own USAGE local item so the HID host assigns the correct
// usage regardless of REPORT_COUNT (local items are consumed per main item).
// Axis usages: 0x30 X · 0x31 Y · 0x32 Z · 0x33 Rx · 0x34 Ry · 0x35 Rz
//              0x36 Slider · 0x37 Dial · 0x38 Wheel · 0x36 Slider (2nd)
#[gen_hid_descriptor(
    (collection = 0x01, usage_page = 0x01, usage = 0x04) = {
        (usage = 0x30,) = { x=input;      };
        (usage = 0x31,) = { y=input;      };
        (usage = 0x32,) = { z=input;      };
        (usage = 0x33,) = { rx=input;     };
        (usage = 0x34,) = { ry=input;     };
        (usage = 0x35,) = { rz=input;     };
        (usage = 0x36,) = { slider=input; };
        (usage = 0x37,) = { dial=input;   };
        (usage = 0x38,) = { wheel=input;  };
        (usage = 0x36,) = { slider2=input;};
    }
)]
pub struct JoystickReport {
    pub x: u16,
    pub y: u16,
    pub z: u16,
    pub rx: u16,
    pub ry: u16,
    pub rz: u16,
    pub slider: u16,
    pub dial: u16,
    pub wheel: u16,
    pub slider2: u16,
}

#[app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use super::*;

    struct AnalogPins {
        pa0: PA0<Analog>,
        pa1: PA1<Analog>,
        pa2: PA2<Analog>,
        pa3: PA3<Analog>,
        pa4: PA4<Analog>,
        pa5: PA5<Analog>,
        pa6: PA6<Analog>,
        pa7: PA7<Analog>,
        pb0: PB0<Analog>,
        pb1: PB1<Analog>,
    }

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, UsbBusType>,
        hid: HIDClass<'static, UsbBusType>, // To report the Joystick readings over USB
    }

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>, // For development feedback
        adc1: adc::Adc<ADC1>,
        pins: AnalogPins, // To connect to the Hall effect sensors
    }

    #[init(local = [usb_bus: Option<UsbBusAllocator<UsbBusType>> = None])]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let mut flash = ctx.device.FLASH.constrain();

        let mut rcc = ctx.device.RCC.freeze(
            stm32f1xx_hal::rcc::Config::hse(8.MHz())
                .sysclk(72.MHz())
                .pclk1(36.MHz())
                .adcclk(9.MHz()),
            &mut flash.acr,
        );

        assert!(rcc.clocks.usbclk_valid());

        // The monothonic timer needs the same frequnecy as the system clock
        Mono::start(ctx.core.SYST, 72_000_000);

        let mut gpioa = ctx.device.GPIOA.split(&mut rcc);
        let mut gpiob = ctx.device.GPIOB.split(&mut rcc);
        let mut gpioc = ctx.device.GPIOC.split(&mut rcc);

        // Pull D+ low to force host re-enumeration when firmware is reflashed.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low();
        delay(rcc.clocks.sysclk().raw() / 100);
        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        let usb = Peripheral {
            usb: ctx.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };
        ctx.local.usb_bus.replace(UsbBus::new(usb));
        let usb_bus = ctx.local.usb_bus.as_ref().unwrap();

        // 1 ms USB polling interval — maximum rate for Full Speed USB.
        let hid = HIDClass::new(usb_bus, JoystickReport::desc(), 1);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dc))
            .strings(&[StringDescriptors::default()
                .manufacturer("Mortara Limited")
                .product("Hall Effect Joystick")
                .serial_number("001")])
            .unwrap()
            .build();

        let mut adc1 = adc::Adc::new(ctx.device.ADC1, &mut rcc);
        // T_239 = 239.5 ADC cycles @ 9 MHz ≈ 26.6 µs/channel. Overrides HAL default T_71
        // (~7.9 µs) which is insufficient for high-impedance Hall effect sensors to settle.
        adc1.set_sample_time(adc::SampleTime::T_239);

        let pa0 = gpioa.pa0.into_analog(&mut gpioa.crl);
        let pa1 = gpioa.pa1.into_analog(&mut gpioa.crl);
        let pa2 = gpioa.pa2.into_analog(&mut gpioa.crl);
        let pa3 = gpioa.pa3.into_analog(&mut gpioa.crl);
        let pa4 = gpioa.pa4.into_analog(&mut gpioa.crl);
        let pa5 = gpioa.pa5.into_analog(&mut gpioa.crl);
        let pa6 = gpioa.pa6.into_analog(&mut gpioa.crl);
        let pa7 = gpioa.pa7.into_analog(&mut gpioa.crl);
        let pb0 = gpiob.pb0.into_analog(&mut gpiob.crl);
        let pb1 = gpiob.pb1.into_analog(&mut gpiob.crl);
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::Low);

        let pins = AnalogPins {
            pa0,
            pa1,
            pa2,
            pa3,
            pa4,
            pa5,
            pa6,
            pa7,
            pb0,
            pb1,
        };

        info!("Hall Effect Joystick initialised — 1 kHz USB reports");
        read_and_report::spawn().ok();

        (Shared { usb_dev, hid }, Local { led, adc1, pins })
    }

    // Both USB interrupt vectors must poll the USB device to keep enumeration alive.
    #[task(binds = USB_HP_CAN_TX, shared = [usb_dev, hid])]
    fn usb_hp(ctx: usb_hp::Context) {
        let mut usb_dev = ctx.shared.usb_dev;
        let mut hid = ctx.shared.hid;
        (&mut usb_dev, &mut hid).lock(|usb_dev, hid| {
            usb_dev.poll(&mut [hid]);
        });
    }

    #[task(binds = USB_LP_CAN_RX0, shared = [usb_dev, hid])]
    fn usb_lp(ctx: usb_lp::Context) {
        let mut usb_dev = ctx.shared.usb_dev;
        let mut hid = ctx.shared.hid;
        (&mut usb_dev, &mut hid).lock(|usb_dev, hid| {
            usb_dev.poll(&mut [hid]);
        });
    }

    // Sample all 10 ADC channels and push one HID joystick report every 1 ms (1 kHz).
    // LED and defmt console are updated every 300 reports (≈ 300 ms) to remain readable.
    #[task(local = [led, adc1, pins, tick: u16 = 0], shared = [hid])]
    async fn read_and_report(mut ctx: read_and_report::Context) {
        loop {
            let adc = &mut *ctx.local.adc1;
            let p = &mut *ctx.local.pins;

            // Shift 12-bit ADC values (0–4095) left 4 to fill the u16 range (0–65520).
            // The descriptor's logical_max is 65535 (u16), so this fills ~99.97 % of the axis.
            let report = JoystickReport {
                x: adc.read(&mut p.pa0).unwrap_or(0u16) << 4,
                y: adc.read(&mut p.pa1).unwrap_or(0u16) << 4,
                z: adc.read(&mut p.pa2).unwrap_or(0u16) << 4,
                rx: adc.read(&mut p.pa3).unwrap_or(0u16) << 4,
                ry: adc.read(&mut p.pa4).unwrap_or(0u16) << 4,
                rz: adc.read(&mut p.pa5).unwrap_or(0u16) << 4,
                slider: adc.read(&mut p.pa6).unwrap_or(0u16) << 4,
                dial: adc.read(&mut p.pa7).unwrap_or(0u16) << 4,
                wheel: adc.read(&mut p.pb0).unwrap_or(0u16) << 4,
                slider2: adc.read(&mut p.pb1).unwrap_or(0u16) << 4,
            };

            ctx.shared.hid.lock(|hid| {
                hid.push_input(&report).ok();
            });

            *ctx.local.tick += 1; // Used for console output and switching the LED
            if *ctx.local.tick >= 300 {
                *ctx.local.tick = 0;

                // Copy out of the packed struct before logging to avoid unaligned references.
                let (x, y, z, rx, ry, rz, sl, di, wh, sl2) = (
                    report.x,
                    report.y,
                    report.z,
                    report.rx,
                    report.ry,
                    report.rz,
                    report.slider,
                    report.dial,
                    report.wheel,
                    report.slider2,
                );

                if ctx.local.led.is_set_high() {
                    ctx.local.led.set_low();
                    info!("LED OFF | x={:04} y={:04} z={:04} rx={:04} ry={:04} rz={:04} sl={:04} di={:04} wh={:04} sl2={:04}",
                        x, y, z, rx, ry, rz, sl, di, wh, sl2);
                } else {
                    ctx.local.led.set_high();
                    info!("LED ON  | x={:04} y={:04} z={:04} rx={:04} ry={:04} rz={:04} sl={:04} di={:04} wh={:04} sl2={:04}",
                        x, y, z, rx, ry, rz, sl, di, wh, sl2);
                }
            }

            // Roughly 1ms + sampling time (≈ 28 µs/ch × 10 ch = 280 µs) ≈ 1.3 ms total per loop
            // iteration.
            Mono::delay(1.millis()).await;
        }
    }
}
