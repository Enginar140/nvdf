//! Errors produced while parsing quantities.

use super::Dimension;
use std::fmt;

/// Error from parsing or checking a quantity string such as `"4.2 kg"`.
///
/// The [`fmt::Display`] messages are written for people editing VDF documents.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum UnitError {
    /// The text is empty or only whitespace.
    Empty,
    /// The text does not start with a valid number (e.g. `"kg"`, `"1e"`).
    BadNumber {
        /// The offending text (trimmed).
        text: String,
    },
    /// The number or the converted value does not fit in an `f64`.
    OutOfRange {
        /// The offending text (trimmed).
        text: String,
    },
    /// A unit symbol is not known.
    UnknownUnit {
        /// The unknown symbol.
        unit: String,
        /// A known symbol differing only in letter case, if any.
        suggestion: Option<&'static str>,
    },
    /// The unit expression is malformed.
    Syntax {
        /// 1-based character column in the original text.
        column: usize,
        /// What was expected or found.
        message: String,
    },
    /// An offset unit such as `degC` was used inside a compound unit.
    AffineMisuse {
        /// The offset unit symbol.
        unit: String,
    },
    /// The unit has a different dimension than the field requires.
    DimensionMismatch {
        /// Dimension the field requires.
        expected: Dimension,
        /// Dimension of the given unit.
        found: Dimension,
    },
}

impl fmt::Display for UnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str(
                "empty value: expected a number, optionally followed by a unit (e.g. \"4.2 kg\")",
            ),
            Self::BadNumber { text } => write!(
                f,
                "invalid number in \"{text}\": a value must start with a number such as \
                 4.2, -3 or 1.5e-3, optionally followed by a unit"
            ),
            Self::OutOfRange { text } => {
                write!(
                    f,
                    "value \"{text}\" is out of range (too large to represent)"
                )
            }
            Self::UnknownUnit { unit, suggestion } => {
                write!(f, "unknown unit \"{unit}\"")?;
                match suggestion {
                    Some(s) => write!(f, "; did you mean \"{s}\"? (units are case-sensitive)"),
                    None => Ok(()),
                }
            }
            Self::Syntax { column, message } => {
                write!(f, "invalid unit at column {column}: {message}")
            }
            Self::AffineMisuse { unit } => write!(
                f,
                "\"{unit}\" can only be used on its own (e.g. \"25 {unit}\") because it is an \
                 offset scale; use \"K\" in compound units such as \"K/s\""
            ),
            Self::DimensionMismatch { expected, found } => write!(
                f,
                "expected {}, found {}",
                expected.describe(),
                found.describe()
            ),
        }
    }
}

impl std::error::Error for UnitError {}
