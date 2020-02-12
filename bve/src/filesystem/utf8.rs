use chardetng::EncodingDetector;
use std::fs::read;
use std::io::Result;
use std::path::Path;

/// Reads a file, detects the encoding, and converts to utf8.
///
/// # Errors
///
/// Returns Err if opening/reading the file fails. All errors come from [`std::fs::read`].
#[bve_derive::span(DEBUG, "Read to UTF-8", filename = ?filename.as_ref())]
pub fn read_convert_utf8(filename: impl AsRef<Path>) -> Result<String> {
    let bytes = read(filename)?;

    Ok(convert_to_utf8(bytes))
}

#[bve_derive::span(TRACE, "UTF-8 Conversion", size = bytes.len())]
fn convert_to_utf8(bytes: Vec<u8>) -> String {
    tracing::trace!("Converting file of {} bytes", bytes.len());

    // Byte order marks are not properly dealt with in chardetng, detect them here, encoding_rs will remove them
    let (encoding, reason) = if bytes.len() >= 2 && bytes[0..2] == [0xFF, 0xFE] {
        (encoding_rs::UTF_16LE, "BOM")
    } else if bytes.len() >= 2 && bytes[0..2] == [0xFE, 0xFF] {
        (encoding_rs::UTF_16BE, "BOM")
    } else if bytes.len() >= 3 && bytes[0..3] == [0xEF, 0xBB, 0xBF] {
        (encoding_rs::UTF_8, "BOM")
    } else {
        let mut detector = EncodingDetector::new();
        let ascii_only = !detector.feed(&bytes, true);
        if ascii_only {
            tracing::debug!("UTF-8 chosen due to All ASCII");
            return String::from_utf8(bytes).expect("Only ascii characters detected, but utf8 validation failed");
        }
        (detector.guess(None, true), "chardetng")
    };

    tracing::debug!("{} chosen due to {}", encoding.name(), reason);
    let (result, ..) = encoding.decode_with_bom_removal(&bytes);

    tracing::trace!("Converted UTF-8 is {} bytes", result.len());

    result.to_string()
}

#[cfg(test)]
mod test {
    use super::convert_to_utf8;

    #[bve_derive::bve_test]
    #[test]
    fn bom_removal() {
        assert_eq!(convert_to_utf8(vec![0xFF, 0xFE]), "");
        assert_eq!(convert_to_utf8(vec![0xFE, 0xFF]), "");
        assert_eq!(convert_to_utf8(vec![0xEF, 0xBB, 0xBF]), "");
    }

    #[bve_derive::bve_test]
    #[test]
    fn shift_jis() {
        // I'm sorry if this is not "hello how are you", blame google
        assert_eq!(
            convert_to_utf8(
                b"\x82\xb1\x82\xf1\x82\xc9\x82\xbf\x82\xcd\x81\x41\x8c\xb3\x8b\x43\x82\xc5\x82\xb7\x82\xa9\x81\x48"
                    .to_vec()
            ),
            "こんにちは、元気ですか？"
        );
    }
}
