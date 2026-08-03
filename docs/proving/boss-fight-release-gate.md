# ECDSA boss-fight release gate

The 0.1.0 cryptographic release gate is the complete BN254 Groth16 lifecycle
for `test/circomlib/ecdsa_verify_test.circom`, not the historical compile-only
constraint counter. The gate remains open until a measured run exports and
checks the witness, completes the ceremony, proves, and cross-verifies.

## What this gate proves

A passing run demonstrates that one exact Achronyme commit can:

1. compile the vendored secp256k1 ECDSA verifier;
2. export a valid optimized R1CS and witness from the fixed public fixture;
3. fit that R1CS in the selected phase-1 power;
4. bind an independently contributed phase-2 zkey to that exact R1CS;
5. generate a proof in snarkjs and verify it in Achronyme;
6. generate a proof natively in Achronyme without local setup and verify it in
   snarkjs;
7. report artifact hashes, sizes, wall time, and maximum RSS by stage.

Compilation or constraint counting alone does not pass this gate.

## Compute host

The first measured attempt uses an `n2-standard-8` Compute Engine VM: 8 vCPUs
and 32 GB memory. This choice is a testable starting envelope; it does not
prove that 32 GB is sufficient. If the bounded run fails, preserve its stage,
exit status, and RSS evidence before selecting a larger machine.

Google currently lists `n2-standard-8` as 8 vCPUs and 32 GB in the
[Compute Engine machine-family reference](https://cloud.google.com/compute/docs/general-purpose-machines).
Use a 150 GB `pd-balanced` boot disk so Rust builds, phase-1 data, R1CS, witness,
zkey, proofs, and duplicate cross-check outputs have headroom. Balanced
Persistent Disk is the SSD-backed general-purpose option described in the
[official disk guide](https://cloud.google.com/compute/docs/disks/persistent-disks).

Example creation command, with project and zone chosen by the operator:

```text
gcloud compute instances create achronyme-proving-010 \
  --project PROJECT_ID \
  --zone ZONE \
  --machine-type n2-standard-8 \
  --boot-disk-type pd-balanced \
  --boot-disk-size 150GB \
  --image-family ubuntu-2404-lts-amd64 \
  --image-project ubuntu-os-cloud
```

The command shape is defined by the
[official gcloud instance-create reference](https://cloud.google.com/sdk/gcloud/reference/compute/instances/create).
Creation, billing, firewall selection, and deletion are operator actions; the
repository workflow does not create cloud resources.

Use a disposable VM without unrelated credentials or workloads. Copy the
release candidate source at an exact commit. Do not perform the independent
phase-2 contribution on this VM under the release operator's control.

## Host prerequisites

Install the pinned toolchain on the VM:

```text
sudo apt-get update
sudo apt-get install -y build-essential curl jq nodejs npm time util-linux
npm install --global snarkjs@0.7.6
cargo build --release -p cli
snarkjs --version
target/release/ach --version
git rev-parse HEAD
git status --short
```

The source checkout must be clean at the tested commit. Do not benchmark a
debug binary or a checkout with unrecorded source changes.

## Stage 1: bounded R1CS and witness export

Inspect the exact command without allocating the circuit:

```text
scripts/proving/run-boss-fight.sh \
  --ach-bin target/release/ach \
  --work-dir /var/tmp/achronyme-0.1.0-proving \
  --dry-run
```

Run the export with the default six-hour timeout, 30 GiB virtual-address
limit, and 20 GiB free-disk preflight:

```text
scripts/proving/run-boss-fight.sh \
  --ach-bin target/release/ach \
  --work-dir /var/tmp/achronyme-0.1.0-proving
```

The harness uses `ach circom --low-memory`, the fixed fixture at
`test/proving/ecdsa_verify.inputs.toml`, GNU `time`, `timeout`, and `prlimit`.
It checks the resulting witness with snarkjs and rejects the output if the
actual constraint domain exceeds power 21. It writes
`bossfight-export.json`, `export.metrics.jsonl`, and `export.log`.

The memory limit is virtual address space, not a claim that peak RSS equals 30
GiB. The evidence records measured maximum RSS separately.

## Stage 2: verified phase 1 and phase-2 challenge

```text
scripts/proving/download-phase1.sh \
  /var/tmp/achronyme-0.1.0-proving/phase1.ptau

scripts/proving/prepare-phase2.sh \
  --r1cs /var/tmp/achronyme-0.1.0-proving/export/circuit.r1cs \
  --wtns /var/tmp/achronyme-0.1.0-proving/export/witness.wtns \
  --phase1 /var/tmp/achronyme-0.1.0-proving/phase1.ptau \
  --work-dir /var/tmp/achronyme-0.1.0-proving
```

The resulting `circuit_0000.zkey` has zero phase-2 contributions and is not a
production key.

## Stage 3: independent phase-2 contribution

Transfer `circuit_0000.zkey` to an independently controlled contributor. The
contributor follows the interactive command in
[the trusted-setup guide](trusted-setup.md#3-obtain-an-independent-contribution).
Do not transfer the witness or private inputs. Do not pass entropy through a
command argument or record the session.

Return `circuit_final.zkey`, the contributor ID or pseudonym, and the
contribution hash.

## Stage 4: final proof and cross-verification

```text
scripts/proving/finalize-phase2.sh \
  --r1cs /var/tmp/achronyme-0.1.0-proving/export/circuit.r1cs \
  --wtns /var/tmp/achronyme-0.1.0-proving/export/witness.wtns \
  --phase1 /var/tmp/achronyme-0.1.0-proving/phase1.ptau \
  --zkey /var/tmp/achronyme-0.1.0-proving/circuit_final.zkey \
  --source test/circomlib/ecdsa_verify_test.circom \
  --input-file test/proving/ecdsa_verify.inputs.toml \
  --lib test/circomlib \
  --work-dir /var/tmp/achronyme-0.1.0-proving \
  --store /var/tmp/achronyme-0.1.0-trusted-keys \
  --ach-bin target/release/ach \
  --phase1-source "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_21.ptau" \
  --contributor "CONTRIBUTOR_ID=CONTRIBUTION_HASH"
```

This stage intentionally recompiles the circuit for the native trusted-key
proof. Equality of the ceremony R1CS and the re-exported Achronyme R1CS is a
checked invariant, not an assumption.

## Required release evidence

Before the 0.1.0 release is approved, review all of these outputs:

- `bossfight-export.json`;
- `release-evidence.json`;
- `export.metrics.jsonl`, `prepare.metrics.jsonl`, and
  `finalize.metrics.jsonl`;
- R1CS, witness, phase-1, final-zkey, transcript, and manifest hashes;
- verified contributor IDs/hashes;
- successful snarkjs-to-Achronyme and Achronyme-to-snarkjs verification;
- the exact tested Git commit and release binary version.

The final evidence must contain no witness values, contributor entropy, toxic
waste, or proving-key bytes. Hashes and resource measurements are safe to
review. The `.wtns` and input artifacts remain confidential when a different
circuit carries private data.

After evidence and required artifacts are copied to their intended release
storage, stop the VM to end compute charges. Delete the VM and disk only after
the copied hashes have been independently rechecked.
