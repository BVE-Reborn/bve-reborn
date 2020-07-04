use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;

pub struct FileInput<'a> {
    base_path: &'a str,
    requested_path: &'a str,
}

pub struct FileOutput {
    path: String,
    output: String,
}

static INCLUDE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)\$include\s*\([^\n]*"#).expect("invalid regex"));
static RND_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)\$rnd\s*\(\s*(\d+)\s*;\s*(\d+)\s*\)"#).expect("invalid regex"));
static CHR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)\$chr\s*\(\s*(\d+)\s*\)"#).expect("invalid regex"));
static SUB_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)\$sub\s*\(\s*(\d+)\s*\)(?:\s*=\s*([^\n]*))?"#).expect("invalid regex"));
static IF_SEARCH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)\$(if|else|endif)\s*\([^\n]*"#).expect("invalid regex"));
static IF_PARSE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)\$if\s*\(\s*(\d+)\s*\)"#).expect("invalid regex"));

type SubMap = HashMap<u64, String>;

pub async fn preprocess_route<R: Rng + ?Sized>(content: &str, rng: &mut R) -> String {
    unimplemented!()
}

fn run_includes<R: Rng + ?Sized>(content: &str, rng: &mut R) -> String {
    // Content will likely get much bigger
    let mut output = String::with_capacity(content.len() * 2);
    let mut last_match = 0_usize;
    for mat in INCLUDE_REGEX.find_iter(content) {
        output.push_str(&content[last_match..mat.start()]);
        let include = &content[mat.range()];
        let include = run_rnd(&include, rng);
        let include = run_chr(&include);
        output.push_str(&parse_include(&include));
        last_match = mat.end();
    }
    output.push_str(&content[last_match..]);

    output
}

fn run_sub<R: Rng + ?Sized>(content: &str, rng: &mut R, sub_map: &mut SubMap) -> String {
    // Content likely gets larger
    let mut output = String::with_capacity(content.len() * 2);
    let mut last_match = 0_usize;
    for capture_set in SUB_REGEX.captures_iter(content) {
        let mat = capture_set.get(0).unwrap_or_else(|| unreachable!());
        output.push_str(&content[last_match..mat.start()]);

        let index = capture_set.get(1).expect("regex has 1-2 groups").as_str();
        let assignment = capture_set.get(2).map(|v| v.as_str());

        let index_int: u64 = index.parse().expect("unable to parse number");

        if let Some(assignment) = assignment {
            sub_map.insert(index_int, assignment.to_string());
        } else {
            let value = sub_map.get(&index_int).map_or("", |s| s.as_str());
            let value = run_rnd(value, rng);
            let value = run_chr(&value);
            output.push_str(&value);
        }
        last_match = mat.end();
    }
    output.push_str(&content[last_match..]);

    output
}

fn run_rnd<R: Rng + ?Sized>(content: &str, rng: &mut R) -> String {
    // Content by definition only gets smaller.
    let mut output = String::with_capacity(content.len());
    let mut last_match = 0_usize;
    for capture_set in RND_REGEX.captures_iter(content) {
        let mat = capture_set.get(0).unwrap_or_else(|| unreachable!());
        output.push_str(&content[last_match..mat.start()]);

        let begin = capture_set.get(1).expect("regex has 2 groups").as_str();
        let end = capture_set.get(2).expect("regex has 2 groups").as_str();

        let begin_int: u64 = begin.parse().expect("unable to parse number");
        let end_int: u64 = end.parse().expect("unable to parse number");

        let value = rng.gen_range(begin_int, end_int.saturating_add(1));
        output.push_str(&value.to_string());

        last_match = mat.end();
    }
    output.push_str(&content[last_match..]);

    output
}

fn run_chr(content: &str) -> String {
    // Content gets a bit larger.
    let mut output = String::with_capacity(content.len() + content.len() / 16);
    let mut last_match = 0_usize;
    for capture_set in CHR_REGEX.captures_iter(content) {
        let mat = capture_set.get(0).unwrap_or_else(|| unreachable!());
        output.push_str(&content[last_match..mat.start()]);

        let value = capture_set.get(1).expect("regex has 1 group").as_str();

        output.push_str(&format!("%C{}%", value));

        last_match = mat.end();
    }
    output.push_str(&content[last_match..]);

    output
}

fn run_if<R: Rng + ?Sized>(content: &str, rng: &mut R, sub_map: &mut SubMap) -> String {
    // Content always gets smaller
    let mut output = String::with_capacity(content.len());
    let mut last_match = 0_usize;

    let mut stack_depth = 0_usize;

    let mut if_value = false;
    let mut if_start = 0_usize;

    for capture_set in IF_SEARCH_REGEX.captures_iter(content) {
        let mat = capture_set.get(0).unwrap_or_else(|| unreachable!());
        let command = capture_set.get(1).expect("regex has 1 group");
        match command.as_str().to_lowercase().as_str() {
            "if" => {
                stack_depth += 1;
                if stack_depth != 1 {
                    continue;
                }
                output.push_str(&content[last_match..mat.start()]);
                let statement = &content[mat.range()];
                let statement = run_sub(statement, rng, sub_map);
                let statement = run_rnd(&statement, rng);
                if let Some(parsed) = IF_PARSE_REGEX.captures(&statement) {
                    let group = parsed.get(1).expect("regex has 1 group");
                    let value: i64 = group.as_str().parse().expect("unable to parse if value");
                    if_value = value != 0;
                    if_start = mat.end();
                } else {
                    unimplemented!()
                }
            }
            "else" => {
                if stack_depth != 1 {
                    continue;
                }
                if if_value {
                    let body = &content[if_start..mat.start()];
                    let body = run_if(body, rng, sub_map);
                    output.push_str(&body);
                }
                if_value = !if_value;
                if_start = mat.end();
            }
            "endif" => {
                if stack_depth == 0 {
                    continue;
                }
                stack_depth -= 1;
                if stack_depth != 0 {
                    continue;
                }
                if if_value {
                    let body = &content[if_start..mat.start()];
                    let body = run_if(body, rng, sub_map);
                    output.push_str(&body);
                }
            }
            _ => unreachable!(),
        }
        last_match = mat.end();
    }
    if stack_depth != 0 {
        if if_value {
            let body = &content[last_match..];
            let body = run_if(body, rng, sub_map);
            output.push_str(&body);
        }
    } else {
        output.push_str(&content[last_match..]);
    }

    output
}

fn parse_include(include: &str) -> String {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::SeedableRng;

    fn new_rng() -> impl Rng {
        rand::rngs::StdRng::seed_from_u64(42)
    }

    #[test]
    fn chr() {
        assert_eq!(run_chr("$chr(10)"), "%C10%");
        assert_eq!(run_chr("$chr(13)"), "%C13%");
        assert_eq!(run_chr("$CHR ( 13 )"), "%C13%");
    }

    #[test]
    fn rnd() {
        assert_eq!(run_rnd("$rnd(1; 6)", &mut new_rng()), "4");
        assert_eq!(run_rnd("$RND ( 1 ; 6 )", &mut new_rng()), "4");
        assert_eq!(run_rnd("$rnd(1;1)", &mut new_rng()), "1");
    }

    #[test]
    fn sub() {
        assert_eq!(
            run_sub("$sub(0) = hi\n$sub(0)", &mut new_rng(), &mut SubMap::new()),
            "\nhi"
        );
        assert_eq!(
            run_sub(
                "$sub(0) = hi\n$sub(0) = bye\n$sub(0)",
                &mut new_rng(),
                &mut SubMap::new()
            ),
            "\n\nbye"
        );
    }

    #[test]
    fn i_f() {
        assert_eq!(
            run_if(
                "$if(1)\ntrue\n$else()\nfalse\n$endif()",
                &mut new_rng(),
                &mut SubMap::new()
            ),
            "\ntrue\n"
        );
        assert_eq!(
            run_if(
                "$if(0)\ntrue\n$else()\nfalse\n$endif()",
                &mut new_rng(),
                &mut SubMap::new()
            ),
            "\nfalse\n"
        );
        assert_eq!(
            run_if(
                "$if($rnd(1;1))\ntrue\n$else()\nfalse\n$endif()",
                &mut new_rng(),
                &mut SubMap::new()
            ),
            "\ntrue\n"
        );
        assert_eq!(
            run_if(
                "$if($rnd(0;0))\ntrue\n$else()\nfalse\n$endif()",
                &mut new_rng(),
                &mut SubMap::new()
            ),
            "\nfalse\n"
        );
        assert_eq!(run_if("$if(1)\ntrue\n", &mut new_rng(), &mut SubMap::new()), "\ntrue\n");
        assert_eq!(run_if("$if(0)\nfalse\n", &mut new_rng(), &mut SubMap::new()), "");
        assert_eq!(
            run_if("$if(1)\ntrue\n$else()\nfalse\n", &mut new_rng(), &mut SubMap::new()),
            "\ntrue\n"
        );
        assert_eq!(
            run_if("$if(0)\nfalse\n$else()\ntrue\n", &mut new_rng(), &mut SubMap::new()),
            "\ntrue\n"
        );
    }

    #[test]
    fn nested_if() {
        assert_eq!(
            run_if(
                "$if(1)\n$if(1)\ntrue\n$endif()\n$else()\n$if(1)\nfalse\n$endif()\n$endif()",
                &mut new_rng(),
                &mut SubMap::new()
            ),
            "\n\ntrue\n\n"
        );
        assert_eq!(
            run_if(
                "$if(0)\n$if(1)\ntrue\n$endif()\n$else()\n$if(1)\nfalse\n$endif()\n$endif()",
                &mut new_rng(),
                &mut SubMap::new()
            ),
            "\n\nfalse\n\n"
        );
    }
}
