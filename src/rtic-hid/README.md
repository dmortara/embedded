# RTIC ADC on Blue Pill #

RTIC (Real Time Interrupt-driven Concurrency) provides an execution framework. It
does not provide any Hardware Abstraction Layer (HAL)

Analog Digital Converter (12 bit ADC) example using RTIC and the STM32F1xx HAL.

This examples uses probe-rs and STLink v2.3 with the last available firmware version
at the time of writing (to support RTT).

Analog Digital Converter (ADC) using:

* 10K Ohm potentiometer connected to __PA0__
* Allegro _ALS31001LUAA_ Hall effect sensor connected to ___PA1___

```bash
# Build
cargo build --bin adc0 --release
# Run (using RTT)
cargo run --bin adc0 --release
```

[RTIC](https://rtic.rs/2/book/en/)

## Summary ##

  ┌──────────────────────────────┬─────────────────────┬──────────────────────────────┐
  │          Component           │ Blue Pill provides? │           Keep it?           │
  ├──────────────────────────────┼─────────────────────┼──────────────────────────────┤
  │ Series resistor (100–1kΩ)    │ No                  │ Yes — protection + RC filter │
  ├──────────────────────────────┼─────────────────────┼──────────────────────────────┤
  │ Filter capacitor (10–100nF)  │ No                  │ Yes — noise rejection        │
  ├──────────────────────────────┼─────────────────────┼──────────────────────────────┤
  │ VCC decoupling cap on sensor │ No                  │ Yes — clean sensor supply    │
  └──────────────────────────────┴─────────────────────┴──────────────────────────────┘

  The combination of a ~1 kΩ resistor + 10–100 nF cap gives you a low-pass
  corner around 1.6–16 kHz, which is more than fast enough for a Hall effect
  reading updated every 300 ms, and well within the ADC's 9 MHz clock budget.
