//! Physical dimensions: exponents over the SI base dimensions plus angle.

use std::fmt;

/// Number of tracked base dimensions (seven SI base dimensions plus angle).
const N: usize = 8;

/// A base dimension that a [`Dimension`] has an exponent over.
///
/// The first seven are the SI base dimensions. [`BaseDimension::Angle`] is a
/// pseudo-dimension: SI treats the radian as dimensionless, but VDF tracks it
/// separately so that `"30 deg"` is rejected where a length is expected and an
/// angular velocity (`rad/s`) is distinguishable from a frequency (`Hz`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseDimension {
    /// Mass, SI unit kilogram (`kg`).
    Mass,
    /// Length, SI unit metre (`m`).
    Length,
    /// Time, SI unit second (`s`).
    Time,
    /// Electric current, SI unit ampere (`A`).
    Current,
    /// Thermodynamic temperature, SI unit kelvin (`K`).
    Temperature,
    /// Amount of substance, SI unit mole (`mol`).
    Amount,
    /// Luminous intensity, SI unit candela (`cd`).
    Luminous,
    /// Plane angle, unit radian (`rad`). Not an SI base dimension; see the type docs.
    Angle,
}

impl BaseDimension {
    /// All base dimensions, in storage order.
    pub const ALL: [BaseDimension; N] = [
        Self::Mass,
        Self::Length,
        Self::Time,
        Self::Current,
        Self::Temperature,
        Self::Amount,
        Self::Luminous,
        Self::Angle,
    ];

    /// Symbol of the SI (or, for angle, coherent) unit of this base dimension.
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Mass => "kg",
            Self::Length => "m",
            Self::Time => "s",
            Self::Current => "A",
            Self::Temperature => "K",
            Self::Amount => "mol",
            Self::Luminous => "cd",
            Self::Angle => "rad",
        }
    }

    /// Human-readable name, e.g. `"length"`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Mass => "mass",
            Self::Length => "length",
            Self::Time => "time",
            Self::Current => "electric current",
            Self::Temperature => "temperature",
            Self::Amount => "amount of substance",
            Self::Luminous => "luminous intensity",
            Self::Angle => "angle",
        }
    }
}

/// Physical dimension of a quantity.
///
/// # Representation
///
/// Each exponent is stored **doubled** as an `i8` ("half-exponents"), so that
/// half-integer exponents are exact. This is needed for noise densities such as
/// `deg/sqrt(h)` (angle · time^-1/2) or `m/s/sqrt(Hz)` (length · time^-1/2).
/// Velocity (`m/s`) is therefore stored as length = 2, time = -2. The
/// representable exponent range is -64 ..= 63.5, far beyond physical use;
/// arithmetic that would leave it is reported as an error, never wrapped.
///
/// Two quantities are compatible only if all exponents match exactly,
/// including the angle pseudo-dimension (see [`BaseDimension::Angle`]).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Dimension {
    half: [i8; N],
}

impl Dimension {
    /// Builds a dimension from integer exponents (mass, length, time, current,
    /// temperature, angle); amount and luminous intensity are zero.
    const fn si(m: i8, l: i8, t: i8, i: i8, th: i8, a: i8) -> Self {
        Self {
            half: [2 * m, 2 * l, 2 * t, 2 * i, 2 * th, 0, 0, 2 * a],
        }
    }

    /// Dimensionless (pure number, ratio).
    pub const DIMENSIONLESS: Self = Self::si(0, 0, 0, 0, 0, 0);
    /// Mass \[kg\].
    pub const MASS: Self = Self::si(1, 0, 0, 0, 0, 0);
    /// Length \[m\].
    pub const LENGTH: Self = Self::si(0, 1, 0, 0, 0, 0);
    /// Time \[s\].
    pub const TIME: Self = Self::si(0, 0, 1, 0, 0, 0);
    /// Electric current \[A\].
    pub const CURRENT: Self = Self::si(0, 0, 0, 1, 0, 0);
    /// Thermodynamic temperature \[K\].
    pub const TEMPERATURE: Self = Self::si(0, 0, 0, 0, 1, 0);
    /// Plane angle \[rad\].
    pub const ANGLE: Self = Self::si(0, 0, 0, 0, 0, 1);
    /// Amount of substance \[mol\].
    pub const AMOUNT: Self = Self::from_base(BaseDimension::Amount);
    /// Luminous intensity \[cd\].
    pub const LUMINOUS_INTENSITY: Self = Self::from_base(BaseDimension::Luminous);
    /// Area \[m^2\].
    pub const AREA: Self = Self::si(0, 2, 0, 0, 0, 0);
    /// Volume \[m^3\].
    pub const VOLUME: Self = Self::si(0, 3, 0, 0, 0, 0);
    /// Velocity \[m/s\].
    pub const VELOCITY: Self = Self::si(0, 1, -1, 0, 0, 0);
    /// Acceleration \[m/s^2\].
    pub const ACCELERATION: Self = Self::si(0, 1, -2, 0, 0, 0);
    /// Frequency \[Hz = 1/s\]. Distinct from [`Self::ANGULAR_VELOCITY`].
    pub const FREQUENCY: Self = Self::si(0, 0, -1, 0, 0, 0);
    /// Angular velocity \[rad/s\].
    pub const ANGULAR_VELOCITY: Self = Self::si(0, 0, -1, 0, 0, 1);
    /// Angular acceleration \[rad/s^2\].
    pub const ANGULAR_ACCELERATION: Self = Self::si(0, 0, -2, 0, 0, 1);
    /// Force \[N\].
    pub const FORCE: Self = Self::si(1, 1, -2, 0, 0, 0);
    /// Torque \[N*m\]. Dimensionally identical to [`Self::ENERGY`].
    pub const TORQUE: Self = Self::si(1, 2, -2, 0, 0, 0);
    /// Energy \[J = W*s\]. Dimensionally identical to [`Self::TORQUE`].
    pub const ENERGY: Self = Self::TORQUE;
    /// Power \[W\].
    pub const POWER: Self = Self::si(1, 2, -3, 0, 0, 0);
    /// Electric potential \[V\].
    pub const VOLTAGE: Self = Self::si(1, 2, -3, -1, 0, 0);
    /// Electric charge \[A*s\] (battery capacity).
    pub const CHARGE: Self = Self::si(0, 0, 1, 1, 0, 0);
    /// Electric resistance \[ohm\].
    pub const RESISTANCE: Self = Self::si(1, 2, -3, -2, 0, 0);
    /// Pressure \[Pa\].
    pub const PRESSURE: Self = Self::si(1, -1, -2, 0, 0, 0);
    /// Moment of inertia \[kg*m^2\].
    pub const MOMENT_OF_INERTIA: Self = Self::si(1, 2, 0, 0, 0, 0);
    /// Density \[kg/m^3\].
    pub const DENSITY: Self = Self::si(1, -3, 0, 0, 0, 0);

    /// The dimension consisting of a single base dimension with exponent 1.
    pub const fn from_base(base: BaseDimension) -> Self {
        let mut half = [0; N];
        half[base as usize] = 2;
        Self { half }
    }

    /// Builds a dimension from doubled exponents in [`BaseDimension::ALL`] order.
    pub const fn from_half_exponents(half: [i8; N]) -> Self {
        Self { half }
    }

    /// Doubled exponents in [`BaseDimension::ALL`] order.
    pub const fn half_exponents(self) -> [i8; N] {
        self.half
    }

    /// Exponent of `base` (may be a half-integer, e.g. `-0.5`).
    pub fn exponent(self, base: BaseDimension) -> f64 {
        f64::from(self.half[base as usize]) / 2.0
    }

    /// `true` if every exponent is zero.
    pub fn is_dimensionless(self) -> bool {
        self.half == [0; N]
    }

    fn zip(self, other: Self, op: fn(i8, i8) -> Option<i8>) -> Option<Self> {
        let mut half = [0; N];
        for (k, h) in half.iter_mut().enumerate() {
            *h = op(self.half[k], other.half[k])?;
        }
        Some(Self { half })
    }

    /// Product of two dimensions (exponents add); `None` on exponent overflow.
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        self.zip(other, i8::checked_add)
    }

    /// Quotient of two dimensions (exponents subtract); `None` on exponent overflow.
    pub fn checked_div(self, other: Self) -> Option<Self> {
        self.zip(other, i8::checked_sub)
    }

    /// Integer power; `None` on exponent overflow.
    pub fn checked_powi(self, n: i32) -> Option<Self> {
        let n = i8::try_from(n).ok()?;
        let mut half = [0; N];
        for (h, &a) in half.iter_mut().zip(&self.half) {
            *h = a.checked_mul(n)?;
        }
        Some(Self { half })
    }

    /// Square root (exponents halve); `None` if an exponent is already a
    /// half-integer, since quarter exponents are not representable.
    pub fn checked_sqrt(self) -> Option<Self> {
        self.zip(self, |a, _| (a % 2 == 0).then_some(a / 2))
    }

    /// Name and preferred unit symbol of common dimensions, e.g. `("force", "N")`.
    pub fn name(self) -> Option<(&'static str, &'static str)> {
        NAMED
            .iter()
            .find(|(d, ..)| *d == self)
            .map(|&(_, n, s)| (n, s))
    }

    /// Human-readable description for messages, e.g. `"length [m]"` for a named
    /// dimension or `"[kg*m^3]"` otherwise.
    pub fn describe(self) -> String {
        match self.name() {
            Some((name, symbol)) => format!("{name} [{symbol}]"),
            None => format!("[{self}]"),
        }
    }
}

/// Named dimensions, used by [`Dimension::name`] and error messages.
const NAMED: &[(Dimension, &str, &str)] = &[
    (Dimension::DIMENSIONLESS, "dimensionless number", "1"),
    (Dimension::MASS, "mass", "kg"),
    (Dimension::LENGTH, "length", "m"),
    (Dimension::TIME, "time", "s"),
    (Dimension::CURRENT, "electric current", "A"),
    (Dimension::TEMPERATURE, "temperature", "K"),
    (Dimension::ANGLE, "angle", "rad"),
    (Dimension::AMOUNT, "amount of substance", "mol"),
    (Dimension::LUMINOUS_INTENSITY, "luminous intensity", "cd"),
    (Dimension::AREA, "area", "m^2"),
    (Dimension::VOLUME, "volume", "m^3"),
    (Dimension::VELOCITY, "velocity", "m/s"),
    (Dimension::ACCELERATION, "acceleration", "m/s^2"),
    (Dimension::FREQUENCY, "frequency", "Hz"),
    (Dimension::ANGULAR_VELOCITY, "angular velocity", "rad/s"),
    (
        Dimension::ANGULAR_ACCELERATION,
        "angular acceleration",
        "rad/s^2",
    ),
    (Dimension::FORCE, "force", "N"),
    (Dimension::TORQUE, "torque or energy", "N*m or J"),
    (Dimension::POWER, "power", "W"),
    (Dimension::VOLTAGE, "voltage", "V"),
    (Dimension::CHARGE, "electric charge", "A*s"),
    (Dimension::RESISTANCE, "resistance", "ohm"),
    (Dimension::PRESSURE, "pressure", "Pa"),
    (Dimension::MOMENT_OF_INERTIA, "moment of inertia", "kg*m^2"),
    (Dimension::DENSITY, "density", "kg/m^3"),
];

/// Formats one factor with a positive doubled exponent `h`.
fn write_factor(f: &mut fmt::Formatter<'_>, symbol: &str, h: i8) -> fmt::Result {
    match (h % 2 == 0, h) {
        (true, 2) => write!(f, "{symbol}"),
        (true, _) => write!(f, "{symbol}^{}", h / 2),
        (false, 1) => write!(f, "sqrt({symbol})"),
        (false, _) => write!(f, "sqrt({symbol}^{h})"),
    }
}

/// Formats in base SI units, in a form [`super::parse_quantity`] accepts:
/// `m/s^2`, `kg*m^2`, `rad/sqrt(s)`, `1/s`, `1` (dimensionless).
impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let terms = || {
            BaseDimension::ALL
                .iter()
                .map(|&b| (b.symbol(), self.half[b as usize]))
        };
        let mut first = true;
        for (symbol, h) in terms().filter(|&(_, h)| h > 0) {
            if !first {
                f.write_str("*")?;
            }
            first = false;
            write_factor(f, symbol, h)?;
        }
        if first {
            f.write_str("1")?;
        }
        for (symbol, h) in terms().filter(|&(_, h)| h < 0) {
            f.write_str("/")?;
            write_factor(f, symbol, -h)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dimension({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_forms() {
        assert_eq!(Dimension::DIMENSIONLESS.to_string(), "1");
        assert_eq!(Dimension::ACCELERATION.to_string(), "m/s^2");
        assert_eq!(Dimension::FREQUENCY.to_string(), "1/s");
        assert_eq!(Dimension::MOMENT_OF_INERTIA.to_string(), "kg*m^2");
        let gyro_noise = Dimension::ANGLE
            .checked_div(Dimension::TIME.checked_sqrt().unwrap())
            .unwrap();
        assert_eq!(gyro_noise.to_string(), "rad/sqrt(s)");
        let h3 = Dimension::from_half_exponents([0, 0, 3, 0, 0, 0, 0, 0]);
        assert_eq!(h3.to_string(), "sqrt(s^3)");
    }

    #[test]
    fn energy_equals_torque_and_is_named() {
        assert_eq!(Dimension::ENERGY, Dimension::TORQUE);
        assert_eq!(Dimension::TORQUE.describe(), "torque or energy [N*m or J]");
    }

    #[test]
    fn arithmetic() {
        let v = Dimension::LENGTH.checked_div(Dimension::TIME).unwrap();
        assert_eq!(v, Dimension::VELOCITY);
        assert_eq!(Dimension::LENGTH.checked_powi(2), Some(Dimension::AREA));
        assert_eq!(Dimension::AREA.checked_sqrt(), Some(Dimension::LENGTH));
        let half = Dimension::TIME.checked_sqrt().unwrap();
        assert_eq!(half.exponent(BaseDimension::Time), 0.5);
        assert_eq!(half.checked_sqrt(), None);
        assert_eq!(Dimension::LENGTH.checked_powi(100), None);
        assert_eq!(Dimension::LENGTH.checked_powi(1000), None);
    }

    #[test]
    fn describe_unnamed() {
        assert_eq!(
            Dimension::VOLUME
                .checked_mul(Dimension::MASS)
                .unwrap()
                .describe(),
            "[kg*m^3]"
        );
    }
}
