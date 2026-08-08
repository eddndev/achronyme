use std::collections::HashSet;
use std::path::Path;

use ir::stats::CircuitStats;
use ir::{IrProgram, SsaVar};
use memory::FieldBackend;

pub(super) fn collect<F: FieldBackend>(
    enabled: bool,
    path: &str,
    program: &IrProgram<F>,
    proven: &HashSet<SsaVar>,
) -> Option<CircuitStats> {
    enabled.then(|| {
        let name = Path::new(path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned());
        CircuitStats::from_program(program, proven, name.as_deref())
    })
}

pub(super) fn print(stats: Option<CircuitStats>, final_r1cs_constraints: Option<usize>) {
    if let Some(stats) = stats {
        let stats = match final_r1cs_constraints {
            Some(constraints) => stats.with_final_r1cs_constraints(constraints),
            None => stats,
        };
        eprintln!("{stats}");
    }
}
