//! These are really tests of the derive macro, but are better logically here.

#![allow(clippy::shadow_unrelated)] // These are tests

use crate::parse::kvp::traits::FromKVPFile;
use crate::parse::kvp::{parse_kvp_file, KVPGenericWarning, KVPGenericWarningKind, ANIMATED_LIKE};
use crate::parse::Span;
use bve_derive::{FromKVPFile, FromKVPSection};
use indoc::indoc;

#[test]
fn empty_struct() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {}

    let file_lit = indoc!(
        r#"
        
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    assert_eq!(parsed, File::default());
    assert_eq!(warnings, vec![]);
}

#[test]
fn bare_section_value() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        #[kvp(bare)]
        first: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {
        #[kvp(bare)]
        some1: f32,
        #[kvp(bare)]
        some2: f32,
    }

    let file_lit = indoc!(
        r#"
        6.2
        6.7
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    let mut answer = File::default();
    answer.first.some1 = 6.2;
    answer.first.some2 = 6.7;
    assert_eq!(parsed, answer);
    assert_eq!(warnings, vec![]);
}

#[test]
fn bare_section_kvp() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        #[kvp(bare)]
        first: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {
        some: f32,
    }

    let file_lit = indoc!(
        r#"
        some = 6.7
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    let mut answer = File::default();
    answer.first.some = 6.7;
    assert_eq!(parsed, answer);
    assert_eq!(warnings, vec![]);
}

#[test]
fn single_section_value() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        first: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {
        #[kvp(bare)]
        some1: f32,
        #[kvp(bare)]
        some2: f32,
    }

    let file_lit = indoc!(
        r#"
        [first]
        6.2
        6.7
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    let mut answer = File::default();
    answer.first.some1 = 6.2;
    answer.first.some2 = 6.7;
    assert_eq!(parsed, answer);
    assert_eq!(warnings, vec![]);
}

#[test]
fn single_section_kvp() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        first: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {
        some: f32,
    }

    let file_lit = indoc!(
        r#"
        [first]
        some = 6.7
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    let mut answer = File::default();
    answer.first.some = 6.7;
    assert_eq!(parsed, answer);
    assert_eq!(warnings, vec![]);
}

#[test]
fn single_section_mixed() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        first: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {
        kvp1: f32,
        #[kvp(bare)]
        bare1: f32,
        kvp2: f32,
        #[kvp(bare)]
        bare2: f32,
    }

    let file_lit = indoc!(
        r#"
        [first]
        kvp1 = 1.1
        kvp2 = 1.2
        1.3
        1.4
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    let mut answer = File::default();
    answer.first.kvp1 = 1.1;
    answer.first.kvp2 = 1.2;
    answer.first.bare1 = 1.3;
    answer.first.bare2 = 1.4;
    assert_eq!(parsed, answer);
    assert_eq!(warnings, vec![]);

    let file_lit = indoc!(
        r#"
        [first]
        kvp1 = 1.1
        1.3
        1.4
        kvp2 = 1.2
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    let mut answer = File::default();
    answer.first.kvp1 = 1.1;
    answer.first.kvp2 = 1.2;
    answer.first.bare1 = 1.3;
    answer.first.bare2 = 1.4;
    assert_eq!(parsed, answer);
    assert_eq!(warnings, vec![]);
}

#[test]
fn additive_value() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        first: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {
        #[kvp(bare)]
        some1: Vec<f32>,
    }

    let file_lit = indoc!(
        r#"
        [first]
        6.2
        6.7
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    let mut answer = File::default();
    answer.first.some1 = vec![6.2, 6.7];
    assert_eq!(parsed, answer);
    assert_eq!(warnings, vec![]);
}

#[test]
fn additive_kvp() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        first: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {
        kvp1: Vec<f32>,
        kvp2: Vec<f32>,
    }

    let file_lit = indoc!(
        r#"
        [first]
        kvp1 = 6.2
        kvp2 = 6.5
        kvp1 = 6.7
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    let mut answer = File::default();
    answer.first.kvp1 = vec![6.2, 6.7];
    answer.first.kvp2 = vec![6.5];
    assert_eq!(parsed, answer);
    assert_eq!(warnings, vec![]);
}

#[test]
fn alias_kvp() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        #[kvp(bare)]
        first: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {
        #[kvp(alias = "some-other")]
        some: f32,
    }

    let file_lit = indoc!(
        r#"
        some-other = 6.7
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    let mut answer = File::default();
    answer.first.some = 6.7;
    assert_eq!(parsed, answer);
    assert_eq!(warnings, vec![]);
}

#[test]
fn section_alias() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        #[kvp(alias = "second")]
        first: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {
        some: f32,
    }

    let file_lit = indoc!(
        r#"
        [second]
        some = 6.7
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    let mut answer = File::default();
    answer.first.some = 6.7;
    assert_eq!(parsed, answer);
    assert_eq!(warnings, vec![]);
}

#[test]
fn unknown_section() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {}

    let file_lit = indoc!(
        r#"
        unknown
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    assert_eq!(parsed, File::default());
    assert_eq!(
        warnings,
        vec![KVPGenericWarning {
            span: Span::from_line(0),
            kind: KVPGenericWarningKind::UnknownSection {
                name: String::from("<file header>"),
            }
        }]
    );

    let file_lit = indoc!(
        r#"
        [unknown]
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    assert_eq!(parsed, File::default());
    assert_eq!(
        warnings,
        vec![KVPGenericWarning {
            span: Span::from_line(1),
            kind: KVPGenericWarningKind::UnknownSection {
                name: String::from("unknown"),
            }
        }]
    );
}

#[test]
fn unknown_field() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        #[kvp(bare)]
        header: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {}

    let file_lit = indoc!(
        r#"
        unknown
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    assert_eq!(parsed, File::default());
    assert_eq!(
        warnings,
        vec![KVPGenericWarning {
            span: Span::from_line(1),
            kind: KVPGenericWarningKind::UnknownField {
                name: String::from("<bare field 1 greater than 0 field count>"),
            }
        }]
    );

    let file_lit = indoc!(
        r#"
        unknown = hi
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    assert_eq!(parsed, File::default());
    assert_eq!(
        warnings,
        vec![KVPGenericWarning {
            span: Span::from_line(1),
            kind: KVPGenericWarningKind::UnknownField {
                name: String::from("unknown"),
            }
        }]
    );
}

#[test]
fn invalid_value() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        #[kvp(bare)]
        header: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {
        #[kvp(bare)]
        bare: i32,
    }

    let file_lit = indoc!(
        r#"
        62.3
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    assert_eq!(parsed, File::default());
    assert_eq!(
        warnings,
        vec![KVPGenericWarning {
            span: Span::from_line(1),
            kind: KVPGenericWarningKind::InvalidValue {
                value: String::from("62.3"),
            }
        }]
    );
}

#[test]
fn invalid_kvp_value() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {
        #[kvp(bare)]
        header: Section,
    }

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {
        known: i32,
    }

    let file_lit = indoc!(
        r#"
        known = 62.3
    "#
    );

    let kvp = parse_kvp_file(file_lit, ANIMATED_LIKE);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    assert_eq!(parsed, File::default());
    assert_eq!(
        warnings,
        vec![KVPGenericWarning {
            span: Span::from_line(1),
            kind: KVPGenericWarningKind::InvalidValue {
                value: String::from("62.3"),
            }
        }]
    );
}
