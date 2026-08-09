# Changelog

All notable changes to the Achronyme language and CLI are recorded here.

## [0.1.2] - 2026-08-09

### Fixed

- Detached verification now honors `--error-format json` when no explicit
  `--format` override is supplied.
- Unreadable detached proof artifacts no longer produce the misleading text
  claim that the proof itself is cryptographically invalid.

### Repository

- Removed the legacy internal strategy document and dated audit records, along
  with test comments and documentation checks that referenced them.

### Compatibility

- ACHB container format, bytecode version, runtime ABI, proof artifacts, and
  trusted-key format are unchanged from 0.1.1.

## [0.1.1] - 2026-08-08

### Added

- A versioned Tilino Lab end-to-end fixture covering namespace modules, typed
  array captures, interpreter and LLVM JIT parity, structured concurrency,
  explicit capabilities, Groth16 generation, detached verification, tampered
  public inputs, resource exhaustion, and the standalone AOT boundary.

### Fixed

- Module parse errors, compile errors, and warnings now retain the canonical
  imported file, exact span, and matching source text in human, JSON, and
  short diagnostics.
- Prove blocks inside imported functions now begin resolver lookup in the
  function's owning module, so typed array captures resolve and count as reads.
- Circuit statistics distinguish the pre-R1CS-optimization estimate from the
  exact finalized R1CS constraint count used for Groth16 proving.
- `max_tasks` consistently counts explicit and implicit live child tasks, and
  `max_task_scopes` consistently counts all simultaneously live structured
  scopes instead of being described as nesting depth.

### Compatibility

- ACHB container format, bytecode version, runtime ABI, trusted-key format,
  and fail-closed proving policy are unchanged from 0.1.0.

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
- A fail-closed proof setup policy with explicit development opt-in and an
  inspectable effective key source.
- Immutable, R1CS-digest-addressed BN254 Groth16 trusted-key stores packaged
  from ceremony-derived snarkjs zkeys with canonical manifests and transcripts.
- Detached Groth16 verification for BN254 and BLS12-381 proof artifacts.
- Reproducible phase-1 verification, phase-2 contribution, bidirectional
  snarkjs conformance, and bounded ECDSA release-gate harnesses.

### Changed

- `concurrent`, `spawn`, and `await` are reserved language keywords.
- ACHB output uses container format `0x0d` and register bytecode version 2.
- The append-only compiled runtime ABI is version 6 and includes structured
  task control entries.
- File and network operations require explicit grants before execution.
- Implicit local Groth16 and Plonkish parameter generation is denied. Local
  setup now requires `--insecure-dev-setup`; production BN254 Groth16 proving
  selects an exact ceremony-derived key with `--trusted-key-dir`.

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
- Production trusted-key import is supported for BN254 Groth16 only.
  BLS12-381 Groth16 and Plonkish proof generation remain explicitly
  development-only in 0.1.0.
- See [the 0.1.0 migration guide](docs/migration/0.1.0.md) for source, grant,
  artifact, proving-policy, AOT, and WASM changes.

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
