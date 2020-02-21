mod ir;
mod parser;

use crate::parse::kvp::FromKVPValue;
pub use ir::*;
pub use parser::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ParsedFunctionScript {
    pub instructions: Vec<Instruction>,
}

impl From<Vec<Instruction>> for ParsedFunctionScript {
    fn from(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }
}

impl FromKVPValue for ParsedFunctionScript {
    fn from_kvp_value(value: &str) -> Option<Self> {
        parse_function_script(value).map(|(_, o)| o).ok()
    }
}
