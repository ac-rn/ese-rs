//! JET database signature structure.

use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// JET database signature.
///
/// This structure contains metadata about the database creation time and origin.
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
#[repr(C, packed)]
pub struct JetSignature {
    /// Random signature value
    pub random: u32,
    /// Database creation time (Windows FILETIME format)
    pub creation_time: u64,
    /// NetBIOS name of the computer that created the database
    pub netbios_name: [u8; 16],
}

impl JetSignature {
    /// Creates a new JET signature from raw bytes.
    pub fn from_bytes(data: &[u8]) -> Option<&Self> {
        if data.len() < std::mem::size_of::<Self>() {
            return None;
        }
        JetSignature::ref_from_prefix(data)
    }

    /// Returns the NetBIOS name as a string (null-terminated).
    pub fn netbios_name_str(&self) -> Option<&str> {
        let end = self
            .netbios_name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.netbios_name.len());
        std::str::from_utf8(&self.netbios_name[..end]).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jet_signature_size() {
        assert_eq!(std::mem::size_of::<JetSignature>(), 28);
    }

    #[test]
    fn test_netbios_name() {
        let mut sig = JetSignature {
            random: 0,
            creation_time: 0,
            netbios_name: [0; 16],
        };
        sig.netbios_name[..4].copy_from_slice(b"TEST");
        assert_eq!(sig.netbios_name_str(), Some("TEST"));
    }
}
