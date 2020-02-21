//! These are really tests of the derive macro, but are better logically here.

use crate::parse::kvp::parse_kvp_file;
use crate::parse::kvp::traits::FromKVPFile;
use bve_derive::{FromKVPFile, FromKVPSection};
use indoc::indoc;

#[test]
fn empty_struct() {
    #[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
    struct File {}

    #[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
    struct Section {}

    let file_lit = indoc!(
        r#"
        
    "#
    );

    let kvp = parse_kvp_file(file_lit);
    let (parsed, warnings) = File::from_kvp_file(&kvp);
    assert_eq!(parsed, File::default());
    assert_eq!(warnings, vec![]);
}
