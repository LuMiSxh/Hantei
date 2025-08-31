use crate::bytecode::compiler::BytecodeProgram;
use crate::error::BackendError;
use ahash::AHashMap;
use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub struct CompiledPathInterpreter {
    pub priority: i32,
    pub name: String,
    pub ast: crate::ast::Expression,
    pub static_map: AHashMap<String, crate::ast::InputId>,
    pub dynamic_map: AHashMap<String, crate::ast::InputId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompiledPathBytecode {
    pub priority: i32,
    pub name: String,
    pub program: BytecodeProgram,
}

#[derive(Serialize, Deserialize)]
pub struct CompiledRecipe {
    pub interpreter_paths: Option<Vec<CompiledPathInterpreter>>,
    pub bytecode_programs: Option<Vec<CompiledPathBytecode>>,
}

impl CompiledRecipe {
    pub fn new(
        interpreter_paths: Option<Vec<CompiledPathInterpreter>>,
        bytecode_programs: Option<Vec<CompiledPathBytecode>>,
    ) -> Self {
        Self {
            interpreter_paths,
            bytecode_programs,
        }
    }

    /// Saves the compiled recipe to a file using the bincode format.
    pub fn save(&self, path: &str) -> Result<(), BackendError> {
        let bytes = encode_to_vec(self, standard())
            .map_err(|e| BackendError::Generic(format!("Serialization failed: {}", e)))?;
        let mut file = fs::File::create(path).map_err(|e| {
            BackendError::Generic(format!("Could not create file '{}': {}", path, e))
        })?;
        file.write_all(&bytes).map_err(|e| {
            BackendError::Generic(format!("Could not write to file '{}': {}", path, e))
        })?;
        Ok(())
    }

    /// Loads a compiled recipe from a file.
    pub fn from_file(path: &str) -> Result<Self, BackendError> {
        let mut file = fs::File::open(path)
            .map_err(|e| BackendError::Generic(format!("Could not open file '{}': {}", path, e)))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|e| {
            BackendError::Generic(format!("Could not read from file '{}': {}", path, e))
        })?;
        Self::from_bytes(&bytes)
    }

    /// Deserializes a compiled recipe from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BackendError> {
        decode_from_slice(bytes, standard())
            .map(|(recipe, _)| recipe) // bincode 2 returns a tuple (data, bytes_read)
            .map_err(|e| BackendError::Generic(format!("Deserialization failed: {}", e)))
    }
}
