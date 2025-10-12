pub mod version_20220406;
pub mod version_20120307;

/// All supported packet versions.
#[derive(Debug, Clone, Copy)]
pub enum SupportedPacketVersion {
    _20220406,
    _20120307,
}

