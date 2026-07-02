import type { UsageReport } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useRef, useState } from 'react'

/** Date-range options exposed by the usage view (days; 1 = today, 0 = all time). */
export type UsageRange = 1 | 7 | 30 | 90 | 0

/** Focus refreshes are throttled — report parsing is local but not free. */
const FOCUS_THROTTLE_MS = 30_000

/**
 * Loads the Claude Code cost/usage report for the selected range. Refetches
 * silently (no loading dim) on window focus and when the background worker
 * emits `usage-changed` (every 5 minutes); the loader only shows on the first
 * load and on range changes.
 */
export function useUsageReport(range: UsageRange) {
  const [report, setReport] = useState<UsageReport | null>(null)
  const [loading, setLoading] = useState(false)
  const hasDataRef = useRef(false)
  const lastLoadedAt = useRef(0)

  const load = useCallback(
    (silent = false) => {
      if (!silent || !hasDataRef.current) setLoading(true)
      lastLoadedAt.current = Date.now()
      invoke<UsageReport>('get_usage', { rangeDays: range })
        .then((data) => {
          hasDataRef.current = true
          setReport(data)
        })
        .catch((e) => console.error('Failed to load usage report:', e))
        .finally(() => setLoading(false))
    },
    [range],
  )

  // Initial load + range change (data shape changes → show the loader again)
  useEffect(() => {
    hasDataRef.current = false
    load()
  }, [load])

  useEffect(() => {
    const unlisten = listen('usage-changed', () => load(true))
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [load])

  // Refresh when the window regains focus — the usual "switch back from
  // Claude Code to check numbers" moment. Local parsing only, no API calls.
  useEffect(() => {
    const onFocus = () => {
      if (Date.now() - lastLoadedAt.current < FOCUS_THROTTLE_MS) return
      load(true)
    }
    window.addEventListener('focus', onFocus)
    return () => window.removeEventListener('focus', onFocus)
  }, [load])

  return { report, loading, reload: load }
}
