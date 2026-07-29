# Akron LLVM backend

`akron-llvm` is optional. The interpreter remains the default execution engine.
Enable `aot` to emit a standalone executable with Clang 21, or `llvm` to use
LLVM ORC JIT. The CLI feature named `llvm` enables both paths.

## Linux LLVM contract

Building a binary with the `llvm` feature requires the linker to find the LLVM
21 C API through `-lLLVM-21`. The resulting ELF binary records a dynamic
dependency on LLVM 21, and the operating-system loader must resolve it before
the process starts.

After startup, the JIT opens a shared library and resolves all LLVM symbols from
that handle. Without `AKRON_LLVM_DYLIB`, it tries the documented LLVM 21 system
names. When `AKRON_LLVM_DYLIB` is set, its value is the only runtime library
candidate and must expose LLVM 21. It selects the symbol source; it does not
replace the link-time dependency and cannot make a binary start on a system
where its recorded LLVM dependency is missing.

The default CLI does not link LLVM. An executable emitted by the AOT backend
also does not link LLVM; compilation requires Clang 21 and the Akron AOT runtime
archive, but the generated executable contains the native program and runtime.

## JIT object cache

The persistent cache is enabled by default when a platform cache directory can
be found. Configure it with:

- `AKRON_JIT_CACHE=0` to disable it.
- `AKRON_JIT_CACHE_DIR` to select a private directory.
- `AKRON_JIT_CACHE_MAX_BYTES` to set the total retained size.

The default total is 128 MiB. The implementation enforces an absolute 1 GiB
total and a 64 MiB per-entry limit even when a larger value is configured. It
checks file metadata on the opened descriptor before allocation, bounds the
read if the file changes, validates the cache identity and checksum, and removes
invalid entries. Allocation and permission failures are safe cache misses.

The cache contains native machine code and is a trust boundary. Identity and
checksums detect stale or accidental corruption; they do not protect against a
same-user attacker who can replace cache files. Use a directory writable only
by the account running Akron, or disable the cache in an untrusted environment.
