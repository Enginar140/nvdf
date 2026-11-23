//! # nvdf — VDF (Vehicle Description Format)
//!
//! Reference implementation of VDF, a simulator-independent format for describing
//! vehicles and robots: fixed-wing UAVs, multirotors, underwater vehicles, ground
//! robots and manipulators, together with their aerodynamics, propulsion, power
//! and sensors.
//!
//! The format is at the design stage; see `docs/tasarim/00_vdf_tasarimi.md`.

/// The VDF format version implemented by this crate (`nvdf:` key in documents).
pub const FORMAT_VERSION: &str = "0.1";

/// File extension used by VDF documents (without the leading dot).
pub const FILE_EXTENSION: &str = "nvdf";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_version_is_major_minor() {
        let parts: Vec<&str> = FORMAT_VERSION.split('.').collect();
        assert_eq!(parts.len(), 2, "format version must be MAJOR.MINOR");
        assert!(parts.iter().all(|p| p.parse::<u32>().is_ok()));
    }

    #[test]
    fn extension_has_no_leading_dot() {
        assert!(!FILE_EXTENSION.starts_with('.'));
    }
}
