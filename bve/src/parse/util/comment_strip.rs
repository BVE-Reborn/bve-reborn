pub fn strip_comments(input: &str, comment_char: char) -> String {
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

    result
}

#[cfg(test)]
mod test {
    use crate::parse::util::strip_comments;

    #[test]
    fn single_line() {
        assert_eq!(strip_comments("abcdefg;abcdefg", ';'), "abcdefg\n");
        assert_eq!(strip_comments(";abcdefg", ';'), "\n");
    }

    #[test]
    fn single_end_of_line() {
        assert_eq!(strip_comments("abcdefg;", ';'), "abcdefg\n");
    }

    #[test]
    fn double_line() {
        assert_eq!(strip_comments("abcdefg\nabcdefg;abcdefg", ';'), "abcdefg\nabcdefg\n");
        assert_eq!(strip_comments(";abcdefg\n;abcdefg", ';'), "\n\n");
    }
}
