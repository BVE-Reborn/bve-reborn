mod ir;
mod parser;

use crate::parse::{kvp::FromKVPValue, util, PrettyPrintResult};
pub use ir::*;
pub use parser::*;
use std::io;

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

impl PrettyPrintResult for ParsedFunctionScript {
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        writeln!(out)?;
        for (idx, instruction) in self.instructions.iter().enumerate() {
            util::indent(indent, out)?;
            write!(out, "{} - ", idx)?;
            writeln!(out, "{}", match instruction {
                Instruction::Addition => "Addition",
                Instruction::Subtraction => "Subtraction",
                Instruction::Multiplication => "Multiplication",
                Instruction::Division => "Division",
                Instruction::LogicalOr => "LogicalOr",
                Instruction::LogicalAnd => "LogicalAnd",
                Instruction::LogicalXor => "LogicalXor",
                Instruction::UnaryLogicalNot => "UnaryLogicalNot",
                Instruction::UnaryNegative => "UnaryNegative",
                Instruction::Equal => "Equals",
                Instruction::NotEqual => "NotEquals",
                Instruction::Less => "Less",
                Instruction::Greater => "Greater",
                Instruction::LessEqual => "LessEqual",
                Instruction::GreaterEqual => "GreaterEqual",
                Instruction::FunctionCall { name, arg_count } => {
                    return write!(out, "{}({} arguments)", name, arg_count);
                }

                Instruction::Variable { name } => return write!(out, "Variable: {}", name),
                Instruction::Number { value } => return write!(out, "{}", value),
            },)?;
        }
        Ok(())
    }
}
