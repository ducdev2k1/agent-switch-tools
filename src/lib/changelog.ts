/**
 * In-app changelog.
 *
 * Single source of truth is `changelog.json` (repo root). It is fetched from
 * GitHub at runtime so release highlights can be updated by editing that file
 * and pushing — no rebuild required. The same JSON is bundled at build time as
 * an offline fallback, and the last successful fetch is cached to disk by the
 * Rust `fetch_changelog` command.
 *
 * Newest version first. Each entry carries both locales; the viewer picks the
 * active language at render time. Keep entries concise (user-facing highlights,
 * not commit-level detail) — full release notes live in docs/release-notes-*.
 */

import { invoke } from '@tauri-apps/api/core'
import bundled from '../../changelog.json'

export interface ChangelogEntry {
  version: string
  date: string // ISO yyyy-mm-dd
  en: string[]
  vi: string[]
}

/** Bundled snapshot — the offline fallback shipped with this build. */
export const CHANGELOG: ChangelogEntry[] = bundled as ChangelogEntry[]

/**
 * Load the changelog, preferring the latest copy fetched from GitHub.
 * Falls back to the disk cache (handled in Rust) and finally to the bundled
 * snapshot when offline or running outside Tauri.
 */
export async function loadChangelog(): Promise<ChangelogEntry[]> {
  try {
    const remote = await invoke<ChangelogEntry[]>('fetch_changelog')
    if (Array.isArray(remote) && remote.length > 0) return remote
  } catch {
    // offline, fetch failed, or not running in Tauri — use the bundled snapshot
  }
  return CHANGELOG
}
