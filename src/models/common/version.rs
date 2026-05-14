use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub is_prerelease: bool,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32, is_prerelease: bool) -> Self {
        Self {
            major,
            minor,
            patch,
            is_prerelease,
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            bail!("Cannot parse empty version");
        }

        let s = s
            .strip_prefix('v')
            .or_else(|| s.strip_prefix('V'))
            .unwrap_or(s);

        let parts: Vec<&str> = s.split('.').collect();
        if parts.is_empty() || parts.len() > 3 {
            bail!("Invalid version format");
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid major"))?;
        let minor = if parts.len() > 1 {
            parts[1]
                .parse::<u32>()
                .map_err(|_| anyhow::anyhow!("Invalid minor"))?
        } else {
            0
        };
        let patch = if parts.len() > 2 {
            parts[2]
                .parse::<u32>()
                .map_err(|_| anyhow::anyhow!("Invalid patch"))?
        } else {
            0
        };

        Ok(Version::new(major, minor, patch, false))
    }

    pub fn cmp(&self, other: &Version) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match (self.is_prerelease, other.is_prerelease) {
            (false, true) => std::cmp::Ordering::Greater,
            (true, false) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_prerelease {
            write!(f, "{}.{}.{}-pre", self.major, self.minor, self.patch)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Version::cmp(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_prefix_and_missing_parts() {
        let v1 = Version::parse("v1").unwrap();
        assert_eq!(v1.major, 1);
        assert_eq!(v1.minor, 0);
        assert_eq!(v1.patch, 0);

        let v2 = Version::parse("V2.3").unwrap();
        assert_eq!(v2.major, 2);
        assert_eq!(v2.minor, 3);
        assert_eq!(v2.patch, 0);

        let v3 = Version::parse("3.4.5").unwrap();
        assert_eq!(v3.major, 3);
        assert_eq!(v3.minor, 4);
        assert_eq!(v3.patch, 5);
    }

    #[test]
    fn parse_rejects_invalid_inputs() {
        for input in ["", "1.2.3.4", "x.2.3", "1.x.3", "1.2.x"] {
            assert!(
                Version::parse(input).is_err(),
                "expected parse failure for '{input}'"
            );
        }
    }

    #[test]
    fn comparison_handles_prerelease_and_numeric_ordering() {
        let prerelease = Version::new(1, 2, 3, true);
        let stable = Version::new(1, 2, 3, false);
        let newer_patch = Version::new(1, 2, 4, false);

        assert!(stable > prerelease);
        assert!(newer_patch > stable);
    }
}
