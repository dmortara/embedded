# RTIC blinking on Blue Pill #

RTIC (Real Time Interrupt-driven Concurrency) provides an execution framework. It
does not provide any Hardware Abstraction Layer (HAL)

Minimal example blinking onboard LED

This examples uses probe-rs and STLink v2.3 with the last available firmware version
at the time of writing (to support RTT).

```bash
# Build
cargo build --bin blinky --release
# Run (using RTT)
cargo run --bin blinky --release
```
