# RTIC ADC on Blue Pill #

RTIC (Real Time Interrupt-driven Concurrency) provides an execution framework. It
does not provide any Hardware Abstraction Layer (HAL)

Analog Digital Converter (12 bit ADC) example using RTIC and the STM32F1xx HAL.

This examples uses probe-rs and STLink v2.3 with the last available firmware version
at the time of writing (to support RTT).

Analog Digital Converter (ADC) using:

* 10K Ohm potentiometer connected to _PA0_
* Allegro __ALS31001LUAA__ Hall effect sensor connected to _PA1_

```bash
# Build
cargo build --bin adc0 --release
# Run (using RTT)
cargo run --bin adc0 --release
```

[RTIC](https://rtic.rs/2/book/en/)
