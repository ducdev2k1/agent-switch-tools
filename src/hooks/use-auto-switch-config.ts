import type { AutoSwitchConfig, AutoSwitchLogEntry } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useState } from 'react'

const DEFAULTS: AutoSwitchConfig = {
  enabled: false,
  threshold: 90,
  cooldownMinutes: 5,
  lastAutoSwitchAt: null,
  allExhaustedNotified: false,
}

/**
 * Loads and persists the Auto Switch Rule configuration plus its history.
 * Reloads both whenever the backend reports a switch or an exhausted state,
 * so `lastAutoSwitchAt` and the history table stay in sync with the worker.
 */
export function useAutoSwitchConfig() {
  const [config, setConfig] = useState<AutoSwitchConfig>(DEFAULTS)
  const [history, setHistory] = useState<AutoSwitchLogEntry[]>([])
  const [loading, setLoading] = useState(true)

  const reload = useCallback(() => {
    invoke<AutoSwitchConfig>('get_auto_switch_config')
      .then((loaded) => setConfig({ ...DEFAULTS, ...loaded }))
      .catch((e) => console.error('Failed to load auto-switch config:', e))
      .finally(() => setLoading(false))
    invoke<AutoSwitchLogEntry[]>('get_auto_switch_history')
      .then(setHistory)
      .catch((e) => console.error('Failed to load auto-switch history:', e))
  }, [])

  useEffect(() => {
    reload()
  }, [reload])

  useEffect(() => {
    const performed = listen('auto-switch-performed', () => reload())
    const exhausted = listen('auto-switch-exhausted', () => reload())
    return () => {
      performed.then((fn) => fn())
      exhausted.then((fn) => fn())
    }
  }, [reload])

  /** Optimistic save; reverts the local state when the backend rejects it. */
  const save = useCallback(
    async (updated: AutoSwitchConfig) => {
      const previous = config
      setConfig(updated)
      try {
        await invoke('set_auto_switch_config', { config: updated })
      } catch (e) {
        console.error('Failed to save auto-switch config:', e)
        setConfig(previous)
      }
    },
    [config],
  )

  return { config, history, loading, save }
}
