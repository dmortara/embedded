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

● Fetch(<https://www.allegromicro.com/-/media/files/datasheets/als31001-datasheet.pdf>)
  ⎿  Received 1.1MB (200 OK)

  Read 1 file (ctrl+o to expand)

❯ The last run worked as expected. INFO  LED ON  | pa0=172 pa1=2126 pa2=2469 pa3=2588 pa4=2618 pa5=2652 pa6=2615 pa7=2637 pb0=2404 pb1=2520

● The system reminder already shows it — re-seating the connection fixed it: PA1 = 2126 (≈ 1.71 V, right at the expected quiescent range).

  But the datasheet reveals something important you should fix to prevent the problem recurring. Page 3 of the datasheet specifies:

  ┌─────────────────────────┬───────────────────┬─────────┐
  │        Parameter        │      Symbol       │   Max   │
  ├─────────────────────────┼───────────────────┼─────────┤
  │ DC Output Resistance    │ R_OUT             │ 3 Ω     │
  ├─────────────────────────┼───────────────────┼─────────┤
  │ Output Load Capacitance │ C_L (VOUT to GND) │ 0.22 nF │
  └─────────────────────────┴───────────────────┴─────────┘

  VCC ──┬── sensor Pin 1
        └── 100 nF ── GND     ← bypass cap here (C_BYPASS)

  sensor Pin 3 (VOUT) ── 1 kΩ ── PA1    ← series resistor for ESD
                                  │
                                ≤ 0.22 nF (optional) ── GND

  PA1 = 2126 at quiescent is right on target — that's 1.71 V against a typical QVO of 1.65 V, well within the sensor's ±1.5% ratiometry spec. You should see the full ±1200 count swing when you move the magnet.
