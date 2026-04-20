//! Utility functions.

/// Converts Windows FILETIME (100-nanosecond intervals since 1601-01-01) to Unix timestamp.
///
/// # Arguments
///
/// * `filetime` - Windows FILETIME value
///
/// # Returns
///
/// Unix timestamp (seconds since 1970-01-01)
pub fn filetime_to_unix(filetime: u64) -> i64 {
    const FILETIME_UNIX_EPOCH_DIFF: u64 = 116444736000000000;
    const FILETIME_TO_SECONDS: u64 = 10000000;

    if filetime < FILETIME_UNIX_EPOCH_DIFF {
        return 0;
    }

    ((filetime - FILETIME_UNIX_EPOCH_DIFF) / FILETIME_TO_SECONDS) as i64
}

/// Decodes a string using the specified codepage.
///
/// # Arguments
///
/// * `data` - Raw byte data to decode
/// * `codepage` - Windows codepage number
///
/// # Returns
///
/// Decoded string, with invalid sequences replaced with replacement character
pub fn decode_string(data: &[u8], codepage: u32) -> Result<String, crate::error::EseError> {
    use encoding_rs::WINDOWS_1252;

    let encoding = match codepage {
        1200 => encoding_rs::UTF_16LE,      // Unicode (UTF-16LE)
        20127 => encoding_rs::WINDOWS_1252, // ASCII (treat as Windows-1252)
        1252 => WINDOWS_1252,               // Western European
        _ => return Err(crate::error::EseError::UnknownCodepage(codepage)),
    };

    let (decoded, _, had_errors) = encoding.decode(data);

    if had_errors {
        // Log a warning but still return the decoded string
        #[cfg(feature = "logging")]
        log::warn!("Decoding errors encountered for codepage {}", codepage);
    }

    Ok(decoded.to_string())
}

/// Decompresses a tagged column payload using the scheme encoded in its
/// first byte, returning `None` if the scheme is unrecognized so the caller
/// can keep the raw bytes.
///
/// Recognized schemes (per libesedb's `libesedb_compression`):
/// - `first & 0xf8 == 0x00 | 0x10` — 7-bit ASCII compressed. The remainder is
///   a bitstream of 7-bit chars packed LE into bytes, one char per 7 bits.
/// - `first & 0xf8 == 0x18` — uncompressed byte-wise; strip the marker byte
///   and return the rest as-is (typically UTF-16LE text).
///
/// Other schemes (LZXPRESS 0x20, LZXPRESS-Huffman 0x28) are not implemented;
/// their data stays raw `Binary` until a real case surfaces.
pub fn decompress_tagged(data: &[u8]) -> Option<Vec<u8>> {
    if data.is_empty() {
        return None;
    }
    match data[0] & 0xf8 {
        0x00 | 0x10 => Some(decompress_7bit_ascii(&data[1..])),
        0x18 => Some(data[1..].to_vec()),
        _ => None,
    }
}

fn decompress_7bit_ascii(packed: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity((packed.len() * 8) / 7 + 1);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for &b in packed {
        acc |= (b as u32) << bits;
        bits += 8;
        while bits >= 7 {
            out.push((acc & 0x7f) as u8);
            acc >>= 7;
            bits -= 7;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filetime_to_unix() {
        // Windows FILETIME for 2000-01-01 00:00:00 UTC
        let filetime = 125911584000000000u64;
        let unix = filetime_to_unix(filetime);
        // Should be around 946684800 (2000-01-01)
        assert_eq!(unix, 946684800);
    }

    #[test]
    fn test_decode_string_ascii() {
        let data = b"Hello, World!";
        let result = decode_string(data, 20127).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_decode_string_utf16le() {
        let data = b"H\x00e\x00l\x00l\x00o\x00";
        let result = decode_string(data, 1200).unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_decode_string_windows1252() {
        let data = b"\xe9"; // é in Windows-1252
        let result = decode_string(data, 1252).unwrap();
        assert_eq!(result, "é");
    }

    #[test]
    fn test_decompress_7bit_ascii_request_os_version() {
        // Real-world sample: a CA RequestAttributes $AttributeValue for
        // RequestOSVersion on a Windows 7 / Server 2008 R2 request.
        let data = [0x15, 0x36, 0x57, 0xcc, 0x75, 0xb3, 0xc1, 0x62, 0x38, 0x38];
        let out = decompress_tagged(&data).expect("recognized marker");
        // First 8 chars must be the version string; trailing chars depend on
        // the full 10-byte payload but '6.1.7601' is the invariant prefix.
        let s = std::str::from_utf8(&out).expect("7-bit ASCII is valid UTF-8");
        assert!(
            s.starts_with("6.1.7601"),
            "decompressed {s:?} should start with 6.1.7601"
        );
    }

    #[test]
    fn test_decompress_tagged_unknown_marker_returns_none() {
        // 0x20 is LZXPRESS — not implemented, caller should keep raw bytes.
        assert!(decompress_tagged(&[0x20, 0x01, 0x02]).is_none());
    }
}
