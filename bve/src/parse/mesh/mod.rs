//! B3D/CSV Static Meshes
//!
//! There are two ways to make a mesh from a file. First is to directly
//! call [`mesh_from_str`]. This is often the easiest as it takes care of
//! parsing, post processing, and execution automatically. The other way is by
//! manually calling the functions in [`instructions`].
//!
//! There is currently no way to stream from disk but these files are so small
//! who cares.

use crate::parse::mesh::instructions::{create_instructions, post_process, InstructionList};
use crate::parse::{FileParser, ParserResult};
pub use errors::*;
use serde::Deserialize;

mod errors;
pub mod instructions;

pub struct ParsedStaticObject(Vec<instructions::Instruction>);

pub struct ParsedStaticObjectB3D;
pub struct ParsedStaticObjectCSV;

impl FileParser for ParsedStaticObjectB3D {
    type Output = ParsedStaticObject;
    type Warnings = MeshWarning;
    type Errors = MeshError;

    fn parse_from(input: &str) -> ParserResult<Self::Output, Self::Warnings, Self::Errors> {
        let InstructionList {
            instructions,
            warnings,
            errors,
        } = post_process(create_instructions(input, FileType::B3D));

        ParserResult {
            output: instructions,
            warnings,
            errors,
        }
    }
}

/// Which type of file to parse as a mesh.
///
/// The differences are only if there is a comma after the instruction name, instructions from both will work as
/// expected.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FileType {
    /// No comma after instruction name
    B3D,
    /// Comma after instruction name
    CSV,
}

/// The glow numbers to use for this mesh. Not sure how exactly this works.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Glow {
    pub attenuation_mode: GlowAttenuationMode,
    pub half_distance: u16,
}

/// The blending mode to use when rendering the mesh
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    Normal,
    Additive,
}

/// No idea what this does, but every mesh has one or the other.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GlowAttenuationMode {
    DivideExponent2,
    DivideExponent4,
}
