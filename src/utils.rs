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
}
