mod ir;
mod parser;

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
