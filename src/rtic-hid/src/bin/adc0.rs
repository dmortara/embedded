/*
 * ADC + USB HID for STM32F103C8T6 (Blue Pill)
 * Samples all 10 ADC channels (PA0-PA7, PB0, PB1) and streams
 * the values to a PC as a vendor-defined HID report at 10 ms intervals.
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

systick_monotonic!(Mono, 300);

// Vendor-defined HID report — 10 × u16, one per ADC channel.
// On the PC side read with hidapi: report is 20 raw bytes, each pair is one channel (little-endian).
// Channel map: ch0=PA0, ch1=PA1 ... ch7=PA7, ch8=PB0, ch9=PB1
#[gen_hid_descriptor(
    (collection = 0x01, usage = 0x01, usage_page = 0xff00) = {
        ch0=input; ch1=input; ch2=input; ch3=input; ch4=input;
        ch5=input; ch6=input; ch7=input; ch8=input; ch9=input;
    }
)]
pub struct SensorReport {
    pub ch0: u16,
    pub ch1: u16,
    pub ch2: u16,
    pub ch3: u16,
    pub ch4: u16,
    pub ch5: u16,
    pub ch6: u16,
    pub ch7: u16,
    pub ch8: u16,
    pub ch9: u16,
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
        hid: HIDClass<'static, UsbBusType>,
    }

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        adc1: adc::Adc<ADC1>,
        pins: AnalogPins,
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

        // The monotonic timer need to match the system clock.
        Mono::start(ctx.core.SYST, 72_000_000);

        let mut gpioa = ctx.device.GPIOA.split(&mut rcc);
        let mut gpiob = ctx.device.GPIOB.split(&mut rcc);
        let mut gpioc = ctx.device.GPIOC.split(&mut rcc);

        // Pull D+ low to force host re-enumeration when firmware is reflashed
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

        let hid = HIDClass::new(usb_bus, SensorReport::desc(), 10);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dc))
            .strings(&[StringDescriptors::default()
                .manufacturer("DIY")
                .product("Hall Sensor HID")
                .serial_number("001")])
            .unwrap()
            .build();

        let adc1 = adc::Adc::new(ctx.device.ADC1, &mut rcc);

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

        info!("Hall Sensor HID initialised");
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

    // Read all 10 ADC channels and push one HID report every 10 ms.
    #[task(local = [led, adc1, pins], shared = [hid])]
    async fn read_and_report(mut ctx: read_and_report::Context) {
        //let mut infoLoop = 0;
        loop {
            let adc = &mut *ctx.local.adc1;
            let p = &mut *ctx.local.pins;

            let report = SensorReport {
                ch0: adc.read(&mut p.pa0).unwrap_or(0),
                ch1: adc.read(&mut p.pa1).unwrap_or(0),
                ch2: adc.read(&mut p.pa2).unwrap_or(0),
                ch3: adc.read(&mut p.pa3).unwrap_or(0),
                ch4: adc.read(&mut p.pa4).unwrap_or(0),
                ch5: adc.read(&mut p.pa5).unwrap_or(0),
                ch6: adc.read(&mut p.pa6).unwrap_or(0),
                ch7: adc.read(&mut p.pa7).unwrap_or(0),
                ch8: adc.read(&mut p.pb0).unwrap_or(0),
                ch9: adc.read(&mut p.pb1).unwrap_or(0),
            };

            ctx.shared.hid.lock(|hid| {
                hid.push_input(&report).ok();
            });

            // Copy out of the packed struct before logging to avoid unaligned references.
            let (c0, c1, c2, c3, c4, c5, c6, c7, c8, c9) = (
                report.ch0, report.ch1, report.ch2, report.ch3, report.ch4, report.ch5, report.ch6,
                report.ch7, report.ch8, report.ch9,
            );

            //if infoLoop == 5 {
            //    infoLoop = 0;
            if ctx.local.led.is_set_high() {
                ctx.local.led.set_low();
                info!("LED OFF | pa0={} pa1={} pa2={} pa3={} pa4={} pa5={} pa6={} pa7={} pb0={} pb1={}",
                    c0, c1, c2, c3, c4, c5, c6, c7, c8, c9);
            } else {
                ctx.local.led.set_high();
                info!("LED ON  | pa0={} pa1={} pa2={} pa3={} pa4={} pa5={} pa6={} pa7={} pb0={} pb1={}",
                    c0, c1, c2, c3, c4, c5, c6, c7, c8, c9);
            }
            //}
            //infoLoop += 1;

            Mono::delay(300.millis()).await;
        }
    }
}
