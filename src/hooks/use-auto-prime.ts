import type {
  AutoPrimeSetting,
  LogPage,
  PrimeDayStat,
  PrimeLogEntry,
  PrimeResult,
} from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useState } from 'react'

type SettingsMap = Record<string, AutoPrimeSetting>

/** Rows fetched per page. The log is capped backend-side, but paging keeps the
 *  IPC payload and the rendered table small regardless of how long it grows. */
export const PRIME_LOG_PAGE_SIZE = 100

/**
 * Manages scheduled-priming state: loads per-profile settings, one page of the
 * activity log and the stats table, and exposes mutators. Reloads when the
 * scheduler emits `auto-prime-updated`.
 */
export function useAutoPrime() {
  const [settings, setSettings] = useState<SettingsMap>({})
  const [logRows, setLogRows] = useState<PrimeLogEntry[]>([])
  const [logTotal, setLogTotal] = useState(0)
  const [logOffset, setLogOffset] = useState(0)
  const [stats, setStats] = useState<PrimeDayStat[]>([])

  const loadLog = useCallback((offset: number) => {
    invoke<LogPage<PrimeLogEntry>>('get_auto_prime_log_page', {
      offset,
      limit: PRIME_LOG_PAGE_SIZE,
    })
      .then((page) => {
        setLogRows(page.rows)
        setLogTotal(page.total)
      })
      .catch((e) => console.error('Failed to load auto-prime log:', e))
  }, [])

  const reload = useCallback(() => {
    invoke<SettingsMap>('get_auto_prime_settings')
      .then(setSettings)
      .catch((e) => console.error('Failed to load auto-prime settings:', e))
    invoke<PrimeDayStat[]>('get_auto_prime_stats')
      .then(setStats)
      .catch((e) => console.error('Failed to load auto-prime stats:', e))
    loadLog(logOffset)
  }, [loadLog, logOffset])

  useEffect(() => {
    reload()
  }, [reload])

  useEffect(() => {
    const unlisten = listen('auto-prime-updated', () => {
      // Only refresh in place on the newest page: pulling the rug out from under
      // someone reading page 12 would be worse than showing slightly stale rows.
      if (logOffset === 0) reload()
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [reload, logOffset])

  const setAutoPrime = useCallback(
    (name: string, enabled: boolean, time: string) =>
      invoke('set_auto_prime', { name, enabled, time }).then(reload),
    [reload],
  )

  const setAll = useCallback(
    (names: string[], enabled: boolean, time: string) =>
      invoke('set_auto_prime_all', { names, enabled, time }).then(reload),
    [reload],
  )

  const primeNow = useCallback(
    (name: string) =>
      invoke<PrimeResult>('prime_now', { name }).then((result) => {
        reload()
        return result
      }),
    [reload],
  )

  return {
    settings,
    logRows,
    logTotal,
    logOffset,
    setLogOffset,
    stats,
    setAutoPrime,
    setAll,
    primeNow,
    reload,
  }
}
