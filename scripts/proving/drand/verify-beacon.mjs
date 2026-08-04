#!/usr/bin/env node

import { verifyStoredEvidence } from './client.mjs'
import { readJson } from './files.mjs'

function usage () {
  return 'usage: verify-beacon.mjs --evidence FILE'
}

function parseArgs (args) {
  if (args.length !== 2 || args[0] !== '--evidence' || args[1].length === 0) {
    throw new Error(usage())
  }
  return args[1]
}

async function main () {
  const path = parseArgs(process.argv.slice(2))
  const evidence = await readJson(path, 'beacon evidence')
  const verified = await verifyStoredEvidence(evidence)
  process.stdout.write(`${JSON.stringify(verified)}\n`)
}

main().catch(error => {
  process.stderr.write(`${error.message}\n`)
  process.exitCode = 1
})
