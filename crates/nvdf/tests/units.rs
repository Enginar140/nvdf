//! Tests for `nvdf::units`.

use nvdf::units::{
    BaseDimension, Dimension as D, Quantity, UnitError, known_units, parse_as, parse_quantity,
};
use std::f64::consts::PI;

const DEG: f64 = PI / 180.0;

#[track_caller]
fn assert_close(actual: f64, expected: f64) {
    let tol = 1e-12 * expected.abs().max(f64::MIN_POSITIVE);
    assert!(
        (actual - expected).abs() <= tol,
        "actual {actual:e}, expected {expected:e}"
    );
}

#[track_caller]
fn check(s: &str, value: f64, dim: D) {
    let q = parse_quantity(s).unwrap_or_else(|e| panic!("{s:?}: {e}"));
    assert_eq!(q.dim, dim, "{s:?}");
    assert_close(q.value, value);
}

#[track_caller]
fn err(s: &str) -> UnitError {
    parse_quantity(s).expect_err(s)
}

#[test]
fn every_unit() {
    let cases: &[(&str, f64, D)] = &[
        ("kg", 1.0, D::MASS),
        ("g", 1e-3, D::MASS),
        ("m", 1.0, D::LENGTH),
        ("cm", 0.01, D::LENGTH),
        ("mm", 0.001, D::LENGTH),
        ("km", 1000.0, D::LENGTH),
        ("ft", 0.3048, D::LENGTH),
        ("in", 0.0254, D::LENGTH),
        ("s", 1.0, D::TIME),
        ("ms", 1e-3, D::TIME),
        ("us", 1e-6, D::TIME),
        ("\u{b5}s", 1e-6, D::TIME),
        ("\u{3bc}s", 1e-6, D::TIME),
        ("min", 60.0, D::TIME),
        ("h", 3600.0, D::TIME),
        ("Hz", 1.0, D::FREQUENCY),
        ("kHz", 1000.0, D::FREQUENCY),
        ("rad", 1.0, D::ANGLE),
        ("deg", DEG, D::ANGLE),
        ("\u{b0}", DEG, D::ANGLE),
        ("rpm", 2.0 * PI / 60.0, D::ANGULAR_VELOCITY),
        ("kn", 1852.0 / 3600.0, D::VELOCITY),
        ("N", 1.0, D::FORCE),
        ("J", 1.0, D::ENERGY),
        ("Wh", 3600.0, D::ENERGY),
        ("W", 1.0, D::POWER),
        ("kW", 1000.0, D::POWER),
        ("Pa", 1.0, D::PRESSURE),
        ("hPa", 100.0, D::PRESSURE),
        ("kPa", 1000.0, D::PRESSURE),
        ("bar", 1e5, D::PRESSURE),
        ("V", 1.0, D::VOLTAGE),
        ("mV", 1e-3, D::VOLTAGE),
        ("A", 1.0, D::CURRENT),
        ("mA", 1e-3, D::CURRENT),
        ("Ah", 3600.0, D::CHARGE),
        ("mAh", 3.6, D::CHARGE),
        ("ohm", 1.0, D::RESISTANCE),
        ("\u{3a9}", 1.0, D::RESISTANCE),
        ("\u{2126}", 1.0, D::RESISTANCE),
        ("mohm", 1e-3, D::RESISTANCE),
        ("m\u{3a9}", 1e-3, D::RESISTANCE),
        ("K", 1.0, D::TEMPERATURE),
        ("mol", 1.0, D::AMOUNT),
        ("cd", 1.0, D::LUMINOUS_INTENSITY),
        ("%", 0.01, D::DIMENSIONLESS),
    ];
    assert_eq!(
        cases.len(),
        known_units().len(),
        "every table entry is tested"
    );
    for &(unit, factor, dim) in cases {
        check(&format!("2.5 {unit}"), 2.5 * factor, dim);
        let def = known_units().iter().find(|d| d.symbol == unit).unwrap();
        assert_close(def.factor, factor);
    }
}

#[test]
fn exact_conversions() {
    assert_eq!(parse_as("1 h", D::TIME).unwrap(), 3600.0);
    assert_eq!(parse_as("1 mAh", D::CHARGE).unwrap(), 3.6);
    assert_eq!(parse_as("1 kn", D::VELOCITY).unwrap(), 1852.0 / 3600.0);
    assert_eq!(parse_as("1 ft", D::LENGTH).unwrap(), 0.3048);
    assert_eq!(parse_as("1 deg", D::ANGLE).unwrap(), PI / 180.0);
    assert_eq!(parse_as("180 deg", D::ANGLE).unwrap(), PI);
}

#[test]
fn compound_expressions() {
    check("9.81 m/s^2", 9.81, D::ACCELERATION);
    check("0.02 kg*m^2", 0.02, D::MOMENT_OF_INERTIA);
    check("1.5 N*m", 1.5, D::TORQUE);
    check("1.225 kg/m^3", 1.225, D::DENSITY);
    check("15 m/s", 15.0, D::VELOCITY);
    check("36 km/h", 10.0, D::VELOCITY);
    check("90 deg/s", 90.0 * DEG, D::ANGULAR_VELOCITY);
    check("10 deg/s^2", 10.0 * DEG, D::ANGULAR_ACCELERATION);
    check("2 kg*m/s^2", 2.0, D::FORCE);
    check("3 W*s", 3.0, D::ENERGY);
    check("1 V*A", 1.0, D::POWER);
    check("1 V/A", 1.0, D::RESISTANCE);
    check("1 kg*m^2/s^2", 1.0, D::ENERGY);
    check("5 1/s", 5.0, D::FREQUENCY);
    check("5 s^-1", 5.0, D::FREQUENCY);
    check("5 m^+2", 5.0, D::AREA);
    check("2 kg/(m*s)", 2.0, D::PRESSURE.checked_mul(D::TIME).unwrap());
    check("4 m^2/m", 4.0, D::LENGTH);
    check("1 m^0", 1.0, D::DIMENSIONLESS);
    check("100 cm^2", 0.01, D::AREA);
    check("1 g/cm^3", 1000.0, D::DENSITY);
    check("3 N * m", 3.0, D::TORQUE);
    check("3 N*m / rad", 3.0, D::TORQUE.checked_div(D::ANGLE).unwrap());
}

#[test]
fn left_to_right_division() {
    // m/s/s == m/s^2, not m/(s/s)
    check("2 m/s/s", 2.0, D::ACCELERATION);
}

#[test]
fn sqrt_densities() {
    let rad_per_sqrt_s = D::ANGLE
        .checked_div(D::TIME.checked_sqrt().unwrap())
        .unwrap();
    assert_eq!(rad_per_sqrt_s.exponent(BaseDimension::Angle), 1.0);
    assert_eq!(rad_per_sqrt_s.exponent(BaseDimension::Time), -0.5);
    check(
        "0.15 deg/sqrt(h)",
        0.15 * DEG / 3600f64.sqrt(),
        rad_per_sqrt_s,
    );

    // m/s/sqrt(Hz) = m * s^-1 * s^(1/2) = m * s^-1/2
    let accel_noise = D::LENGTH
        .checked_div(D::TIME.checked_sqrt().unwrap())
        .unwrap();
    check("0.002 m/s/sqrt(Hz)", 0.002, accel_noise);
    // m/s^2/sqrt(Hz) = m * s^-3/2
    let accel_density = D::from_half_exponents([0, 2, -3, 0, 0, 0, 0, 0]);
    check("1e-3 m/s^2/sqrt(Hz)", 1e-3, accel_density);
    check("0.01 deg/s/sqrt(Hz)", 0.01 * DEG, rad_per_sqrt_s);
    check("2 sqrt(m^2)", 2.0, D::LENGTH);
    check(
        "2 sqrt(kHz)",
        2.0 * 1000f64.sqrt(),
        D::TIME.checked_sqrt().unwrap().checked_powi(-1).unwrap(),
    );
    check("1 sqrt(s)^2", 1.0, D::TIME);

    assert!(matches!(err("1 sqrt(sqrt(s))"), UnitError::Syntax { .. }));
    assert!(matches!(err("1 sqrt Hz"), UnitError::Syntax { .. }));
    assert!(matches!(err("1 sqrt(Hz"), UnitError::Syntax { .. }));
}

#[test]
fn motor_kv() {
    let kv = parse_quantity("700 rpm/V").unwrap();
    assert_eq!(kv.dim, D::ANGULAR_VELOCITY.checked_div(D::VOLTAGE).unwrap());
    assert_close(kv.value, 700.0 * 2.0 * PI / 60.0);
    // rpm is angular velocity, not frequency
    assert!(parse_as("1000 rpm", D::ANGULAR_VELOCITY).is_ok());
    assert!(parse_as("1000 rpm", D::FREQUENCY).is_err());
    assert!(parse_as("50 Hz", D::ANGULAR_VELOCITY).is_err());
}

#[test]
fn percent() {
    assert_close(parse_as("85 %", D::DIMENSIONLESS).unwrap(), 0.85);
    assert_close(parse_as("85%", D::DIMENSIONLESS).unwrap(), 0.85);
    assert_close(parse_as("-2.5%", D::DIMENSIONLESS).unwrap(), -0.025);
    assert!(parse_as("85 %", D::LENGTH).is_err());
}

#[test]
fn celsius_is_affine() {
    assert_close(parse_as("25 degC", D::TEMPERATURE).unwrap(), 298.15);
    assert_close(parse_as("-40 \u{b0}C", D::TEMPERATURE).unwrap(), 233.15);
    assert_close(parse_as(" 0degC ", D::TEMPERATURE).unwrap(), 273.15);
    assert!(parse_as("25 degC", D::LENGTH).is_err());
    for s in ["1 degC/s", "1 W/degC", "1 degC^2", "1 \u{b0}C*s"] {
        assert!(matches!(err(s), UnitError::AffineMisuse { .. }), "{s}");
    }
    assert!(err("1 degC/s").to_string().contains("\"K/s\""));
    check("2 K/s", 2.0, D::TEMPERATURE.checked_div(D::TIME).unwrap());
}

#[test]
fn plain_numbers_are_si() {
    let all = [
        D::MASS,
        D::LENGTH,
        D::TIME,
        D::ANGLE,
        D::VELOCITY,
        D::ACCELERATION,
        D::ANGULAR_VELOCITY,
        D::FORCE,
        D::TORQUE,
        D::POWER,
        D::VOLTAGE,
        D::CURRENT,
        D::CHARGE,
        D::ENERGY,
        D::RESISTANCE,
        D::PRESSURE,
        D::TEMPERATURE,
        D::FREQUENCY,
        D::DIMENSIONLESS,
        D::MOMENT_OF_INERTIA,
        D::DENSITY,
        D::AREA,
    ];
    for dim in all {
        assert_eq!(parse_as("0.5", dim).unwrap(), 0.5);
    }
    check("42", 42.0, D::DIMENSIONLESS);
    for (s, v) in [
        ("-3", -3.0),
        ("+3", 3.0),
        ("3.", 3.0),
        (".5", 0.5),
        ("1e3", 1000.0),
        ("1E+3", 1000.0),
        ("2.5e-3", 2.5e-3),
        ("  7  ", 7.0),
        ("0", 0.0),
    ] {
        assert_eq!(parse_as(s, D::LENGTH).unwrap(), v, "{s:?}");
    }
}

#[test]
fn whitespace_variants() {
    for s in ["4.2kg", "4.2 kg", " 4.2  kg ", "\t4.2\tkg\n", "4.2 kg "] {
        check(s, 4.2, D::MASS);
    }
    check("1e-3m", 1e-3, D::LENGTH);
    check("2 m / s ^ 2", 2.0, D::ACCELERATION);
}

#[test]
fn dimension_mismatch() {
    let e = parse_as("3 kg", D::LENGTH).unwrap_err();
    assert_eq!(
        e,
        UnitError::DimensionMismatch {
            expected: D::LENGTH,
            found: D::MASS
        }
    );
    assert_eq!(e.to_string(), "expected length [m], found mass [kg]");
    assert_eq!(
        parse_as("30 deg", D::LENGTH).unwrap_err().to_string(),
        "expected length [m], found angle [rad]"
    );
    // angle is distinct from dimensionless
    assert!(parse_as("1 rad", D::DIMENSIONLESS).is_err());
    assert!(parse_as("1 rad", D::ANGLE).is_ok());
    assert_eq!(
        parse_as("1 kg*m^3", D::MASS).unwrap_err().to_string(),
        "expected mass [kg], found [kg*m^3]"
    );
    assert_eq!(
        parse_quantity("2 m").unwrap().expect(D::LENGTH).unwrap(),
        2.0
    );
}

#[test]
fn unknown_units() {
    assert_eq!(
        err("4.2 kgg"),
        UnitError::UnknownUnit {
            unit: "kgg".into(),
            suggestion: None
        }
    );
    let e = err("1500 MAH");
    assert_eq!(
        e,
        UnitError::UnknownUnit {
            unit: "MAH".into(),
            suggestion: Some("mAh")
        }
    );
    assert!(e.to_string().contains("did you mean \"mAh\""));
    assert!(matches!(err("1 m/furlong"), UnitError::UnknownUnit { .. }));
    assert!(matches!(err("1 degF"), UnitError::UnknownUnit { .. }));
}

#[test]
fn malformed_input() {
    assert_eq!(err(""), UnitError::Empty);
    assert_eq!(err("   "), UnitError::Empty);
    for s in [
        "kg", "1e", "1e+", "-", ".", "+.e3", "4.2.3 kg", "- 3", "inf", "NaN", "e5",
    ] {
        assert!(
            matches!(err(s), UnitError::BadNumber { .. }),
            "{s:?}: {:?}",
            err(s)
        );
    }
    assert_eq!(
        err("4.2 m/"),
        UnitError::Syntax {
            column: 7,
            message: "expected a unit".into()
        }
    );
    for s in [
        "4.2 m*",
        "4.2 m^",
        "4.2 m^x",
        "4.2 m^2.5",
        "4.2 N m",
        "4.2 m)",
        "4.2 (m",
        "4.2 /s",
        "4.2 1",
        "4.2 1 s",
        "4.2 m^2^2",
        "4.2 m^99",
        "4.2 m^99999999999",
        "4.2 kg,",
        "4.2 m s",
    ] {
        assert!(
            matches!(err(s), UnitError::Syntax { .. }),
            "{s:?}: {:?}",
            err(s)
        );
    }
    assert!(err("4.2 N m").to_string().contains("use '*' or '/'"));
    assert!(matches!(err("1e400 m"), UnitError::OutOfRange { .. }));
    assert!(matches!(err("1e300 km^3"), UnitError::OutOfRange { .. }));
}

#[test]
fn error_messages_are_readable() {
    assert_eq!(
        err("4.2 m/").to_string(),
        "invalid unit at column 7: expected a unit"
    );
    assert!(
        err("kg")
            .to_string()
            .starts_with("invalid number in \"kg\"")
    );
    assert!(err("").to_string().starts_with("empty value"));
    assert_eq!(err("1 xyz").to_string(), "unknown unit \"xyz\"");
}

#[test]
fn display_round_trips() {
    for s in [
        "9.81 m/s^2",
        "700 rpm/V",
        "0.15 deg/sqrt(h)",
        "0.002 m/s/sqrt(Hz)",
        "5 1/s",
        "2 kg*m^2",
        "1 V",
        "1 ohm",
        "3 %",
        "1 mol/cd",
    ] {
        let q = parse_quantity(s).unwrap();
        let back: Quantity = q.to_string().parse().unwrap();
        assert_eq!(back.dim, q.dim, "{s} -> {q}");
        assert_close(back.value, q.value);
    }
    assert_eq!(parse_quantity("1.5 km").unwrap().to_string(), "1500 m");
}
