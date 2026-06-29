import type { AutoPrimeSetting, PrimeDayStat, PrimeResult } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useState } from 'react'

type SettingsMap = Record<string, AutoPrimeSetting>

/**
 * Manages scheduled-priming state: loads per-profile settings, activity log and
 * stats, and exposes mutators. Reloads when the scheduler emits
 * `auto-prime-updated`.
 */
export function useAutoPrime() {
  const [settings, setSettings] = useState<SettingsMap>({})
  const [log, setLog] = useState('')
  const [stats, setStats] = useState<PrimeDayStat[]>([])

  const reload = useCallback(() => {
    invoke<SettingsMap>('get_auto_prime_settings')
      .then(setSettings)
      .catch((e) => console.error('Failed to load auto-prime settings:', e))
    invoke<string>('get_auto_prime_log')
      .then(setLog)
      .catch((e) => console.error('Failed to load auto-prime log:', e))
    invoke<PrimeDayStat[]>('get_auto_prime_stats')
      .then(setStats)
      .catch((e) => console.error('Failed to load auto-prime stats:', e))
  }, [])

  useEffect(() => {
    reload()
  }, [reload])

  useEffect(() => {
    const unlisten = listen('auto-prime-updated', () => reload())
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [reload])

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

  return { settings, log, stats, setAutoPrime, setAll, primeNow, reload }
}
