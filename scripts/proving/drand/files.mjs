import { open, rename, rm } from 'node:fs/promises'

export async function readJson (path, label) {
  let text
  try {
    const handle = await open(path, 'r')
    try {
      text = await handle.readFile('utf8')
    } finally {
      await handle.close()
    }
  } catch (error) {
    throw new Error(`cannot read ${label} ${path}: ${error.message}`)
  }

  try {
    return JSON.parse(text)
  } catch (error) {
    throw new Error(`invalid JSON in ${label} ${path}: ${error.message}`)
  }
}

export async function writeJsonExclusive (path, value) {
  const partial = `${path}.partial`
  let handle
  try {
    handle = await open(partial, 'wx', 0o600)
    await handle.writeFile(`${JSON.stringify(value, null, 2)}\n`, 'utf8')
    await handle.sync()
    await handle.close()
    handle = undefined
    await rename(partial, path)
  } catch (error) {
    if (handle !== undefined) {
      await handle.close().catch(() => {})
    }
    await rm(partial, { force: true }).catch(() => {})
    throw error
  }
}
