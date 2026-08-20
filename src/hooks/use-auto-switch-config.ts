import type { AutoSwitchConfig, AutoSwitchLogEntry, LogPage } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useState } from 'react'

/** Rows fetched per page, matching the scheduled-priming log. */
export const SWITCH_HISTORY_PAGE_SIZE = 100

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
  const [historyTotal, setHistoryTotal] = useState(0)
  const [historyOffset, setHistoryOffset] = useState(0)
  const [loading, setLoading] = useState(true)

  const reload = useCallback(() => {
    invoke<AutoSwitchConfig>('get_auto_switch_config')
      .then((loaded) => setConfig({ ...DEFAULTS, ...loaded }))
      .catch((e) => console.error('Failed to load auto-switch config:', e))
      .finally(() => setLoading(false))
    invoke<LogPage<AutoSwitchLogEntry>>('get_auto_switch_history', {
      offset: historyOffset,
      limit: SWITCH_HISTORY_PAGE_SIZE,
    })
      .then((page) => {
        setHistory(page.rows)
        setHistoryTotal(page.total)
      })
      .catch((e) => console.error('Failed to load auto-switch history:', e))
  }, [historyOffset])

  useEffect(() => {
    reload()
  }, [reload])

  useEffect(() => {
    // Only refresh in place on the newest page, so paging back through history is
    // not interrupted by a switch landing.
    const refresh = () => {
      if (historyOffset === 0) reload()
    }
    const performed = listen('auto-switch-performed', refresh)
    const exhausted = listen('auto-switch-exhausted', refresh)
    return () => {
      performed.then((fn) => fn())
      exhausted.then((fn) => fn())
    }
  }, [reload, historyOffset])

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

  return {
    config,
    history,
    historyTotal,
    historyOffset,
    setHistoryOffset,
    loading,
    save,
  }
}
