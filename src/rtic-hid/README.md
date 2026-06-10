# RTIC ADC on Blue Pill #

## Summary ##

RTIC (Real Time Interrupt-driven Concurrency) provides an execution framework. It
does not provide any Hardware Abstraction Layer (HAL)

Analog Digital Converter (12 bit ADC) example using RTIC and the STM32F1xx HAL.

This examples uses probe-rs and STLink v2.3 with the last available firmware version
at the time of writing (to support RTT).

Analog Digital Converter (ADC) using:

* 10K Ohm potentiometer connected to __PA0__ (To compare to)
* Allegro _ALS31001LUAA_ Hall effect sensor connected to ___PA1___
* pa0-pa7, pb0, pb1 for analog input connected to ADC0 (12 bits per ch)

## Compile and run ##

```bash
# Build
cargo build --bin adc0 --release
# Run (using RTT)
cargo run --bin adc0 --release
```

[RTIC](https://rtic.rs/2/book/en/)

● [ALS31001 3.3v Hall Effect](<https://www.allegromicro.com/-/media/files/datasheets/als31001-datasheet.pdf>) connected as:

```bash
  VCC ──┬── sensor Pin 1 (3.3V)
        └── 100 nF ── GND (104M) ← bypass cap here (C_BYPASS)  

  Pin 2 (GND)

  Sensor Pin 3 (VOUT) ── 1 kΩ ── PA1 ← series resistor for ESD  
                                  │  
                                ≤ 0.22 nF (optional 221K) ── GND  
```

## Current settings ##

  PA1 = 2126 at quiescent is right on target — that's 1.71 V against a typical
  QVO of 1.65 V, well within the sensor's ±1.5% ratiometry spec.  
  Full ±1200 count swing when you move the magnet depends on the
  magnetic field (__1.8 mV/G__ from datasheet)

## Send HID info to USB as HID ##

 Analog inputs (12bits) shifted 4 bits to the left and sent as 10 ch (16 bits)
 joystick HID report (20 bytes) to USB host using the `usbd-hid` crate.
