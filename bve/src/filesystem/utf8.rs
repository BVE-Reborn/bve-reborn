use chardetng::EncodingDetector;
use std::fs::read;
use std::io::Result;
use std::path::Path;

/// Reads a file, detects the encoding, and converts to utf8
pub fn read_convert_utf8(filename: impl AsRef<Path>) -> Result<String> {
    let bytes = read(filename)?;

    let mut detector = EncodingDetector::new();
    let ascii_only = !detector.feed(&bytes, true);

    if ascii_only {
        Ok(String::from_utf8(bytes).expect("Only ascii characters detected, but utf8 validation failed"))
    } else {
        let encoding = detector.guess(None, true);

        let (result, _encoding, _malformed) = encoding.decode(&bytes);

        Ok(result.to_string())
    }
}
