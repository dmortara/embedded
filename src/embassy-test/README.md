# Embassy blinking on Blue Pill #

Embassy provides an execution framework and a Hardware Abstraction Layer (HAL)

Minimal example blinking onboard LED

This examples uses probe-rs and STLink v2.3 with the last available  firmware version
at the time of writing.

```bash
# Build
cargo build --bin blinky --release
# Run (using RTT)
cargo run --bin blinky --release
```
