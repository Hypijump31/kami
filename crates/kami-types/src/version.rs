//! `ToolVersion` parsing and display.

use std::fmt;
use std::str::FromStr;

use crate::error::KamiError;
use crate::tool::ToolVersion;

impl fmt::Display for ToolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for ToolVersion {
    type Err = KamiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(KamiError::invalid_input(
                "version must be in semver format: MAJOR.MINOR.PATCH",
            ));
        }
        let parse = |p: &str| -> Result<u32, KamiError> {
            p.parse::<u32>()
                .map_err(|_| KamiError::invalid_input(format!("invalid version component: {p}")))
        };
        Ok(Self {
            major: parse(parts[0])?,
            minor: parse(parts[1])?,
            patch: parse(parts[2])?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_parse() {
        let v: ToolVersion = "1.2.3".parse().unwrap();
        assert_eq!(v, ToolVersion::new(1, 2, 3));
    }

    #[test]
    fn version_display() {
        let v = ToolVersion::new(0, 1, 0);
        assert_eq!(v.to_string(), "0.1.0");
    }

    #[test]
    fn invalid_version_rejected() {
        assert!("1.2".parse::<ToolVersion>().is_err());
        assert!("a.b.c".parse::<ToolVersion>().is_err());
    }
}
