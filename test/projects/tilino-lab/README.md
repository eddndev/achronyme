# Tilino Lab: Private Concurrent Auction

Tilino Lab is Achronyme's versioned end-to-end maturity fixture. It combines
namespace modules, typed array captures across a module boundary, structured
concurrency, bounded channels, owned TCP and file resources, explicit host
capabilities, Poseidon commitments, Merkle membership, and a Groth16 winner
proof in one program.

## Module layout

```text
src/main.ach       application orchestration
src/transport.ach  TCP clients, bounded channel, deadline, and task outcomes
src/auction.ach    commitments, winner proof, and public receipt formatting
src/registry.ach   bidder leaves, Merkle root, path, and direction indices
src/artifacts.ach  concurrent artifact writes and receipt read-back
```

`src/main.ach` remains a thin orchestrator. Internal network and file helpers
remain private to their modules. The `Field[2]` Merkle path and index arrays
cross from `registry.ach` through `main.ach` into a `prove` block in
`auction.ach`, locking the multi-module capture contract.

## Security boundary

The fixture uses `--insecure-dev-setup`. Its proving parameters are local and
development-only. The resulting proof tests language and interoperability
behavior; it is not a production trusted proof. Production requires a
ceremony-derived key for the exact optimized circuit.

Only commitments cross the loopback TCP boundary. Bid amounts and nonces stay
private witness values. The proof exposes the three commitments and the bidder
registry root.

## Run

```text
sh scripts/run-demo.sh
```

The runner grants only its temporary output directory and one exact loopback
address. Override `TILINO_ADDRESS`, `TILINO_OUTPUT_DIR`, `TILINO_ENGINE`, or
the three bid variables to exercise alternate cases.

## Contract

```text
ACH_BIN=/path/to/ach sh test.sh
```

The contract checks interpreter and LLVM JIT execution, detached verification,
tampered public-input rejection, capability and proving-authority denial,
structured-concurrency exhaustion, a false-winner constraint failure, exact
R1CS statistics, and the standalone AOT capability boundary. All generated
keys and artifacts live under a temporary directory removed on exit.
