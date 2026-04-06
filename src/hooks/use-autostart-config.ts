import { settingsStore } from '@/lib/settings-store'
import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useState } from 'react'

const STORE_KEY = 'autostart_initialized'

/**
 * Hook to manage autostart (launch at login) via tauri-plugin-autostart.
 * On first launch, enables autostart by default and persists the flag.
 */
export function useAutoStartConfig() {
  const [enabled, setEnabled] = useState(false)
  const [loading, setLoading] = useState(true)

  // Read current autostart state from plugin
  useEffect(() => {
    let cancelled = false

    async function init() {
      try {
        // Check if this is the first time the app runs
        const initialized = await settingsStore.get<boolean>(STORE_KEY)

        if (!initialized) {
          // First launch: enable autostart by default
          await invoke('plugin:autostart|enable')
          await settingsStore.set(STORE_KEY, true)
          await settingsStore.save()
          if (!cancelled) setEnabled(true)
        } else {
          // Read current state from plugin
          const isEnabled = await invoke<boolean>('plugin:autostart|is_enabled')
          if (!cancelled) setEnabled(isEnabled)
        }
      } catch (e) {
        console.warn('Autostart plugin not available:', e)
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    init()
    return () => { cancelled = true }
  }, [])

  const toggle = useCallback(async (value: boolean) => {
    try {
      if (value) {
        await invoke('plugin:autostart|enable')
      } else {
        await invoke('plugin:autostart|disable')
      }
      setEnabled(value)
    } catch (e) {
      console.warn('Failed to toggle autostart:', e)
    }
  }, [])

  return { enabled, loading, toggle }
}
