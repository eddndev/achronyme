# Trusted zkey test fixture

`trusted_basic_arithmetic.zkey.b64` is a local-only interoperability fixture.
It was generated with snarkjs 0.7.6 from the R1CS exported for
`test/circuit/basic_arithmetic.ach` with `out=42`, `a=6`, and `b=7`.

The decoded zkey has SHA-256
`a86566942cf29175d9318b58459a01bafd58b74ad74e10c341bd01e18fb4b49d`.
The exact R1CS has SHA-256
`d641de2416205f323639887a159ce421142bea5d60972f41ea0777ba0a2a5082`.

This fixture does not establish a production ceremony. Its phase-1 and
phase-2 entropy was created by one local test operator. It must never be used
as a production proving key or cited as evidence of independent contribution.
