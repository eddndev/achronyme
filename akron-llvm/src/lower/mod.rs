mod blocks;
mod emit;
mod scalar;
mod verify;

use std::fmt;

use akron::CompiledProgram;

pub struct LoweredModule {
    pub ir: String,
    pub entry_symbol: &'static str,
    pub instruction_count: usize,
    pub native_instruction_count: usize,
}

impl fmt::Debug for LoweredModule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoweredModule")
            .field("entry_symbol", &self.entry_symbol)
            .field("instruction_count", &self.instruction_count)
            .field("native_instruction_count", &self.native_instruction_count)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum LoweringError {
    InvalidProgram(String),
    Unsupported { instruction: usize, reason: String },
    Internal(String),
}

impl LoweringError {
    pub(super) fn unsupported(instruction: usize, reason: impl Into<String>) -> Self {
        Self::Unsupported {
            instruction,
            reason: reason.into(),
        }
    }
}

impl fmt::Display for LoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProgram(message) => write!(formatter, "invalid program: {message}"),
            Self::Unsupported {
                instruction,
                reason,
            } => write!(
                formatter,
                "instruction {instruction} is not eligible for LLVM lowering: {reason}"
            ),
            Self::Internal(message) => write!(formatter, "LLVM lowering error: {message}"),
        }
    }
}

impl std::error::Error for LoweringError {}

pub fn lower_program(program: &CompiledProgram) -> Result<LoweredModule, LoweringError> {
    program
        .validate()
        .map_err(|error| LoweringError::InvalidProgram(error.to_string()))?;
    let verification = verify::verify(program)?;
    let ir = emit::emit(program, &verification)?;
    Ok(LoweredModule {
        ir,
        entry_symbol: "akron_compiled_main",
        instruction_count: program.main.chunk.len(),
        native_instruction_count: verification.native_count(),
    })
}

#[cfg(test)]
mod tests;
