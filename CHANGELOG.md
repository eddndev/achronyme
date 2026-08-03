# Changelog

All notable changes to the Achronyme language and CLI are recorded here.

## [0.1.0] - 2026-08-03

### Added

- Lexical `concurrent` scopes, scope-bound `spawn`, explicit `await`,
  recoverable task outcomes, task races, cooperative cancellation, and task
  tree diagnostics.
- Interprocedural effect inference with a strict transitive boundary between
  host execution and `prove` or `circuit` code.
- Deterministic effect and capability manifests generated from one audited
  builtin registry.
- Bounded channels, permit-based server fan-out, timers, and a cooperative
  single-lane task scheduler with rooted suspended state.
- Owned file, TCP listener, and TCP connection resources with deterministic
  lexical cleanup and explicit ownership transfer to child tasks.
- Suspending file operations through a bounded blocking pool and TCP operations
  through one readiness reactor.
- Exact CLI and project grants for file roots and numeric TCP addresses, plus
  configurable task, scope, pending-request, retained-result, resource,
  channel, worker, and queue limits.
- Stable inspect output for effects, requested and granted capabilities,
  grants, artifact versions, and effective runtime limits.
- Interpreter, LLVM JIT, and AOT conformance coverage for task, timer, channel,
  owned file, and owned TCP behavior.
- An explicit WASM support matrix: pure tasks, bounded channels, cooperative
  yield, and virtual output are supported without ambient host access.

### Changed

- `concurrent`, `spawn`, and `await` are reserved language keywords.
- ACHB output uses container format `0x0d` and register bytecode version 2.
- The append-only compiled runtime ABI is version 6 and includes structured
  task control entries.
- File and network operations require explicit grants before execution.

### Compatibility

- Sequential source remains compatible except identifiers that use the three
  new reserved words.
- `read_line`, `read_file`, and `write_file` remain blocking compatibility
  calls and do not acquire an `await` requirement.
- The runtime continues to load Achronyme 0.0.1 ACHB format `0x0c` with
  bytecode version 1.
- The CLI explicitly preserves console, clock, and randomness authority for
  sequential compatibility. File and network authority remains opt-in;
  embedders and WASM start untrusted.
- Timers, files, network, console input, and randomness remain unsupported in
  WASM until an embedder supplies both an adapter and an explicit grant.
- See [the 0.1.0 migration guide](docs/migration/0.1.0.md) for source, grant,
  artifact, AOT, and WASM changes.

## [0.0.1] - 2026-08-02

This release starts plain `MAJOR.MINOR.PATCH` numbering for the pre-1.0 line.
It intentionally follows the historical `0.1.0-beta.22` series as a numbering
reset. The `0.x` API and language remain subject to change between minor
versions.

### Added

- LLVM 21 ORC JIT execution in the default CLI, with explicit interpreter
  selection and visible fallback when LLVM is unavailable.
- Native AOT compilation with packaged `libakron_aot_runtime.a` discovery.
- Reproducible Linux release bundles with licenses and SHA-256 checksums.
- A production gate for Linux x86_64 and aarch64 covering JIT, fallback, AOT,
  resource limits, file I/O, installed-layout execution, and lazy LLVM loading.

### Compatibility

- Linux GNU x86_64 and aarch64 are the native LLVM release-gated targets.
- macOS and Windows retain interpreter fallback and are not certified by the
  Linux native gate.
