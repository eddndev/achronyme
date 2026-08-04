import assert from 'node:assert/strict'
import test from 'node:test'

import { buildCommitment, buildEvidence } from './beacon.mjs'
import {
  verifyStoredBeacon,
  verifyStoredEvidence
} from './client.mjs'

const anchor = {
  round: 31006443,
  randomness: '93353da5a67d1ff7ced5f59824f31f4d6b2845f661a71bc2ccb3409d3389642c',
  signature: 'b5467d8714a59bebcd874e7162b34351cc0818f7a0d63dd167a07398b012a8cdcf0a9f379c616d00f098195008df2612'
}

const beacon = {
  round: 31006463,
  randomness: '06664dcb57258c3ad1142e1f19575f3e597d29ee8eb49e957355dbab9d6935c9',
  signature: 'b2a038a125417dadfbfbe3e285efb5c0b96ad4195b3bf30c79b02dd940a223f39fa803f63eb55709774a8220acc916b6'
}

test('stored quicknet beacon is verified cryptographically offline', async t => {
  t.mock.method(console, 'error', () => {})
  await assert.doesNotReject(() => verifyStoredBeacon(beacon))
  await assert.rejects(
    () => verifyStoredBeacon({ ...beacon, randomness: '00'.repeat(32) }),
    /not valid/
  )
})

test('stored evidence verifies both commitment anchor and target', async () => {
  const commitment = buildCommitment(
    anchor,
    20,
    '2026-08-04T05:51:37.067Z'
  )
  const evidence = buildEvidence(
    commitment,
    beacon,
    '2026-08-04T05:52:45.708Z',
    'https://example.invalid/test-only/commitment-31006463'
  )

  assert.deepEqual(await verifyStoredEvidence(evidence), {
    round: 31006463,
    source: commitment.target.source,
    randomness: beacon.randomness,
    commitment_publication:
      'https://example.invalid/test-only/commitment-31006463',
    commitment_sha256:
      '54703c6c0df8236e97524ec5f0aaa8733566cf8131188d280a84b9b3f0e18a59'
  })
})
