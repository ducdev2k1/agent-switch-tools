import { settingsStore } from '@/lib/settings-store'
import { useCallback, useEffect, useState } from 'react'

const STORE_KEY = 'autoUpdate'

interface AutoUpdateConfig {
  enabled: boolean
}

const DEFAULTS: AutoUpdateConfig = { enabled: true }

export function useAutoUpdateConfig() {
  const [enabled, setEnabledState] = useState(DEFAULTS.enabled)

  // Load from settingsStore on mount
  useEffect(() => {
    settingsStore
      .get<AutoUpdateConfig>(STORE_KEY)
      .then((val) => {
        if (val && typeof val.enabled === 'boolean') {
          setEnabledState(val.enabled)
        }
      })
      .catch(() => {
        // Store corrupt or missing — use defaults
      })
  }, [])

  const setEnabled = useCallback((value: boolean) => {
    setEnabledState(value)
    settingsStore
      .set(STORE_KEY, { enabled: value })
      .then(() => settingsStore.save())
  }, [])

  return { enabled, setEnabled }
}
