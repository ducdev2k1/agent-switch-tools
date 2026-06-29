import type { UsageReport } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useState } from 'react'

/** Date-range options exposed by the usage view (days; 0 = all time). */
export type UsageRange = 7 | 30 | 90 | 0

/**
 * Loads the Claude Code cost/usage report for the selected range and refetches
 * when the background worker emits `usage-changed` (every 5 minutes).
 */
export function useUsageReport(range: UsageRange) {
  const [report, setReport] = useState<UsageReport | null>(null)
  const [loading, setLoading] = useState(false)

  const load = useCallback(() => {
    setLoading(true)
    invoke<UsageReport>('get_usage', { rangeDays: range })
      .then(setReport)
      .catch((e) => console.error('Failed to load usage report:', e))
      .finally(() => setLoading(false))
  }, [range])

  useEffect(() => {
    load()
  }, [load])

  useEffect(() => {
    const unlisten = listen('usage-changed', () => load())
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [load])

  return { report, loading, reload: load }
}
