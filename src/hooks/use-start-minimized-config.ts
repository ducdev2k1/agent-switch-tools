import { settingsStore } from '@/lib/settings-store'
import { useCallback, useEffect, useState } from 'react'

const STORE_KEY = 'start_minimized'

/**
 * Hook to manage whether the app window stays hidden in the tray on launch
 * instead of showing the dashboard immediately. Read on the Rust side during
 * setup, before the window is shown, to avoid a visible flash.
 */
export function useStartMinimizedConfig() {
  const [enabled, setEnabledState] = useState(false)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let cancelled = false

    async function init() {
      try {
        const value = await settingsStore.get<boolean>(STORE_KEY)
        if (!cancelled) setEnabledState(value ?? false)
      } catch (e) {
        console.warn('Failed to read start_minimized setting:', e)
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    init()
    return () => { cancelled = true }
  }, [])

  const toggle = useCallback(async (value: boolean) => {
    try {
      await settingsStore.set(STORE_KEY, value)
      await settingsStore.save()
      setEnabledState(value)
    } catch (e) {
      console.warn('Failed to toggle start_minimized setting:', e)
    }
  }, [])

  return { enabled, loading, toggle }
}
