/**
 * Build guard: the app version must have a matching top entry in
 * changelog.json (repo root). Fails the build when a release forgets to add its
 * changelog entry, so "What's New" is never stale in a shipped build.
 */
import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'))
const changelog = JSON.parse(
  readFileSync(join(root, 'changelog.json'), 'utf8'),
)

const appVersion = pkg.version
const latest = changelog[0]?.version

if (latest !== appVersion) {
  console.error(
    `\n✖ changelog.json is not updated for v${appVersion} (newest entry: v${latest ?? 'none'}).`,
  )
  console.error(
    `  Add a v${appVersion} entry to the top of changelog.json (both en & vi), then build again.\n`,
  )
  process.exit(1)
}

console.log(`✓ changelog.json matches app version (v${appVersion}).`)
