//! Business-neutral semantic version values.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A compact three-part version value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Version {
    major: u16,
    minor: u16,
    patch: u16,
}

impl Version {
    /// Creates a version from major, minor, and patch components.
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Returns the major component.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the minor component.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }

    /// Returns the patch component.
    #[must_use]
    pub const fn patch(self) -> u16 {
        self.patch
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::Version;

    #[test]
    fn compares_versions() {
        let older = Version::new(0, 1, 0);
        let newer = Version::new(0, 2, 0);

        assert!(older < newer);
        assert_eq!(older.major(), 0);
        assert_eq!(older.minor(), 1);
        assert_eq!(older.patch(), 0);
    }

    #[test]
    fn displays_versions() {
        assert_eq!(Version::new(1, 2, 3).to_string(), "1.2.3");
    }
}
