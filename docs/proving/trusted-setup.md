# Production trusted setup

Achronyme 0.1.0 fails closed when proof generation would need an untrusted
local setup. Production BN254 Groth16 proving uses a ceremony-derived snarkjs
zkey bound to the exact optimized R1CS. Local Arkworks setup remains available
only through an explicit development opt-in.

This document covers the cryptographic trust boundary. The ECDSA release-scale
execution plan is in [the boss-fight gate](boss-fight-release-gate.md).

## Support contract

| Backend | Development proof | Production key import | Detached verification |
| --- | --- | --- | --- |
| BN254 Groth16 | `--insecure-dev-setup` | Supported through a trusted store | `ach verify --curve bn254` |
| BLS12-381 Groth16 | `--insecure-dev-setup` | Not supported in 0.1.0 | `ach verify --curve bls12-381` |
| BN254 Plonkish | `--insecure-dev-setup` | Unsupported for production in 0.1.0 | Not supported out of process |
| Browser WASM | No setup authority | Unsupported | No proving-key or filesystem adapter |

Do not represent a BLS12-381 or Plonkish development proof as production
proving. Achronyme rejects a trusted-store request on those unsupported paths.

## Key-source modes

The default is `deny-insecure-setup`. A prove request without an applicable
trusted key fails before setup randomness is sampled.

For local tests only:

```text
ach --insecure-dev-setup run program.ach
ach --insecure-dev-setup circuit circuit.ach --inputs "out=42,a=6,b=7" --prove
```

For production BN254 Groth16 proving:

```text
ach --trusted-key-dir ./trusted-keys circuit circuit.ach \
  --inputs "out=42,a=6,b=7" --prove
```

The same policy can be selected in project configuration:

```toml
[proving]
trusted_key_dir = "trusted-keys"
```

`insecure_dev_setup = true` is accepted for development, but must not appear
in a production configuration. No environment acknowledgement grants setup
authority. Inspect the effective mode before execution:

```text
ach --error-format json inspect program.ach --manifest
```

The JSON field `proving.key_source` is one of `deny-insecure-setup`,
`insecure-local`, or `trusted-store`.

## Trusted-store contract

The store is addressed by the SHA-256 digest of the exact Achronyme R1CS:

```text
trusted-keys/
  R1CS_SHA256/
    manifest.json
    transcript.json
    proving_key.zkey
```

`ach trusted-setup package` creates this directory without replacing an
existing artifact. It rejects symlinked inputs, malformed or mismatched BN254
headers, invalid contribution hashes, and non-canonical metadata. The loader
then binds the zkey matrices to the in-memory constraint system before witness
evaluation or proof generation.

The transcript records:

- exact R1CS, zkey, and phase-1 SHA-256 digests;
- the published phase-1 BLAKE2b-512 digest and source;
- circuit dimensions;
- the exact ceremony tool version;
- every phase-2 contributor ID or pseudonym and 128-hex contribution hash;
- canonical commands for independently checking phase 1 and the final zkey.

An ID is provenance, not proof of independence. Release reviewers must verify
that at least one contribution was produced by an independently controlled
contributor.

## Pinned ceremony tools

The checked workflow uses `snarkjs@0.7.6`. For the ECDSA circuit it expects
the Hermez powers-of-tau power-21 phase-1 artifact listed by snarkjs:

```text
https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_21.ptau
```

Its published BLAKE2b-512 digest is pinned in
`scripts/proving/common.sh`. The download script verifies both that digest and
the internal powers-of-tau transcript:

```text
scripts/proving/download-phase1.sh ./ceremony/phase1.ptau
```

The upstream procedure and prepared phase-1 digest are documented in the
[official snarkjs repository](https://github.com/iden3/snarkjs).

## Ceremony workflow

### 1. Export and validate the circuit

For an ordinary circuit:

```text
ach --no-config circom circuit.circom \
  --input-file inputs.toml \
  --r1cs ceremony/circuit.r1cs \
  --wtns ceremony/witness.wtns
snarkjs wtns check ceremony/circuit.r1cs ceremony/witness.wtns
```

Use `--low-memory` for a large optimized R1CS export. It cannot be combined
with `--no-optimize`, `--dump-ir`, or `--circuit-stats`, because those modes
require metadata the bounded path deliberately does not retain.

Treat input files and `.wtns` artifacts as confidential whenever they contain
private witnesses. A phase-2 contributor needs the initial zkey, not the
witness.

### 2. Create the zero-contribution phase-2 key

```text
scripts/proving/prepare-phase2.sh \
  --r1cs ceremony/circuit.r1cs \
  --wtns ceremony/witness.wtns \
  --phase1 ceremony/phase1.ptau \
  --work-dir ceremony
```

The resulting `circuit_0000.zkey` is only a challenge. A zero-contribution key
is not a production artifact. A zero-contribution key is not trusted and must
never be installed or used for a production proof.

### 3. Obtain an independent contribution

Transfer only `circuit_0000.zkey` to a contributor outside the release
operator's control. On that contributor's machine, run:

```text
scripts/proving/contribute-phase2.sh \
  --input circuit_0000.zkey \
  --output circuit_final.zkey \
  --name "contributor-pseudonym"
```

snarkjs requests entropy interactively. Do not use its `-e` option in a real
ceremony: command arguments can be captured by shell history, process lists,
CI logs, or monitoring agents. Do not record the interactive session. The
contributor must destroy any temporary entropy and toxic-waste material after
the contribution is complete.

Return the final zkey plus the contributor ID and contribution hash. The
release operator must not accept an ID/hash pair that is absent from
`snarkjs zkey verify` output.

### 4. Verify, package, prove, and cross-verify

```text
scripts/proving/finalize-phase2.sh \
  --r1cs ceremony/circuit.r1cs \
  --wtns ceremony/witness.wtns \
  --phase1 ceremony/phase1.ptau \
  --zkey ceremony/circuit_final.zkey \
  --source circuit.circom \
  --input-file inputs.toml \
  --work-dir ceremony \
  --store trusted-keys \
  --ach-bin target/release/ach \
  --phase1-source "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_21.ptau" \
  --contributor "contributor-pseudonym=CONTRIBUTION_HASH"
```

The finalizer performs all of the following before reporting success:

1. verifies the phase-1 transcript and witness;
2. verifies that the final zkey matches the exact R1CS and phase 1;
3. matches every declared contributor ID/hash against the verified zkey;
4. generates a snarkjs proof and verifies it with Achronyme;
5. packages the immutable trusted store;
6. recompiles and proves with Achronyme without local setup;
7. verifies the Achronyme proof with both Achronyme and snarkjs;
8. writes `release-evidence.json` with hashes, sizes, commit, constraints,
   elapsed time, and maximum RSS for each measured stage.

### 5. Verify detached artifacts

Verification is self-contained and does not read project configuration:

```text
ach verify \
  --proof proof.json \
  --public public.json \
  --vkey verification_key.json \
  --curve bn254 \
  --format json
```

Invalid, malformed, curve-mismatched, or tampered artifacts return a non-zero
exit code.

## Independent audit commands

An auditor with the R1CS, phase 1, and final zkey can rerun:

```text
b2sum phase1.ptau
snarkjs powersoftau verify phase1.ptau
snarkjs zkey verify circuit.r1cs phase1.ptau proving_key.zkey
sha256sum circuit.r1cs phase1.ptau proving_key.zkey transcript.json manifest.json
```

Compare every digest and contribution with the packaged manifest, transcript,
and release evidence. A successful proof does not compensate for a missing or
unverified ceremony record.

## Fast conformance gate

The repository contains a small, local-only ceremony fixture for TDD:

```text
cargo build -p cli
bash test/proving/trusted_setup_workflow.sh
```

This gate tests format compatibility and both proof directions. Its local test
beacon is intentionally public and provides no production trust.
