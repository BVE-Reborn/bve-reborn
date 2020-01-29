use chardetng::EncodingDetector;
use std::fs::read;
use std::io::Result;
use std::path::Path;

/// Reads a file, detects the encoding, and converts to utf8
///
/// # Errors
///
/// Returns Err if opening/reading the file fails. All errors come from [`std::fs::read`].
pub fn read_convert_utf8(filename: impl AsRef<Path>) -> Result<String> {
    let bytes = read(filename)?;

    convert_to_utf8(bytes)
}

fn convert_to_utf8(bytes: Vec<u8>) -> Result<String> {
    // Byte order marks are not properly dealt with in chardetng, detect them here, encoding_rs will remove them
    let encoding = if bytes.len() >= 2 && bytes[0..2] == [0xFF, 0xFE] {
        encoding_rs::UTF_16LE
    } else if bytes.len() >= 2 && bytes[0..2] == [0xFE, 0xFF] {
        encoding_rs::UTF_16BE
    } else if bytes.len() >= 3 && bytes[0..3] == [0xEF, 0xBB, 0xBF] {
        encoding_rs::UTF_8
    } else {
        let mut detector = EncodingDetector::new();
        let ascii_only = !detector.feed(&bytes, true);
        if ascii_only {
            return Ok(String::from_utf8(bytes).expect("Only ascii characters detected, but utf8 validation failed"));
        }
        detector.guess(None, true)
    };

    println!("{}", encoding.name());

    let (result, ..) = encoding.decode_with_bom_removal(&bytes);

    Ok(result.to_string())
}

#[cfg(test)]
mod test {
    use super::convert_to_utf8;

    #[test]
    fn bom_removal() {
        assert_eq!(convert_to_utf8(vec![0xFF, 0xFE]).unwrap(), "");
        assert_eq!(convert_to_utf8(vec![0xFE, 0xFF]).unwrap(), "");
        assert_eq!(convert_to_utf8(vec![0xEF, 0xBB, 0xBF]).unwrap(), "");
    }

    #[test]
    fn shift_jis() {
        // I'm sorry if this is not "hello how are you", blame google
        assert_eq!(
            convert_to_utf8(
                b"\x82\xb1\x82\xf1\x82\xc9\x82\xbf\x82\xcd\x81\x41\x8c\xb3\x8b\x43\x82\xc5\x82\xb7\x82\xa9\x81\x48"
                    .to_vec()
            )
            .unwrap(),
            "こんにちは、元気ですか？"
        );
    }
}
