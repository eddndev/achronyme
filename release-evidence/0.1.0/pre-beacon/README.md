# Achronyme 0.1.0 pre-beacon evidence

This directory records the safe, public checkpoint reached after the
independent phase-2 contribution was verified and before a final drand beacon
round was selected.

The evidence is bound to source commit
`fd07b38e16256e2ed6a8f2b438d340a681c9b0ac`. Large ceremony artifacts are
identified by size and cryptographic hash in `manifest.json`; they are not
stored in Git. This directory contains no witness values, private inputs,
contributor entropy, toxic waste, phase-1 bytes, or proving-key bytes.

`operator-verification.txt` is an ASCII extract of the authoritative
`snarkjs@0.7.6 zkey verify` result. `manifest.json` also records the SHA-256 of
the original result JSON, raw verification log, and measurement output.

This file describes the historical pre-beacon checkpoint recorded by its
original commit. Later commits in the same evidence branch added the exact
`drand-commitment.json`, the verified `drand-beacon.json`, and the completed
public dossier under `../final/`. The checkpoint `manifest.json` intentionally
remains a snapshot of the earlier state instead of rewriting history.
