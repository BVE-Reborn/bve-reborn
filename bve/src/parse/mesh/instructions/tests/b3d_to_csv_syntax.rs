use crate::parse::mesh::instructions::b3d_to_csv_syntax;

#[bve_derive::bve_test]
#[test]
fn comma_add() {
    assert_eq!(b3d_to_csv_syntax("myinstruction arg1"), "myinstruction, arg1\n");
}

#[bve_derive::bve_test]
#[test]
fn multiline_comma_add() {
    assert_eq!(
        b3d_to_csv_syntax("myinstruction arg1\nmyother arg2"),
        "myinstruction, arg1\nmyother, arg2\n"
    );
}

#[bve_derive::bve_test]
#[test]
fn spaceless() {
    assert_eq!(b3d_to_csv_syntax("myinstruction"), "myinstruction\n");
    assert_eq!(b3d_to_csv_syntax(""), "\n");
}

#[bve_derive::bve_test]
#[test]
fn multiline_spaceless() {
    assert_eq!(b3d_to_csv_syntax("myinstruction\n\nfk2"), "myinstruction\n\nfk2\n");
    assert_eq!(b3d_to_csv_syntax("\n\n"), "\n\n");
}
