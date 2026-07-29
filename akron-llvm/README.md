# Akron LLVM backend

The normal Achronyme CLI includes this backend and selects `auto`: it attempts
LLVM ORC JIT and falls back to the interpreter when LLVM is unavailable. Use
`--engine interpreter` for an explicit runtime rollback, or build the CLI with
`--no-default-features` to exclude the backend. The low-level `akron-llvm`
crate keeps separate `llvm` and `aot` features.

## Linux LLVM contract

The JIT loads LLVM lazily and resolves every required symbol from one shared
library handle. The CLI records no ELF dependency on LLVM, so it can start and
use the interpreter on systems without LLVM. Without `AKRON_LLVM_DYLIB`, the
loader tries the documented LLVM 21 system names. When the variable is set, its
value is the only candidate and must expose the LLVM 21 C API.

An executable emitted by the AOT backend also does not link LLVM. Compilation
requires Clang 21 and the Akron AOT runtime archive, but the generated
executable contains the native program and runtime.

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
