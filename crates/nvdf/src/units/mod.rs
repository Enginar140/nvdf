//! Physical quantities with units: parsing and dimensional checking.
//!
//! VDF stores **plain numbers in SI** (angles in radians). Where a document gives
//! a string with a unit, such as `"4.2 kg"`, `"30 deg"`, `"1500 mAh"` or
//! `"700 rpm/V"`, it is converted to SI at load time and its dimension is
//! checked against the field.
//!
//! ```
//! use nvdf::units::{Dimension, parse_as, parse_quantity};
//!
//! assert_eq!(parse_as("1500 mAh", Dimension::CHARGE).unwrap(), 5400.0);
//! assert_eq!(parse_as("0.25", Dimension::ANGLE).unwrap(), 0.25); // plain number: radians
//! assert!(parse_as("3 kg", Dimension::LENGTH).is_err());
//!
//! let q = parse_quantity("9.81 m/s^2").unwrap();
//! assert_eq!(q.dim, Dimension::ACCELERATION);
//! ```
//!
//! # Unit expressions
//!
//! A unit is a product of known symbols joined by `*` and `/`, evaluated left to
//! right, each optionally raised to an integer power with `^`: `m/s^2`, `kg*m^2`,
//! `N*m`, `rpm/V`. `sqrt(...)` gives half exponents for noise densities, e.g.
//! `deg/sqrt(h)` or `m/s/sqrt(Hz)`. Parentheses may group, and `1/s` is accepted.
//! Symbols are case-sensitive and listed by [`known_units`]. The offset scale
//! `degC` (`°C`) is accepted only as the whole unit, e.g. `"25 degC"`.
//!
//! # Angles
//!
//! Angle is tracked as its own dimension (see [`BaseDimension::Angle`]), so `rpm`
//! (angular velocity) and `Hz` (frequency) are not interchangeable, and `rad` does
//! not satisfy a dimensionless field.

mod dimension;
mod error;
mod parse;
mod table;

use std::fmt;
use std::str::FromStr;

pub use dimension::{BaseDimension, Dimension};
pub use error::UnitError;
pub use table::{UnitDef, known_units};

/// A value in SI units together with its dimension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity {
    /// Value in SI units (radians for angles, kelvin for temperatures).
    pub value: f64,
    /// Dimension of the value.
    pub dim: Dimension,
}

impl Quantity {
    /// Returns the SI value if the dimension is `expected`.
    pub fn expect(self, expected: Dimension) -> Result<f64, UnitError> {
        if self.dim == expected {
            Ok(self.value)
        } else {
            Err(UnitError::DimensionMismatch {
                expected,
                found: self.dim,
            })
        }
    }
}

/// Formats as the SI value followed by its base-unit expression, e.g.
/// `"2.5 m/s^2"`; the output is accepted by [`parse_quantity`].
impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dim.is_dimensionless() {
            write!(f, "{}", self.value)
        } else {
            write!(f, "{} {}", self.value, self.dim)
        }
    }
}

impl FromStr for Quantity {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_quantity(s)
    }
}

/// Parses a quantity string such as `"4.2 kg"` into an SI value and dimension.
///
/// A plain number without a unit is returned as [`Dimension::DIMENSIONLESS`];
/// use [`parse_as`] to check against a field, where a plain number is accepted
/// for any dimension.
pub fn parse_quantity(s: &str) -> Result<Quantity, UnitError> {
    let parsed = parse::parse(s)?;
    Ok(Quantity {
        value: parsed.value,
        dim: parsed.unit.unwrap_or(Dimension::DIMENSIONLESS),
    })
}

/// Parses `s` for a field of dimension `expected` and returns its SI value.
///
/// A plain number (no unit) is accepted for any dimension and taken as already
/// being in SI, i.e. radians for angles. A unit with a different dimension is a
/// [`UnitError::DimensionMismatch`].
pub fn parse_as(s: &str, expected: Dimension) -> Result<f64, UnitError> {
    let parsed = parse::parse(s)?;
    match parsed.unit {
        None => Ok(parsed.value),
        Some(dim) => Quantity {
            value: parsed.value,
            dim,
        }
        .expect(expected),
    }
}
