import {
  HttpCachingChain,
  HttpChainClient,
  fetchBeacon
} from 'drand-client'

import {
  QUICKNET,
  requireConsensus,
  validateEvidence
} from './beacon.mjs'

const options = Object.freeze({
  disableBeaconVerification: false,
  noCache: true,
  chainVerificationParams: Object.freeze({
    chainHash: QUICKNET.chainHash,
    publicKey: QUICKNET.publicKey
  })
})

const chainInfo = Object.freeze({
  public_key: QUICKNET.publicKey,
  period: QUICKNET.periodSeconds,
  genesis_time: QUICKNET.genesisTime,
  hash: QUICKNET.chainHash,
  groupHash: QUICKNET.groupHash,
  schemeID: QUICKNET.scheme,
  metadata: Object.freeze({ beaconID: QUICKNET.id })
})

async function fetchFromMirror (mirror, round) {
  const chain = new HttpCachingChain(mirror, options)
  const client = new HttpChainClient(chain, options)
  const beacon = round === undefined
    ? await fetchBeacon(client)
    : await fetchBeacon(client, round)
  return { mirror, beacon }
}

export async function fetchLatestConsensus () {
  const latest = await fetchFromMirror(QUICKNET.mirrors[0])
  return fetchRoundConsensus(latest.beacon.round)
}

export async function fetchRoundConsensus (round) {
  const observations = await Promise.all(
    QUICKNET.mirrors.map(mirror => fetchFromMirror(mirror, round))
  )
  return requireConsensus(observations)
}

export async function verifyStoredBeacon (beacon) {
  const client = {
    options,
    get: async round => {
      if (round !== beacon.round) throw new Error('stored beacon round mismatch')
      return beacon
    },
    chain: () => ({
      baseUrl: 'offline:pinned-quicknet',
      info: async () => chainInfo
    })
  }
  await fetchBeacon(client, beacon.round)
}

export async function verifyStoredEvidence (evidenceInput) {
  const evidence = validateEvidence(evidenceInput)
  await verifyStoredBeacon(evidence.commitment.anchor)
  await verifyStoredBeacon(evidence.beacon)
  return {
    round: evidence.beacon.round,
    source: evidence.beacon.source,
    randomness: evidence.beacon.randomness,
    commitment_publication: evidence.publication.url,
    commitment_sha256: evidence.publication.commitment_sha256
  }
}
