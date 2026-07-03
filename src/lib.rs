//! # standard-atmosphere
//!
//! The International Standard Atmosphere (ISA / U.S. Standard Atmosphere 1976), with
//! the vertical-coordinate conversions weather and aviation tooling actually needs:
//! pressure to altitude, and flight level to pressure.
//!
//! ## Flight levels and pressure levels are the same axis
//!
//! Gridded weather lives on pressure levels (500 hPa, 300 hPa, 250 hPa). Aviation lives
//! on flight levels (FL300, FL350). A flight level is the altitude an altimeter reads
//! with the standard subscale set to 1013.25 hPa, and an altimeter reports the ISA
//! pressure-altitude of the ambient static pressure. So an aircraft holding FL300 is
//! sitting wherever the ambient pressure equals the ISA pressure for 30,000 ft, about
//! 300.9 hPa. A flight level is therefore, to a very good approximation, a real isobaric
//! surface: converting a flight level to a pressure level to sample a weather grid is not
//! an approximation, it is physically what the aircraft is doing.
//!
//! ```
//! use standard_atmosphere as isa;
//!
//! // FL300 is the ~301 hPa surface.
//! let p = isa::pressure_hpa_from_flight_level(300.0);
//! assert!((p - 300.9).abs() < 0.5);
//!
//! // ...and back.
//! let fl = isa::flight_level_from_pressure_hpa(300.9);
//! assert!((fl - 300.0).abs() < 0.5);
//! ```
//!
//! ## Model
//!
//! The atmosphere is the standard's stack of layers, each with a constant temperature
//! lapse rate, integrated hydrostatically, from sea level to 71 km geopotential. That
//! span covers all aviation and tropospheric/stratospheric meteorology. The piecewise
//! structure matters: a single tropospheric formula used above the 11 km tropopause
//! (about FL360) is wrong, which is the classic bug this crate avoids.
//!
//! Heights are geopotential metres internally, the coordinate the standard is defined in.
//! [`geometric_from_geopotential`] and [`geopotential_from_geometric`] convert to and from
//! true geometric altitude when you need it; the difference is about 0.3% at 20 km.
//!
//! Units are explicit in every function name: `_pa` pascals, `_hpa` hectopascals
//! (millibars), `_m` metres, `_ft` feet, `_k` kelvin. SI is the core; the hPa and
//! flight-level helpers wrap it. No dependencies, no `unsafe`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Standard gravitational acceleration (m/s^2).
pub const G0: f64 = 9.806_65;
/// Molar mass of dry air (kg/mol), per the 1976 standard.
pub const M_AIR: f64 = 0.028_964_4;
/// Universal gas constant used by the 1976 standard (J/(mol*K)).
pub const R_STAR: f64 = 8.314_32;
/// Specific gas constant for dry air, `R_STAR / M_AIR` (J/(kg*K)).
pub const R_AIR: f64 = R_STAR / M_AIR;
/// Hydrostatic constant `G0 * M_AIR / R_STAR` ("GMR"), in K/m.
pub const GMR: f64 = G0 * M_AIR / R_STAR;
/// Effective Earth radius for the geopotential/geometric conversion (m), per the standard.
pub const EARTH_RADIUS_M: f64 = 6_356_766.0;
/// Sea-level standard pressure (Pa).
pub const P0_PA: f64 = 101_325.0;
/// Sea-level standard temperature (K).
pub const T0_K: f64 = 288.15;

/// Feet per metre.
const FT_PER_M: f64 = 1.0 / 0.3048;

/// One atmospheric layer: geopotential base height, base temperature, lapse rate, and
/// the base pressure (the standard's tabulated value, from hydrostatic integration).
struct Layer {
    base_geopotential_m: f64,
    base_temp_k: f64,
    lapse_k_per_m: f64,
    base_pressure_pa: f64,
}

/// ISA / U.S. Standard Atmosphere 1976 layers, sea level to 71 km geopotential.
#[rustfmt::skip]
static LAYERS: [Layer; 7] = [
    Layer { base_geopotential_m:      0.0, base_temp_k: 288.15, lapse_k_per_m: -0.006_5, base_pressure_pa: 101_325.0 },
    Layer { base_geopotential_m: 11_000.0, base_temp_k: 216.65, lapse_k_per_m:  0.0,     base_pressure_pa:  22_632.06 },
    Layer { base_geopotential_m: 20_000.0, base_temp_k: 216.65, lapse_k_per_m:  0.001,   base_pressure_pa:   5_474.889 },
    Layer { base_geopotential_m: 32_000.0, base_temp_k: 228.65, lapse_k_per_m:  0.002_8, base_pressure_pa:     868.018_7 },
    Layer { base_geopotential_m: 47_000.0, base_temp_k: 270.65, lapse_k_per_m:  0.0,     base_pressure_pa:     110.906_3 },
    Layer { base_geopotential_m: 51_000.0, base_temp_k: 270.65, lapse_k_per_m: -0.002_8, base_pressure_pa:      66.938_87 },
    Layer { base_geopotential_m: 71_000.0, base_temp_k: 214.65, lapse_k_per_m: -0.002,   base_pressure_pa:       3.956_420 },
];

fn layer_for_geopotential(h_m: f64) -> &'static Layer {
    let mut chosen = &LAYERS[0];
    for layer in LAYERS.iter() {
        if h_m >= layer.base_geopotential_m {
            chosen = layer;
        } else {
            break;
        }
    }
    chosen
}

fn layer_for_pressure(p_pa: f64) -> &'static Layer {
    // Base pressures descend with altitude; the containing layer is the deepest one
    // whose base pressure is still at or above p.
    let mut chosen = &LAYERS[0];
    for layer in LAYERS.iter() {
        if p_pa <= layer.base_pressure_pa {
            chosen = layer;
        } else {
            break;
        }
    }
    chosen
}

/// Temperature at a geopotential altitude (kelvin).
pub fn temperature_k_from_geopotential_m(h_m: f64) -> f64 {
    let l = layer_for_geopotential(h_m);
    l.base_temp_k + l.lapse_k_per_m * (h_m - l.base_geopotential_m)
}

/// Pressure at a geopotential altitude (pascals).
pub fn pressure_pa_from_geopotential_m(h_m: f64) -> f64 {
    let l = layer_for_geopotential(h_m);
    let dh = h_m - l.base_geopotential_m;
    if l.lapse_k_per_m.abs() < f64::EPSILON {
        // Isothermal layer.
        l.base_pressure_pa * (-GMR * dh / l.base_temp_k).exp()
    } else {
        // Constant-lapse layer: P = P_b * (T_b / T)^(GMR / L).
        let t = l.base_temp_k + l.lapse_k_per_m * dh;
        l.base_pressure_pa * (l.base_temp_k / t).powf(GMR / l.lapse_k_per_m)
    }
}

/// Pressure at a geopotential altitude (hectopascals / millibars).
pub fn pressure_hpa_from_geopotential_m(h_m: f64) -> f64 {
    pressure_pa_from_geopotential_m(h_m) / 100.0
}

/// Geopotential altitude (metres) for a given pressure (pascals). Inverse of
/// [`pressure_pa_from_geopotential_m`].
pub fn geopotential_m_from_pressure_pa(p_pa: f64) -> f64 {
    let l = layer_for_pressure(p_pa);
    if l.lapse_k_per_m.abs() < f64::EPSILON {
        l.base_geopotential_m - (l.base_temp_k / GMR) * (p_pa / l.base_pressure_pa).ln()
    } else {
        let t = l.base_temp_k * (p_pa / l.base_pressure_pa).powf(-l.lapse_k_per_m / GMR);
        l.base_geopotential_m + (t - l.base_temp_k) / l.lapse_k_per_m
    }
}

/// Air density at a geopotential altitude (kg/m^3), from the ideal gas law.
pub fn density_kg_m3_from_geopotential_m(h_m: f64) -> f64 {
    let p = pressure_pa_from_geopotential_m(h_m);
    let t = temperature_k_from_geopotential_m(h_m);
    p / (R_AIR * t)
}

/// Convert geometric altitude (true height above sea level) to geopotential altitude.
pub fn geopotential_from_geometric(z_m: f64) -> f64 {
    EARTH_RADIUS_M * z_m / (EARTH_RADIUS_M + z_m)
}

/// Convert geopotential altitude to geometric altitude (true height above sea level).
pub fn geometric_from_geopotential(h_m: f64) -> f64 {
    EARTH_RADIUS_M * h_m / (EARTH_RADIUS_M - h_m)
}

/// Pressure (hPa) at a pressure-altitude given as a flight level (hundreds of feet).
///
/// `FL300` maps to about 300.9 hPa. A flight level is pressure-altitude, which is the
/// geopotential altitude in the ISA, so this is the pressure level a weather grid should
/// be sampled at for that flight level. See the crate docs for why that is exact, not an
/// approximation.
pub fn pressure_hpa_from_flight_level(fl: f64) -> f64 {
    let h_m = fl * 100.0 / FT_PER_M; // hundreds of feet -> metres (pressure-altitude == geopotential)
    pressure_pa_from_geopotential_m(h_m) / 100.0
}

/// Flight level (hundreds of feet) for a given pressure (hPa). Inverse of
/// [`pressure_hpa_from_flight_level`].
pub fn flight_level_from_pressure_hpa(p_hpa: f64) -> f64 {
    let h_m = geopotential_m_from_pressure_pa(p_hpa * 100.0);
    h_m * FT_PER_M / 100.0
}

/// Pressure-altitude in feet for a given pressure (hPa): the value an altimeter set to
/// the standard 1013.25 hPa would display.
pub fn pressure_altitude_ft_from_pressure_hpa(p_hpa: f64) -> f64 {
    geopotential_m_from_pressure_pa(p_hpa * 100.0) * FT_PER_M
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() < tol, "{a} vs {b} (tol {tol})");
    }

    #[test]
    fn sea_level_matches_the_standard() {
        approx(pressure_hpa_from_geopotential_m(0.0), 1013.25, 0.01);
        approx(temperature_k_from_geopotential_m(0.0), 288.15, 0.01);
        approx(density_kg_m3_from_geopotential_m(0.0), 1.225, 0.001);
    }

    #[test]
    fn flight_levels_are_isobars() {
        approx(pressure_hpa_from_flight_level(300.0), 300.9, 0.3);
        approx(pressure_hpa_from_flight_level(350.0), 238.4, 0.3);
        approx(pressure_hpa_from_flight_level(400.0), 187.5, 0.3);
    }

    #[test]
    fn flight_level_round_trips() {
        for fl in [50.0, 180.0, 300.0, 410.0] {
            let p = pressure_hpa_from_flight_level(fl);
            approx(flight_level_from_pressure_hpa(p), fl, 0.01);
        }
    }

    #[test]
    fn known_pressure_heights() {
        // 500 hPa ~ 5575 m geopotential, 250 hPa ~ 10,363 m: standard reference values.
        approx(geopotential_m_from_pressure_pa(50_000.0), 5575.0, 5.0);
        approx(geopotential_m_from_pressure_pa(25_000.0), 10_363.0, 10.0);
    }

    #[test]
    fn tropopause_is_piecewise() {
        // The 11 km tropopause: a single tropospheric formula would diverge above here.
        approx(pressure_hpa_from_geopotential_m(11_000.0), 226.32, 0.05);
        approx(temperature_k_from_geopotential_m(11_000.0), 216.65, 0.01);
        // Into the isothermal stratosphere, temperature holds.
        approx(temperature_k_from_geopotential_m(18_000.0), 216.65, 0.01);
    }

    #[test]
    fn geopotential_geometric_correction_is_small_but_present() {
        let z = geometric_from_geopotential(11_000.0);
        assert!(z > 11_000.0 && z - 11_000.0 < 25.0); // about 19 m at 11 km
        approx(geopotential_from_geometric(z), 11_000.0, 1e-6);
    }
}
