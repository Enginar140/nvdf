//! The table of known unit symbols.

use super::Dimension as D;
use std::f64::consts::PI;

/// A known unit symbol and its conversion to SI.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct UnitDef {
    /// Symbol as written in documents, e.g. `"mAh"`. Symbols are case-sensitive.
    pub symbol: &'static str,
    /// Multiply a value in this unit by `factor` to get SI (radians for angles).
    pub factor: f64,
    /// Dimension of the unit.
    pub dimension: D,
}

const fn u(symbol: &'static str, factor: f64, dimension: D) -> UnitDef {
    UnitDef {
        symbol,
        factor,
        dimension,
    }
}

/// Multiplicative units. Prefixed forms are listed explicitly instead of being
/// derived from a prefix system, so that e.g. `min` or `mm` are never ambiguous.
const UNITS: &[UnitDef] = &[
    // Mass
    u("kg", 1.0, D::MASS),
    u("g", 1e-3, D::MASS),
    // Length
    u("m", 1.0, D::LENGTH),
    u("cm", 1e-2, D::LENGTH),
    u("mm", 1e-3, D::LENGTH),
    u("km", 1e3, D::LENGTH),
    u("ft", 0.3048, D::LENGTH),
    u("in", 0.0254, D::LENGTH),
    // Time
    u("s", 1.0, D::TIME),
    u("ms", 1e-3, D::TIME),
    u("us", 1e-6, D::TIME),
    u("\u{b5}s", 1e-6, D::TIME),  // MICRO SIGN
    u("\u{3bc}s", 1e-6, D::TIME), // GREEK SMALL LETTER MU
    u("min", 60.0, D::TIME),
    u("h", 3600.0, D::TIME),
    // Frequency
    u("Hz", 1.0, D::FREQUENCY),
    u("kHz", 1e3, D::FREQUENCY),
    // Angle and angular velocity
    u("rad", 1.0, D::ANGLE),
    u("deg", PI / 180.0, D::ANGLE),
    u("\u{b0}", PI / 180.0, D::ANGLE), // DEGREE SIGN
    u("rpm", PI / 30.0, D::ANGULAR_VELOCITY),
    // Velocity
    u("kn", 1852.0 / 3600.0, D::VELOCITY),
    // Mechanics
    u("N", 1.0, D::FORCE),
    u("J", 1.0, D::ENERGY),
    u("Wh", 3600.0, D::ENERGY),
    u("W", 1.0, D::POWER),
    u("kW", 1e3, D::POWER),
    u("Pa", 1.0, D::PRESSURE),
    u("hPa", 1e2, D::PRESSURE),
    u("kPa", 1e3, D::PRESSURE),
    u("bar", 1e5, D::PRESSURE),
    // Electrical
    u("V", 1.0, D::VOLTAGE),
    u("mV", 1e-3, D::VOLTAGE),
    u("A", 1.0, D::CURRENT),
    u("mA", 1e-3, D::CURRENT),
    u("Ah", 3600.0, D::CHARGE),
    u("mAh", 3.6, D::CHARGE),
    u("ohm", 1.0, D::RESISTANCE),
    u("\u{3a9}", 1.0, D::RESISTANCE),  // GREEK CAPITAL LETTER OMEGA
    u("\u{2126}", 1.0, D::RESISTANCE), // OHM SIGN
    u("mohm", 1e-3, D::RESISTANCE),
    u("m\u{3a9}", 1e-3, D::RESISTANCE),
    // Other SI base units
    u("K", 1.0, D::TEMPERATURE),
    u("mol", 1.0, D::AMOUNT),
    u("cd", 1.0, D::LUMINOUS_INTENSITY),
    // Ratios
    u("%", 1e-2, D::DIMENSIONLESS),
];

/// Affine (offset) temperature units. Only valid as the whole unit of a value.
const AFFINE: &[(&str, f64)] = &[("degC", 273.15), ("\u{b0}C", 273.15)];

/// All known multiplicative units (excludes affine units such as `degC`).
pub fn known_units() -> &'static [UnitDef] {
    UNITS
}

/// Looks up a multiplicative unit by exact symbol.
pub(super) fn lookup(symbol: &str) -> Option<&'static UnitDef> {
    UNITS.iter().find(|d| d.symbol == symbol)
}

/// Offset to add (after scaling by 1) for an affine unit, e.g. `degC` → 273.15.
pub(super) fn affine_offset(symbol: &str) -> Option<f64> {
    AFFINE.iter().find(|(s, _)| *s == symbol).map(|&(_, o)| o)
}

/// A known symbol that differs from `symbol` only in letter case, if any.
pub(super) fn suggest(symbol: &str) -> Option<&'static str> {
    let lower = symbol.to_lowercase();
    UNITS
        .iter()
        .map(|d| d.symbol)
        .chain(AFFINE.iter().map(|(s, _)| *s))
        .find(|s| s.to_lowercase() == lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbols_are_unique() {
        for (i, a) in UNITS.iter().enumerate() {
            assert!(
                UNITS[i + 1..].iter().all(|b| b.symbol != a.symbol),
                "{}",
                a.symbol
            );
            assert!(affine_offset(a.symbol).is_none());
        }
    }
}
