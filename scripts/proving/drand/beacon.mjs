import { createHash } from 'node:crypto'

export const QUICKNET = Object.freeze({
  id: 'quicknet',
  chainHash: '52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971',
  publicKey: '83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a',
  periodSeconds: 3,
  genesisTime: 1692803367,
  groupHash: 'f477d5c89f21a17c863a7f937c6a6d15859414d2be09cd448d4279af331c5d3e',
  scheme: 'bls-unchained-g1-rfc9380',
  client: 'drand-client@1.4.2',
  mirrors: Object.freeze([
    'https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971',
    'https://api2.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971',
    'https://api3.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971'
  ])
})

export function parsePositiveInteger (value, label) {
  if (typeof value !== 'string' || !/^[1-9][0-9]*$/.test(value)) {
    throw new Error(`${label} must be a positive integer`)
  }
  const parsed = Number(value)
  if (!Number.isSafeInteger(parsed)) {
    throw new Error(`${label} must be a safe positive integer`)
  }
  return parsed
}

function requirePositiveInteger (value, label) {
  if (!Number.isSafeInteger(value) || value < 1) {
    throw new Error(`${label} must be a safe positive integer`)
  }
  return value
}

function requireTimestamp (value, label) {
  if (typeof value !== 'string') {
    throw new Error(`${label} must be an ISO-8601 timestamp`)
  }
  try {
    if (new Date(value).toISOString() !== value) {
      throw new Error()
    }
  } catch {
    throw new Error(`${label} must be an ISO-8601 timestamp`)
  }
  return value
}

function requireHex (value, length, label) {
  const pattern = new RegExp(`^[0-9a-f]{${length}}$`)
  if (typeof value !== 'string' || !pattern.test(value)) {
    throw new Error(`${label} must be lowercase ${length}-hex`)
  }
  return value
}

function requireHttpsUrl (value, label) {
  if (typeof value !== 'string' || value.length > 2048 ||
      !/^https:\/\/[^\s]+$/.test(value)) {
    throw new Error(`${label} must be a non-empty HTTPS URL`)
  }
  return value
}

function jsonSha256 (value) {
  return createHash('sha256')
    .update(`${JSON.stringify(value, null, 2)}\n`, 'utf8')
    .digest('hex')
}

function normalizeBeacon (beacon) {
  if (beacon === null || typeof beacon !== 'object') {
    throw new Error('beacon must be an object')
  }
  return {
    round: requirePositiveInteger(beacon.round, 'beacon round'),
    randomness: requireHex(beacon.randomness, 64, 'beacon randomness'),
    signature: requireHex(beacon.signature, 96, 'beacon signature')
  }
}

function requireExactArray (actual, expected, label) {
  if (!Array.isArray(actual) ||
      actual.length !== expected.length ||
      actual.some((value, index) => value !== expected[index])) {
    throw new Error(`${label} does not match pinned quicknet configuration`)
  }
}

export function requireConsensus (observations) {
  if (!Array.isArray(observations) || observations.length < 2) {
    throw new Error('beacon consensus requires at least two mirrors')
  }

  const mirrors = new Set()
  const beacons = observations.map(({ mirror, beacon }) => {
    if (!QUICKNET.mirrors.includes(mirror)) {
      throw new Error(`unrecognized quicknet mirror: ${mirror}`)
    }
    if (mirrors.has(mirror)) {
      throw new Error(`duplicate quicknet mirror: ${mirror}`)
    }
    mirrors.add(mirror)
    return normalizeBeacon(beacon)
  })

  const expected = beacons[0]
  for (const beacon of beacons.slice(1)) {
    if (beacon.round !== expected.round ||
        beacon.randomness !== expected.randomness ||
        beacon.signature !== expected.signature) {
      throw new Error('quicknet mirror consensus failed')
    }
  }
  return expected
}

function networkRecord () {
  return {
    id: QUICKNET.id,
    chain_hash: QUICKNET.chainHash,
    public_key: QUICKNET.publicKey,
    period_seconds: QUICKNET.periodSeconds,
    scheme: QUICKNET.scheme,
    client: QUICKNET.client,
    mirrors: [...QUICKNET.mirrors]
  }
}

function roundSources (round) {
  return QUICKNET.mirrors.map(mirror => `${mirror}/public/${round}`)
}

function roundTimeMillis (round) {
  return (QUICKNET.genesisTime + (round - 1) * QUICKNET.periodSeconds) * 1000
}

export function buildCommitment (anchorInput, leadRounds, createdAt) {
  const anchor = normalizeBeacon(anchorInput)
  requirePositiveInteger(leadRounds, 'lead rounds')
  requireTimestamp(createdAt, 'commitment creation time')
  const targetRound = anchor.round + leadRounds
  if (!Number.isSafeInteger(targetRound)) {
    throw new Error('target round exceeds the safe integer range')
  }
  const creationMillis = Date.parse(createdAt)
  if (creationMillis < roundTimeMillis(anchor.round)) {
    throw new Error('commitment creation time must be after anchor publication')
  }
  if (creationMillis >= roundTimeMillis(targetRound)) {
    throw new Error('commitment creation time must be before target publication')
  }

  return {
    format: 'achronyme-drand-commitment',
    version: 1,
    created_at: createdAt,
    publication_required_before_target: true,
    network: networkRecord(),
    anchor: {
      ...anchor,
      sources: roundSources(anchor.round)
    },
    target: {
      round: targetRound,
      lead_rounds: leadRounds,
      source: roundSources(targetRound)[0],
      mirrors: roundSources(targetRound)
    }
  }
}

export function validateCommitment (commitment) {
  if (commitment === null || typeof commitment !== 'object') {
    throw new Error('commitment must be an object')
  }
  if (commitment.format !== 'achronyme-drand-commitment' ||
      commitment.version !== 1) {
    throw new Error('unsupported drand commitment format')
  }
  requireTimestamp(commitment.created_at, 'commitment creation time')
  if (commitment.publication_required_before_target !== true) {
    throw new Error('commitment must require publication before its target')
  }

  const network = commitment.network
  if (network === null || typeof network !== 'object') {
    throw new Error('commitment network must be an object')
  }
  if (network.id !== QUICKNET.id) {
    throw new Error('commitment network ID does not match quicknet')
  }
  if (network.chain_hash !== QUICKNET.chainHash) {
    throw new Error('commitment chain hash does not match quicknet')
  }
  if (network.public_key !== QUICKNET.publicKey) {
    throw new Error('commitment public key does not match quicknet')
  }
  if (network.period_seconds !== QUICKNET.periodSeconds ||
      network.scheme !== QUICKNET.scheme ||
      network.client !== QUICKNET.client) {
    throw new Error('commitment network parameters do not match quicknet')
  }
  requireExactArray(network.mirrors, QUICKNET.mirrors, 'commitment mirrors')

  const anchor = normalizeBeacon(commitment.anchor)
  requireExactArray(
    commitment.anchor.sources,
    roundSources(anchor.round),
    'anchor sources'
  )
  const target = commitment.target
  if (target === null || typeof target !== 'object') {
    throw new Error('commitment target must be an object')
  }
  const targetRound = requirePositiveInteger(target.round, 'target round')
  if (targetRound <= anchor.round) {
    throw new Error('committed round must be after anchor round')
  }
  if (target.lead_rounds !== targetRound - anchor.round) {
    throw new Error('commitment lead rounds do not match target round')
  }
  const sources = roundSources(targetRound)
  if (target.source !== sources[0]) {
    throw new Error('commitment target source does not match quicknet')
  }
  requireExactArray(target.mirrors, sources, 'target mirrors')
  const creationMillis = Date.parse(commitment.created_at)
  if (creationMillis < roundTimeMillis(anchor.round)) {
    throw new Error('commitment creation time must be after anchor publication')
  }
  if (creationMillis >= roundTimeMillis(targetRound)) {
    throw new Error('commitment creation time must be before target publication')
  }
  return commitment
}

export function buildEvidence (
  commitmentInput,
  beaconInput,
  fetchedAt,
  publicationUrl
) {
  const commitment = validateCommitment(commitmentInput)
  const beacon = normalizeBeacon(beaconInput)
  requireTimestamp(fetchedAt, 'beacon fetch time')
  requireHttpsUrl(publicationUrl, 'commitment publication')
  if (beacon.round !== commitment.target.round) {
    throw new Error('verified beacon does not match committed round')
  }
  if (Date.parse(fetchedAt) < roundTimeMillis(beacon.round)) {
    throw new Error('beacon fetch time must be after target publication')
  }

  return {
    format: 'achronyme-drand-beacon',
    version: 2,
    fetched_at: fetchedAt,
    verified: true,
    verification: {
      pinned_chain_identity: true,
      signature: true,
      mirror_consensus: true,
      client: QUICKNET.client
    },
    publication: {
      url: publicationUrl,
      commitment_sha256: jsonSha256(commitment)
    },
    commitment,
    beacon: {
      ...beacon,
      source: commitment.target.source,
      mirrors: [...commitment.target.mirrors]
    }
  }
}

export function validateEvidence (evidence) {
  if (evidence === null || typeof evidence !== 'object') {
    throw new Error('beacon evidence must be an object')
  }
  if (evidence.format !== 'achronyme-drand-beacon' || evidence.version !== 2) {
    throw new Error('unsupported drand beacon evidence format')
  }
  requireTimestamp(evidence.fetched_at, 'beacon fetch time')
  if (evidence.verified !== true ||
      evidence.verification?.pinned_chain_identity !== true ||
      evidence.verification?.signature !== true ||
      evidence.verification?.mirror_consensus !== true) {
    throw new Error('beacon evidence verification flags are incomplete')
  }
  if (evidence.verification.client !== QUICKNET.client) {
    throw new Error('beacon evidence client does not match pinned version')
  }

  if (evidence.publication === null ||
      typeof evidence.publication !== 'object') {
    throw new Error('beacon evidence publication must be an object')
  }
  requireHttpsUrl(evidence.publication.url, 'commitment publication')
  requireHex(
    evidence.publication.commitment_sha256,
    64,
    'published commitment SHA-256'
  )

  const commitment = validateCommitment(evidence.commitment)
  if (evidence.publication.commitment_sha256 !== jsonSha256(commitment)) {
    throw new Error('published commitment SHA-256 does not match commitment')
  }
  const beacon = normalizeBeacon(evidence.beacon)
  if (beacon.round !== commitment.target.round) {
    throw new Error('beacon evidence does not match committed round')
  }
  if (evidence.beacon.source !== commitment.target.source) {
    throw new Error('beacon evidence source does not match commitment')
  }
  requireExactArray(
    evidence.beacon.mirrors,
    commitment.target.mirrors,
    'beacon evidence mirrors'
  )
  if (Date.parse(evidence.fetched_at) < roundTimeMillis(beacon.round)) {
    throw new Error('beacon fetch time must be after target publication')
  }
  return evidence
}
