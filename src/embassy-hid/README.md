# Embassy ADC on Blue Pill #

Embassy provides an execution framework as well as a Hardware Abstraction Layer (HAL)

Analog Digital Converter (12 bit ADC) example using Embassy and the STM32F1xx HAL.

This examples uses probe-rs and STLink v2.3 with the last available firmware version
at the time of writing (to support RTT).

```bash
# Build
cargo build --bin adc0 --release
# Run (using RTT)
cargo run --bin adc0 --release
```

[Embassy](https://embassy.dev/)
