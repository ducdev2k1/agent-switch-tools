import { settingsStore } from '@/lib/settings-store'
import type { WebhookConfig, WebhookResponse } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useRef } from 'react'

const COOLDOWN_MS = 30_000
const STARTUP_DELAY_MS = 15_000
const STORE_KEY = 'webhook'

/** Read fresh webhook config from store (not from React state) */
async function readConfig(): Promise<WebhookConfig | null> {
  try {
    const val = await settingsStore.get<WebhookConfig>(STORE_KEY)
    if (val && typeof val.enabled === 'boolean') return val
    return null
  } catch {
    return null
  }
}

/**
 * Hook that handles webhook trigger modes (manual, on_startup, on_change).
 * Reads fresh config from store each time before sending — no stale closures.
 * Should be mounted at app level (App.tsx) so triggers work globally.
 */
export function useWebhookSender() {
  const lastSentRef = useRef<number>(0)
  const startupFiredRef = useRef(false)

  /** Core send: reads fresh config from store, then invokes Rust */
  const sendFromStore = useCallback(
    async (testMode = false): Promise<WebhookResponse | null> => {
      const cfg = await readConfig()
      if (!cfg || !cfg.enabled || !cfg.url) return null
      try {
        return await invoke<WebhookResponse>('send_webhook', {
          url: cfg.url,
          secret: cfg.secret || null,
          apiKey: cfg.apiKey || null,
          testMode,
          includeCredentials: cfg.includeCredentials,
          includeSessionUsage: cfg.includeSessionUsage ?? true,
          memberEmail: cfg.memberEmail || null,
        })
      } catch (e) {
        return { success: false, statusCode: null, message: String(e) }
      }
    },
    [],
  )

  /** Send with cooldown */
  const sendWithCooldown = useCallback(async () => {
    const now = Date.now()
    if (now - lastSentRef.current < COOLDOWN_MS) return null
    lastSentRef.current = now
    return sendFromStore(false)
  }, [sendFromStore])

  // Trigger: on_startup — fire once after 15s delay
  useEffect(() => {
    if (startupFiredRef.current) return
    startupFiredRef.current = true

    const timer = setTimeout(async () => {
      const cfg = await readConfig()
      if (cfg?.enabled && cfg.triggerMode === 'on_startup' && cfg.url) {
        sendFromStore(false)
      }
    }, STARTUP_DELAY_MS)

    return () => clearTimeout(timer)
  }, [sendFromStore])

  // Trigger: on_change — listen for usage-updated events
  useEffect(() => {
    const unlisten = listen('usage-updated', async () => {
      const cfg = await readConfig()
      if (!cfg?.enabled || cfg.triggerMode !== 'on_change' || !cfg.url) return
      const now = Date.now()
      if (now - lastSentRef.current < COOLDOWN_MS) return
      lastSentRef.current = now
      sendFromStore(false)
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [sendFromStore])

  return {
    sendManual: sendWithCooldown,
    testConnection: () => sendFromStore(true),
  }
}
