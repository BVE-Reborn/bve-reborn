use bve_common::nom::w;
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, tag_no_case},
    combinator::map_res,
    multi::separated_list,
    sequence::{delimited, separated_pair, tuple},
    IResult,
};
use once_cell::sync::Lazy;
use rand::{distributions::WeightedIndex, prelude::*};
use regex::Regex;
use smallvec::SmallVec;
use std::{collections::HashMap, future::Future, pin::Pin};

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

pub struct FileInput {
    pub base_path: String,
    pub requested_path: String,
}

pub struct FileOutput {
    pub path: String,
    pub output: String,
}

// (pass -> pass2) means pass2 is applied to the result of pass
// (pass2 <- pass) means pass2 is applied to the skipped input between the last tag and the current
// pass(pass2) means pass2 is applied to pass's arguments
//
// Preprocessing happens as:
// (include(rnd -> chr) -> include) -> (sub <- if(sub -> rnd)) -> rnd -> chr
pub async fn preprocess_route<R, FileFn, FileFut>(
    file_path: &str,
    content: &str,
    rng: &mut R,
    file_fn: FileFn,
) -> String
where
    R: Rng + ?Sized,
    FileFn: FnMut(FileInput) -> FileFut + Copy,
    FileFut: Future<Output = Option<FileOutput>>,
{
    let content = run_includes(file_path, content, rng, file_fn).await;
    let content = run_if(&content, rng, &mut SubMap::new());
    let content = run_rnd(&content, rng);
    run_chr(&content)
}

fn run_includes<'a, R, FileFn, FileFut>(
    file_path: &'a str,
    content: &'a str,
    rng: &'a mut R,
    mut file_fn: FileFn,
) -> Pin<Box<dyn Future<Output = String> + 'a>>
where
    R: Rng + ?Sized,
    FileFn: FnMut(FileInput) -> FileFut + Copy + 'a,
    FileFut: Future<Output = Option<FileOutput>>,
{
    Box::pin(async move {
        // Content will likely get much bigger
        let mut output = String::with_capacity(content.len() * 2);
        let mut last_match = 0_usize;
        for mat in INCLUDE_REGEX.find_iter(content) {
            output.push_str(&content[last_match..mat.start()]);
            let include = &content[mat.range()];
            let include = run_rnd(include, rng);
            let include = run_chr(&include);
            let chosen_opt: Option<(Include<'_>, FileOutput)> = try {
                let parsed = parse_include(&include)?;
                let chosen = choose_include(&parsed, rng)?;
                (
                    chosen,
                    file_fn(FileInput {
                        base_path: file_path.to_owned(),
                        requested_path: chosen.file.to_owned(),
                    })
                    .await?,
                )
            };
            let (chosen, content) = match chosen_opt {
                Some(c) => c,
                None => {
                    last_match = mat.end();
                    continue;
                }
            };

            let recursive_processed = run_includes(&content.path, &content.output, rng, file_fn).await;

            output.push_str(&format!("\n%O{}%\n", chosen.offset));
            output.push_str(&recursive_processed);
            output.push_str(&format!("\n%O-{}%\n", chosen.offset));
            last_match = mat.end();
        }
        output.push_str(&content[last_match..]);

        output
    })
}

fn run_sub<R: Rng + ?Sized>(content: &str, rng: &mut R, sub_map: &mut SubMap) -> String {
    // Content likely gets larger
    let mut output = String::with_capacity(content.len() * 2);
    let mut last_match = 0_usize;
    for capture_set in SUB_REGEX.captures_iter(content) {
        let mat = capture_set.get(0).unwrap_or_else(|| unreachable!());
        output.push_str(&content[last_match..mat.start()]);

        let index_int_opt: Option<u64> = try {
            let index = capture_set.get(1)?.as_str();
            index.parse().ok()?
        };
        let index_int = match index_int_opt {
            Some(v) => v,
            None => {
                last_match = mat.end();
                continue;
            }
        };

        let assignment = capture_set.get(2).map(|v| v.as_str());

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

        let ints_opt = try {
            let begin = capture_set.get(1)?.as_str();
            let end = capture_set.get(2)?.as_str();

            let begin_int: u64 = begin.parse().ok()?;
            let end_int: u64 = end.parse().ok()?;
            (begin_int, end_int)
        };
        let (begin_int, end_int): (u64, u64) = match ints_opt {
            Some(v) => v,
            None => {
                last_match = mat.end();
                continue;
            }
        };

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

        let value_opt = try { capture_set.get(1)?.as_str() };
        let value: &str = match value_opt {
            Some(v) => v,
            None => {
                last_match = mat.end();
                continue;
            }
        };

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
                let previous = &content[last_match..mat.start()];
                let previous = run_sub(previous, rng, sub_map);
                output.push_str(&previous);

                let statement = &content[mat.range()];
                let statement = run_sub(statement, rng, sub_map);
                let statement = run_rnd(&statement, rng);
                let bool_value = if let Some(parsed) = IF_PARSE_REGEX.captures(&statement) {
                    let bool_value_opt: Option<bool> = try {
                        let group = parsed.get(1)?;
                        let value: i64 = group.as_str().parse().ok()?;
                        value != 0
                    };

                    bool_value_opt.unwrap_or(false)
                } else {
                    false
                };
                if_value = bool_value;
                if_start = mat.end();
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
    if stack_depth == 0 {
        let remaining = &content[last_match..];
        let remaining = run_sub(remaining, rng, sub_map);
        output.push_str(&remaining);
    } else if if_value {
        let remaining = &content[last_match..];
        let remaining = run_if(remaining, rng, sub_map);
        output.push_str(&remaining);
    }

    output
}

type IncludeSmallVec<'a> = SmallVec<[Include<'a>; 4]>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Include<'a> {
    file: &'a str,
    offset: i64,
    weight: i64,
}

fn choose_include<'a, R: Rng + ?Sized>(includes: &IncludeSmallVec<'a>, rng: &mut R) -> Option<Include<'a>> {
    if includes.len() == 1 {
        return Some(includes[0]);
    }
    let index = WeightedIndex::new(includes.iter().map(|inc| inc.weight)).ok()?;
    Some(includes[index.sample(rng)])
}

fn parse_include(include: &str) -> Option<IncludeSmallVec<'_>> {
    delimited(
        tuple((w(tag_no_case("$include")), w(tag("(")))),
        alt((parse_weighted_include, parse_offset_include, parse_naked_include)),
        w(tag(")")),
    )(include)
    .map(|(_, v)| v)
    .ok()
}

fn parse_naked_include(include: &str) -> IResult<&str, IncludeSmallVec<'_>> {
    parse_filename(include).map(|(input, file)| {
        (input, smallvec::smallvec![Include {
            file,
            offset: 0,
            weight: 0
        }])
    })
}

fn parse_offset_include(include: &str) -> IResult<&str, IncludeSmallVec<'_>> {
    separated_pair(w(parse_filename), w(tag(":")), w(parse_number))(include).map(|(input, (file, offset))| {
        (input, smallvec::smallvec![Include {
            file,
            offset,
            weight: 0
        }])
    })
}

fn parse_weighted_include(include: &str) -> IResult<&str, IncludeSmallVec<'_>> {
    map_res(
        separated_list(
            w(tag(";")),
            separated_pair(w(parse_filename), w(tag(";")), w(parse_number)),
        ),
        |v| {
            if v.is_empty() {
                return Err(());
            }
            Ok(v.into_iter()
                .map(|(file, weight)| Include {
                    file,
                    weight,
                    offset: 0,
                })
                .collect())
        },
    )(include)
}

fn parse_filename(include: &str) -> IResult<&str, &str> {
    is_not(";:()\n")(include).map(|(i, v)| (i, v.trim()))
}

fn parse_number(include: &str) -> IResult<&str, i64> {
    map_res(is_a("0123456789-"), str::parse)(include)
}

#[cfg(test)]
mod test {
    use super::*;
    use itertools::assert_equal;
    use rand::SeedableRng;

    fn new_rng() -> impl Rng {
        rand::rngs::StdRng::seed_from_u64(42)
    }

    type NewFileFnFut = impl Future<Output = Option<FileOutput>>;
    fn new_file_fn(file_database: HashMap<String, String>) -> impl Fn(FileInput) -> NewFileFnFut {
        move |file_input| {
            let requested = file_input.requested_path;
            let output_opt = file_database.get(&requested).map(String::clone);
            async move {
                Some(FileOutput {
                    path: requested,
                    output: output_opt?,
                })
            }
        }
    }

    static PREPROCESSING_VALIDATION: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"(?i)(include|if|else|endif|sub|rnd|chr)"#).expect("invalid regex"));

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
            run_sub("$sub ( 0 ) = hi\n$sub ( 0 )", &mut new_rng(), &mut SubMap::new()),
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

    #[test]
    #[allow(clippy::shadow_unrelated)]
    fn if_sub_integration() {
        let input_positive: &str = indoc::indoc!(
            r"
            $sub(0) = 1
            $if($sub(0))
                true
            $else()
                false
            $endif()
        "
        );
        let input_negative: &str = indoc::indoc!(
            r"
            $sub(0) = 0
            $if($sub(0))
                true
            $else()
                false
            $endif()
        "
        );
        let processed = run_if(input_positive, &mut new_rng(), &mut SubMap::new());
        assert!(processed.contains("true"), "output missing true: {}", processed);
        assert!(!processed.contains("false"), "output contains false: {}", processed);
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );

        let processed = run_if(input_negative, &mut new_rng(), &mut SubMap::new());
        assert!(processed.contains("false"), "output missing false: {}", processed);
        assert!(!processed.contains("true"), "output contains true: {}", processed);
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );

        let input_positive: &str = indoc::indoc!(
            r"
            $if(1)
                $sub(0) = true
            $else()
                $sub(0) = false
            $endif()
            $sub(0)
        "
        );
        let input_negative: &str = indoc::indoc!(
            r"
            $if(0)
                $sub(0) = true
            $else()
                $sub(0) = false
            $endif()
            $sub(0)
        "
        );
        let processed = run_if(input_positive, &mut new_rng(), &mut SubMap::new());
        assert!(processed.contains("true"), "output missing true: {}", processed);
        assert!(!processed.contains("false"), "output contains false: {}", processed);
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );

        let processed = run_if(input_negative, &mut new_rng(), &mut SubMap::new());
        assert!(processed.contains("false"), "output missing false: {}", processed);
        assert!(!processed.contains("true"), "output contains true: {}", processed);
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );
    }

    #[test]
    fn if_sub_rnd_integration() {
        let input_positive: &str = indoc::indoc!(
            r"
            $sub(1) = $rnd(1;4)
            $if($sub(1))
                $sub(0) = true
            $else()
                $sub(0) = false
            $endif()
            $sub(0)
        "
        );
        let input_negative: &str = indoc::indoc!(
            r"
            $sub(1) = $rnd(0;0)
            $if($sub(1))
                $sub(0) = true
            $else()
                $sub(0) = false
            $endif()
            $sub(0)
        "
        );
        let processed = run_if(input_positive, &mut new_rng(), &mut SubMap::new());
        assert!(processed.contains("true"), "output missing true: {}", processed);
        assert!(!processed.contains("false"), "output contains false: {}", processed);
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );

        let processed = run_if(input_negative, &mut new_rng(), &mut SubMap::new());
        assert!(processed.contains("false"), "output missing false: {}", processed);
        assert!(!processed.contains("true"), "output contains true: {}", processed);
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );
    }

    #[test]
    fn include_parse() {
        assert_equal(
            parse_include(r#"$include(Thing\Other/Thing with Space)"#).expect("parse failed"),
            std::iter::once(Include {
                file: r#"Thing\Other/Thing with Space"#,
                offset: 0,
                weight: 0,
            }),
        );
        assert_equal(
            parse_include(r#"$include(Thing\Other/Thing with Space:1000)"#).expect("parse failed"),
            std::iter::once(Include {
                file: r#"Thing\Other/Thing with Space"#,
                offset: 1000,
                weight: 0,
            }),
        );
        assert_equal(
            parse_include(r#"$include(Thing\Other/Thing with Space:-1000)"#).expect("parse failed"),
            std::iter::once(Include {
                file: r#"Thing\Other/Thing with Space"#,
                offset: -1000,
                weight: 0,
            }),
        );
        assert_equal(
            parse_include(r#"$include(Thing\Other/Thing with Space   :  1000)"#).expect("parse failed"),
            std::iter::once(Include {
                file: r#"Thing\Other/Thing with Space"#,
                offset: 1000,
                weight: 0,
            }),
        );
        assert_equal(
            parse_include(r#"$include(Thing\Other/Thing with Space;12)"#).expect("parse failed"),
            std::iter::once(Include {
                file: r#"Thing\Other/Thing with Space"#,
                offset: 0,
                weight: 12,
            }),
        );
        assert_equal::<_, IncludeSmallVec<'_>>(
            parse_include(r#"$include(Thing\Other/Thing with Space;12;OtherThing;76)"#).expect("parse failed"),
            smallvec::smallvec![
                Include {
                    file: r#"Thing\Other/Thing with Space"#,
                    offset: 0,
                    weight: 12,
                },
                Include {
                    file: r#"OtherThing"#,
                    offset: 0,
                    weight: 76,
                }
            ],
        );
    }

    #[async_std::test]
    async fn include() {
        let file_database = maplit::hashmap! {
            String::from("file1") => String::from("contents1"),
        };

        let file_fn = new_file_fn(file_database);
        let mut rng = new_rng();

        let input: &str = indoc::indoc!(
            r"
            $include(file1)
        "
        );

        let processed: String = run_includes("", input, &mut rng, &file_fn).await;
        assert!(
            processed.contains("contents1"),
            "output missing contents: {}",
            processed
        );
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );
    }

    #[async_std::test]
    async fn offset_include() {
        let file_database = maplit::hashmap! {
            String::from("file1") => String::from("contents1"),
        };

        let file_fn = new_file_fn(file_database);
        let mut rng = new_rng();

        let input: &str = indoc::indoc!(
            r"
            $include(file1:1000)
        "
        );

        let processed: String = run_includes("", input, &mut rng, &file_fn).await;
        assert!(
            processed.contains("contents1"),
            "output missing contents: {}",
            processed
        );
        assert!(processed.contains("%O1000%"), "output missing offset: {}", processed);
        assert!(
            processed.contains("%O-1000%"),
            "output missing reverse offset: {}",
            processed
        );
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );
    }

    #[async_std::test]
    async fn rng_include() {
        let file_database = maplit::hashmap! {
            String::from("file1") => String::from("contents1"),
            String::from("file2") => String::from("contents2"),
        };

        let file_fn = new_file_fn(file_database);
        let mut rng = new_rng();

        let positive_input: &str = indoc::indoc!(
            r"
            $include(file1;1;file2;0)
        "
        );
        let negative_input: &str = indoc::indoc!(
            r"
            $include(file1;0;file2;1)
        "
        );

        let processed: String = run_includes("", positive_input, &mut rng, &file_fn).await;
        assert!(
            processed.contains("contents1"),
            "output missing contents: {}",
            processed
        );
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );

        let processed: String = run_includes("", negative_input, &mut rng, &file_fn).await;
        assert!(
            processed.contains("contents2"),
            "output missing contents: {}",
            processed
        );
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );
    }

    #[async_std::test]
    async fn recursive_include() {
        let file_database = maplit::hashmap! {
            String::from("file1") => String::from("$include(file2)\ncontents1"),
            String::from("file2") => String::from("contents2"),
        };

        let file_fn = new_file_fn(file_database);
        let mut rng = new_rng();

        let input: &str = indoc::indoc!(
            r"
            $include(file1)
        "
        );

        let processed: String = run_includes("", input, &mut rng, &file_fn).await;
        assert!(
            processed.contains("contents1"),
            "output missing contents: {}",
            processed
        );
        assert!(
            processed.contains("contents2"),
            "output missing contents: {}",
            processed
        );
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );
    }

    #[async_std::test]
    async fn include_sub_integration() {
        let file_database = maplit::hashmap! {
            String::from("file1") => String::from("$sub(0) = contents1"),
        };

        let file_fn = new_file_fn(file_database);
        let mut rng = new_rng();

        let input: &str = indoc::indoc!(
            r"
            $include(file1)
            $sub(0)
        "
        );

        let processed: String = preprocess_route("", input, &mut rng, &file_fn).await;
        assert!(
            processed.contains("contents1"),
            "output missing contents: {}",
            processed
        );
        assert!(
            !PREPROCESSING_VALIDATION.is_match(&processed),
            "contains preprocessing directives: {}",
            processed
        );
    }
}
