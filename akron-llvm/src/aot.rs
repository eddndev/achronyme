use std::fmt::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use akron::CompiledProgram;

use crate::{lower_program_with_options, LlvmTierOptions, LoweringError};

#[derive(Debug, Clone)]
pub struct AotOptions {
    pub clang: PathBuf,
    pub runtime_archive: PathBuf,
    pub optimization: u8,
    pub linker_gc_sections: bool,
    pub tier2: LlvmTierOptions,
}

impl AotOptions {
    pub fn new(clang: impl Into<PathBuf>, runtime_archive: impl Into<PathBuf>) -> Self {
        Self {
            clang: clang.into(),
            runtime_archive: runtime_archive.into(),
            optimization: 2,
            linker_gc_sections: true,
            tier2: LlvmTierOptions::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClangVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AotArtifact {
    pub executable: PathBuf,
    pub clang_version: ClangVersion,
    pub instruction_count: usize,
    pub native_instruction_count: usize,
}

pub struct AotCompiler {
    options: AotOptions,
}

impl AotCompiler {
    pub fn new(options: AotOptions) -> Self {
        Self { options }
    }

    pub fn build_executable(
        &self,
        program: &CompiledProgram,
        output: impl AsRef<Path>,
    ) -> Result<AotArtifact, AotError> {
        let output = output.as_ref();
        if !self.options.runtime_archive.is_file() {
            return Err(AotError::Configuration(format!(
                "AOT runtime archive not found: {}",
                self.options.runtime_archive.display()
            )));
        }
        let clang_version = clang_version(&self.options.clang)?;
        if clang_version.major != 21 {
            return Err(AotError::Configuration(format!(
                "AOT requires clang 21, found {}.{}.{}",
                clang_version.major, clang_version.minor, clang_version.patch
            )));
        }

        let lowered = lower_program_with_options(program, self.options.tier2)?;
        let ir = executable_ir(program, &lowered.ir, self.options.tier2)?;
        let parent = output.parent().filter(|path| !path.as_os_str().is_empty());
        if let Some(parent) = parent {
            std::fs::create_dir_all(parent)?;
        }
        let directory = tempfile::Builder::new()
            .prefix("akron-aot-")
            .tempdir_in(parent.unwrap_or_else(|| Path::new(".")))?;
        let ir_path = directory.path().join("program.ll");
        let object_path = directory.path().join("program.o");
        std::fs::write(&ir_path, ir)?;

        let compile = Command::new(&self.options.clang)
            .arg(format!("-O{}", self.options.optimization.min(3)))
            .arg("-c")
            .arg("-x")
            .arg("ir")
            .arg(&ir_path)
            .arg("-o")
            .arg(&object_path)
            .output()?;
        require_success("compile LLVM IR", compile)?;

        let linked = link_command(&self.options, &object_path, output).output()?;
        require_success("link native executable", linked)?;
        if !output.is_file() {
            return Err(AotError::Tool(
                "linker reported success without producing an executable".to_string(),
            ));
        }

        Ok(AotArtifact {
            executable: output.to_path_buf(),
            clang_version,
            instruction_count: lowered.instruction_count,
            native_instruction_count: lowered.native_instruction_count,
        })
    }
}

fn link_command(options: &AotOptions, object: &Path, output: &Path) -> Command {
    let mut command = Command::new(&options.clang);
    command.arg(object).arg(&options.runtime_archive);
    #[cfg(target_os = "linux")]
    if options.linker_gc_sections {
        command.arg("-Wl,--gc-sections");
    }
    command.arg("-o").arg(output);
    #[cfg(target_os = "linux")]
    command.args(["-ldl", "-lpthread", "-lm", "-lrt", "-lutil"]);
    command
}

fn executable_ir(
    program: &CompiledProgram,
    lowered_ir: &str,
    options: LlvmTierOptions,
) -> Result<String, AotError> {
    let runtime = options.runtime_requirement();
    let mut image = Vec::new();
    program
        .write_executable(&mut image)
        .map_err(|error| AotError::Program(error.to_string()))?;
    let mut escaped = String::with_capacity(image.len() * 3);
    for byte in &image {
        write!(escaped, "\\{byte:02X}").expect("write to String");
    }

    let mut ir = String::with_capacity(lowered_ir.len() + escaped.len() + 512);
    ir.push_str(lowered_ir);
    writeln!(
        ir,
        "@akron_program_image = private unnamed_addr constant [{} x i8] c\"{}\", align 1",
        image.len(),
        escaped
    )
    .expect("write to String");
    ir.push_str("declare i32 @akron_aot_runtime_main(ptr, i64, ptr, i32, i32, i64)\n\n");
    ir.push_str("define i32 @main() {\nentry:\n");
    writeln!(
        ir,
        "  %status = call i32 @akron_aot_runtime_main(ptr @akron_program_image, i64 {}, ptr @akron_compiled_main, i32 {}, i32 {}, i64 {})",
        image.len(),
        runtime.version,
        runtime.size,
        runtime.capabilities.bits(),
    )
    .expect("write to String");
    ir.push_str("  ret i32 %status\n}\n");
    Ok(ir)
}

fn clang_version(clang: &Path) -> Result<ClangVersion, AotError> {
    let output = Command::new(clang).arg("--version").output()?;
    require_success("query clang version", output.clone())?;
    let text = String::from_utf8_lossy(&output.stdout);
    let version = text
        .split_whitespace()
        .find(|part| {
            part.chars()
                .next()
                .is_some_and(|character| character.is_ascii_digit())
        })
        .ok_or_else(|| AotError::Tool("clang did not report a parseable version".to_string()))?;
    let mut parts = version.split('.');
    let major = parse_version_part(parts.next(), "major")?;
    let minor = parse_version_part(parts.next(), "minor")?;
    let patch_text = parts.next().unwrap_or("0");
    let patch_digits: String = patch_text
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect();
    let patch = patch_digits.parse().unwrap_or(0);
    Ok(ClangVersion {
        major,
        minor,
        patch,
    })
}

fn parse_version_part(value: Option<&str>, name: &str) -> Result<u32, AotError> {
    value
        .and_then(|part| part.parse().ok())
        .ok_or_else(|| AotError::Tool(format!("clang version has no valid {name} component")))
}

fn require_success(action: &str, output: Output) -> Result<(), AotError> {
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(AotError::Tool(format!("failed to {action}: {stderr}")))
}

#[derive(Debug)]
pub enum AotError {
    Configuration(String),
    Program(String),
    Lowering(LoweringError),
    Io(std::io::Error),
    Tool(String),
}

impl fmt::Display for AotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(message) | Self::Program(message) | Self::Tool(message) => {
                formatter.write_str(message)
            }
            Self::Lowering(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "AOT I/O error: {error}"),
        }
    }
}

impl std::error::Error for AotError {}

impl From<LoweringError> for AotError {
    fn from(error: LoweringError) -> Self {
        Self::Lowering(error)
    }
}

impl From<std::io::Error> for AotError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use super::{link_command, AotOptions};

    fn contains_arg(command: &std::process::Command, expected: &str) -> bool {
        command
            .get_args()
            .any(|argument| argument == OsStr::new(expected))
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linker_gc_sections_are_enabled_by_default_and_can_be_disabled() {
        let enabled = AotOptions::new("clang-21", "runtime.a");
        let object = Path::new("program.o");
        let executable = Path::new("program");

        let enabled_link = link_command(&enabled, object, executable);
        assert!(contains_arg(&enabled_link, "-Wl,--gc-sections"));

        let mut disabled = enabled;
        disabled.linker_gc_sections = false;
        let disabled_link = link_command(&disabled, object, executable);
        assert!(!contains_arg(&disabled_link, "-Wl,--gc-sections"));
    }
}
