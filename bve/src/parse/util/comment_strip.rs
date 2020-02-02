pub fn strip_comments(input: &str, comment_char: char) -> String {
    tracing::trace!(%comment_char, input_size = input.len(), "Stripping comments.");

    let mut result = String::new();

    for line in input.lines() {
        let processed = if let Some(idx) = line.find(comment_char) {
            &line[0..idx]
        } else {
            line
        };
        result.push_str(processed);
        result.push('\n');
    }

    tracing::trace!(%comment_char, output_size = result.len(), "Stripped comments");

    result
}

#[cfg(test)]
mod test {
    use crate::parse::util::strip_comments;

    #[bve_derive::bve_test]
    #[test]
    fn single_line() {
        assert_eq!(strip_comments("abcdefg;abcdefg", ';'), "abcdefg\n");
        assert_eq!(strip_comments(";abcdefg", ';'), "\n");
    }

    #[bve_derive::bve_test]
    #[test]
    fn single_end_of_line() {
        assert_eq!(strip_comments("abcdefg;", ';'), "abcdefg\n");
    }

    #[bve_derive::bve_test]
    #[test]
    fn double_line() {
        assert_eq!(strip_comments("abcdefg\nabcdefg;abcdefg", ';'), "abcdefg\nabcdefg\n");
        assert_eq!(strip_comments(";abcdefg\n;abcdefg", ';'), "\n\n");
    }
}
