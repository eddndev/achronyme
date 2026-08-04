import assert from 'node:assert/strict'
import test from 'node:test'

import {
  QUICKNET,
  buildCommitment,
  buildEvidence,
  parsePositiveInteger,
  requireConsensus,
  validateCommitment,
  validateEvidence
} from './beacon.mjs'

const anchor = {
  round: 31006443,
  randomness: 'ab'.repeat(32),
  signature: 'cd'.repeat(48)
}

const target = {
  round: 31006463,
  randomness: '12'.repeat(32),
  signature: '34'.repeat(48)
}

const commitmentTime = '2026-08-04T05:51:37.067Z'
const fetchTime = '2026-08-04T05:52:45.708Z'
const publication = 'https://example.invalid/test-only/commitment-31006463'

test('quicknet identity is pinned', () => {
  assert.equal(
    QUICKNET.chainHash,
    '52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971'
  )
  assert.equal(QUICKNET.publicKey.length, 192)
  assert.equal(QUICKNET.periodSeconds, 3)
  assert.equal(QUICKNET.scheme, 'bls-unchained-g1-rfc9380')
  assert.equal(QUICKNET.mirrors.length, 3)
})

test('positive integer parser rejects ambiguous values', () => {
  assert.equal(parsePositiveInteger('42', 'round'), 42)
  for (const value of ['0', '-1', '1.5', ' 2', '2 ', '01', '9007199254740992']) {
    assert.throws(() => parsePositiveInteger(value, 'round'), /round/)
  }
})

test('consensus requires matching verified observations from two mirrors', () => {
  const consensus = requireConsensus([
    { mirror: QUICKNET.mirrors[0], beacon: anchor },
    { mirror: QUICKNET.mirrors[1], beacon: { ...anchor } }
  ])

  assert.deepEqual(consensus, anchor)
  assert.throws(
    () => requireConsensus([{ mirror: QUICKNET.mirrors[0], beacon: anchor }]),
    /at least two mirrors/
  )
  assert.throws(
    () => requireConsensus([
      { mirror: QUICKNET.mirrors[0], beacon: anchor },
      {
        mirror: QUICKNET.mirrors[1],
        beacon: { ...anchor, randomness: 'ef'.repeat(32) }
      }
    ]),
    /mirror consensus failed/
  )
})

test('commitment fixes a future round after the verified anchor', () => {
  const commitment = buildCommitment(anchor, 20, commitmentTime)

  assert.equal(commitment.format, 'achronyme-drand-commitment')
  assert.equal(commitment.version, 1)
  assert.equal(commitment.network.chain_hash, QUICKNET.chainHash)
  assert.equal(commitment.anchor.round, 31006443)
  assert.equal(commitment.target.round, 31006463)
  assert.equal(
    commitment.target.source,
    `${QUICKNET.mirrors[0]}/public/31006463`
  )
  assert.deepEqual(validateCommitment(commitment), commitment)
})

test('commitment validation rejects changed network identity or elapsed targets', () => {
  const commitment = buildCommitment(anchor, 20, commitmentTime)

  assert.throws(
    () => validateCommitment({
      ...commitment,
      network: { ...commitment.network, chain_hash: '00'.repeat(32) }
    }),
    /chain hash/
  )
  assert.throws(
    () => validateCommitment({
      ...commitment,
      target: { ...commitment.target, round: commitment.anchor.round }
    }),
    /after anchor/
  )
  assert.throws(
    () => buildCommitment(anchor, 20, '2026-08-04T05:53:00.000Z'),
    /before target/
  )
})

test('evidence binds the commitment to the exact verified target beacon', () => {
  const commitment = buildCommitment(anchor, 20, commitmentTime)
  const evidence = buildEvidence(commitment, target, fetchTime, publication)

  assert.equal(evidence.format, 'achronyme-drand-beacon')
  assert.equal(evidence.version, 2)
  assert.equal(evidence.verified, true)
  assert.equal(evidence.publication.url, publication)
  assert.match(evidence.publication.commitment_sha256, /^[0-9a-f]{64}$/)
  assert.equal(evidence.beacon.round, 31006463)
  assert.equal(evidence.beacon.randomness, target.randomness)
  assert.equal(evidence.commitment.target.round, 31006463)
  assert.deepEqual(validateEvidence(evidence), evidence)
  assert.throws(
    () => buildEvidence(
      commitment,
      { ...target, round: 121 },
      fetchTime,
      publication
    ),
    /committed round/
  )
  assert.throws(
    () => buildEvidence(
      commitment,
      target,
      '2026-08-04T05:52:00.000Z',
      publication
    ),
    /after target/
  )
  assert.throws(
    () => validateEvidence({
      ...evidence,
      verification: { ...evidence.verification, signature: false }
    }),
    /verification flags/
  )
  assert.throws(
    () => validateEvidence({ ...evidence, publication: undefined }),
    /publication/
  )
})
