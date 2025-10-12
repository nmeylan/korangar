pub mod version_20220406;
pub mod version_20120307;

/// All supported packet versions.
#[derive(Debug, Clone, Copy)]
pub enum SupportedPacketVersion {
    _20220406,
    _20120307,
}

impl SupportedPacketVersion {
    /// Get the version string for packet header lookup.
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportedPacketVersion::_20220406 => "20220406",
            SupportedPacketVersion::_20120307 => "20120307",
        }
    }
}
