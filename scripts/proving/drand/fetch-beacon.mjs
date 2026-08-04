#!/usr/bin/env node

import { buildEvidence, validateCommitment } from './beacon.mjs'
import { fetchRoundConsensus } from './client.mjs'
import { readJson, writeJsonExclusive } from './files.mjs'

function usage () {
  return 'usage: fetch-beacon.mjs --commitment FILE --publication URL --output FILE'
}

function parseArgs (args) {
  let commitment
  let publication
  let output
  for (let index = 0; index < args.length; index += 2) {
    const flag = args[index]
    const value = args[index + 1]
    if (value === undefined) throw new Error(usage())
    if (flag === '--commitment') commitment = value
    else if (flag === '--publication') publication = value
    else if (flag === '--output') output = value
    else throw new Error(`unknown option: ${flag}\n${usage()}`)
  }
  if (commitment === undefined || publication === undefined ||
      output === undefined || commitment.length === 0 ||
      publication.length === 0 || output.length === 0) {
    throw new Error(usage())
  }
  return { commitment, publication, output }
}

async function main () {
  const paths = parseArgs(process.argv.slice(2))
  const commitment = validateCommitment(
    await readJson(paths.commitment, 'commitment')
  )
  const beacon = await fetchRoundConsensus(commitment.target.round)
  const evidence = buildEvidence(
    commitment,
    beacon,
    new Date().toISOString(),
    paths.publication
  )
  await writeJsonExclusive(paths.output, evidence)
  process.stdout.write(
    `verified quicknet round ${beacon.round} in ${paths.output}\n`
  )
}

main().catch(error => {
  process.stderr.write(`${error.message}\n`)
  process.exitCode = 1
})
