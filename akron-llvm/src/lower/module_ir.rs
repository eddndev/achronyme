use std::fmt::Write;

pub(super) fn module_header(output: &mut String) {
    writeln!(output, "; Generated from canonical Akron bytecode.").unwrap();
    writeln!(
        output,
        "%RuntimeApi = type {{ [8 x i8], i32, i32, i64, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr }}"
    )
    .unwrap();
    writeln!(
        output,
        "declare {{ i64, i1 }} @llvm.sadd.with.overflow.i64(i64, i64)"
    )
    .unwrap();
    writeln!(
        output,
        "declare {{ i64, i1 }} @llvm.ssub.with.overflow.i64(i64, i64)"
    )
    .unwrap();
    writeln!(
        output,
        "declare {{ i64, i1 }} @llvm.smul.with.overflow.i64(i64, i64)\n"
    )
    .unwrap();
}
