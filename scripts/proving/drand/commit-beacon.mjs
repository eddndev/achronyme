#!/usr/bin/env node

import { buildCommitment, parsePositiveInteger } from './beacon.mjs'
import { fetchLatestConsensus } from './client.mjs'
import { writeJsonExclusive } from './files.mjs'

function usage () {
  return 'usage: commit-beacon.mjs --output FILE [--lead-rounds N]'
}

function parseArgs (args) {
  let output
  let leadRounds = 1200
  for (let index = 0; index < args.length; index += 2) {
    const flag = args[index]
    const value = args[index + 1]
    if (value === undefined) throw new Error(usage())
    if (flag === '--output') output = value
    else if (flag === '--lead-rounds') {
      leadRounds = parsePositiveInteger(value, 'lead rounds')
    } else throw new Error(`unknown option: ${flag}\n${usage()}`)
  }
  if (output === undefined || output.length === 0) throw new Error(usage())
  if (leadRounds < 20) {
    throw new Error('lead rounds must be at least 20')
  }
  return { output, leadRounds }
}

async function main () {
  const { output, leadRounds } = parseArgs(process.argv.slice(2))
  const anchor = await fetchLatestConsensus()
  const commitment = buildCommitment(
    anchor,
    leadRounds,
    new Date().toISOString()
  )
  await writeJsonExclusive(output, commitment)
  process.stdout.write(
    `committed quicknet round ${commitment.target.round} in ${output}\n`
  )
}

main().catch(error => {
  process.stderr.write(`${error.message}\n`)
  process.exitCode = 1
})
