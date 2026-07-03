# standard-atmosphere

[![CI](https://github.com/NewEarthTech/standard-atmosphere/actions/workflows/ci.yml/badge.svg)](https://github.com/NewEarthTech/standard-atmosphere/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/standard-atmosphere.svg)](https://crates.io/crates/standard-atmosphere)
[![docs.rs](https://docs.rs/standard-atmosphere/badge.svg)](https://docs.rs/standard-atmosphere)
[![license](https://img.shields.io/crates/l/standard-atmosphere.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.83-blue.svg)](https://www.rust-lang.org)

The International Standard Atmosphere (ISA / U.S. Standard Atmosphere 1976) in Rust, with the vertical-coordinate conversions weather and aviation tooling actually needs: pressure to altitude, and flight level to pressure.

## Why this exists

Gridded weather lives on pressure levels (500 hPa, 300 hPa, 250 hPa). Aviation lives on flight levels (FL300, FL350). They are the same axis. An altimeter set to the standard 1013.25 hPa reads the ISA pressure-altitude of the ambient pressure, so an aircraft at FL300 is sitting on roughly the 301 hPa surface. Converting a flight level to a pressure level to sample a weather grid is not a hack: it is physically what the aircraft is doing. This crate does that conversion correctly, including the piecewise tropopause that a single-formula version gets wrong above FL360.

## Example

```rust
use standard_atmosphere as isa;

// A flight level is, in effect, a real isobaric surface.
assert!((isa::pressure_hpa_from_flight_level(300.0) - 300.9).abs() < 0.5);
assert!((isa::pressure_hpa_from_flight_level(400.0) - 187.5).abs() < 0.5);

// Round-trips both ways.
let fl = isa::flight_level_from_pressure_hpa(238.4);
assert!((fl - 350.0).abs() < 0.5);

// The full model is there too.
let t = isa::temperature_k_from_geopotential_m(11_000.0); // 216.65 K at the tropopause
let rho = isa::density_kg_m3_from_geopotential_m(0.0);     // 1.225 kg/m^3 at sea level
```

## What it covers

- Pressure, temperature, and density at any geopotential altitude, from sea level to 71 km.
- Pressure to and from geopotential altitude (the inverse).
- Flight level to and from pressure, plus pressure-altitude in feet.
- Geopotential to and from geometric altitude (the small Earth-curvature correction the standard defines).

Heights are geopotential metres internally, the coordinate the standard is defined in. Units are explicit in every function name: `_pa`, `_hpa`, `_m`, `_ft`, `_k`. No dependencies, no `unsafe`.

## Accuracy

Values track the published U.S. Standard Atmosphere 1976: sea-level pressure 1013.25 hPa and density 1.225 kg/m^3, the tropopause at 11 km / 226.32 hPa / 216.65 K, and FL300 at 300.9 hPa.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
