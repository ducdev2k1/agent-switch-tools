import { settingsStore } from '@/lib/settings-store'
import type { WebhookConfig } from '@/lib/types'
import { useCallback, useEffect, useState } from 'react'

const STORE_KEY = 'webhook'

const DEFAULTS: WebhookConfig = {
  enabled: false,
  url: '',
  secret: '',
  triggerMode: 'manual',
  includeCredentials: false,
  memberEmail: '',
  sessionUsagePeriod: '24h',
  sessionUsageDetailLevel: 'detailed',
}

export function useWebhookConfig() {
  const [config, setConfig] = useState<WebhookConfig>(DEFAULTS)
  const [loading, setLoading] = useState(true)

  // Load from settingsStore on mount
  useEffect(() => {
    settingsStore
      .get<WebhookConfig>(STORE_KEY)
      .then((val) => {
        if (val && typeof val.enabled === 'boolean') {
          setConfig({ ...DEFAULTS, ...val })
        }
      })
      .catch(() => {
        // Store corrupt — use defaults silently
      })
      .finally(() => setLoading(false))
  }, [])

  const save = useCallback(async (updated: WebhookConfig) => {
    setConfig(updated)
    await settingsStore.set(STORE_KEY, updated)
    await settingsStore.save()
  }, [])

  const reset = useCallback(async () => {
    setConfig(DEFAULTS)
    await settingsStore.set(STORE_KEY, DEFAULTS)
    await settingsStore.save()
  }, [])

  return { config, loading, save, reset }
}
