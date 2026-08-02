# Changelog

All notable changes to the Achronyme language and CLI are recorded here.

## [0.0.1] - Unreleased

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
