mod ir;
mod parser;

use crate::parse::kvp::FromKVPValue;
use crate::parse::PrettyPrintResult;
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
        out.write_all("\n".as_bytes())?;
        for (idx, instruction) in self.instructions.iter().enumerate() {
            out.write_all(&vec![b' '; indent * 4])?;
            out.write_all(format!("{} - ", idx).as_bytes())?;
            out.write_all(
                match instruction {
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
                        format!("{}({} arguments)", name, arg_count).as_str()
                    }

                    Instruction::Variable { name } => format!("Variable: {}", name).as_str(),
                    Instruction::Number { value } => format!("{}", value),
                }
                .as_bytes(),
            )?;
            out.write_all("\n".as_bytes());
        }
        Ok(())
    }
}
