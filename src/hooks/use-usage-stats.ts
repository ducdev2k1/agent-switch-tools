import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useState, useEffect, useCallback, useRef } from 'react'
import type { UsageStats, UsageLimits } from '@/lib/types'

export function useUsageStats() {
  const [stats, setStats] = useState<UsageStats | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    setLoading(true)
    invoke<UsageStats>('get_usage_stats')
      .then(setStats)
      .catch((e) => console.error('Failed to load usage stats:', e))
      .finally(() => setLoading(false))
  }, [])

  return { stats, loading }
}

const THROTTLE_MS = 120_000 // 120 seconds — matches backend cache TTL

export function useUsageLimits() {
  const [limits, setLimits] = useState<UsageLimits | null>(null)
  const [loading, setLoading] = useState(false)
  const lastRefreshedAt = useRef<number>(0)

  const refresh = useCallback((force = false) => {
    const now = Date.now()
    if (!force && now - lastRefreshedAt.current < THROTTLE_MS) return

    setLoading(true)
    lastRefreshedAt.current = now
    invoke<UsageLimits | null>('get_usage_limits')
      .then((data) => setLimits(data ?? null))
      .catch((e) => console.error('Failed to load usage limits:', e))
      .finally(() => setLoading(false))
  }, [])

  // Initial fetch (force — no throttle on mount)
  useEffect(() => {
    refresh(true)
  }, [refresh])

  // Listen for background worker `usage-updated` events (every 5 min from Rust)
  useEffect(() => {
    const unlisten = listen<UsageLimits>('usage-updated', (event) => {
      setLimits(event.payload)
      lastRefreshedAt.current = Date.now()
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  // Refresh on window focus — throttled to once per 120 seconds
  useEffect(() => {
    const onFocus = () => refresh()
    window.addEventListener('focus', onFocus)
    return () => window.removeEventListener('focus', onFocus)
  }, [refresh])

  return { limits, loading, refresh: () => refresh(true) }
}

export function useProfileUsage(profileName: string | null) {
  const [limits, setLimits] = useState<UsageLimits | null>(null)
  const [loading, setLoading] = useState(false)

  const fetchUsage = useCallback(
    (forceRefresh = false) => {
      if (!profileName) return
      setLoading(true)
      invoke<UsageLimits | null>('get_profile_usage', {
        profileName,
        forceRefresh,
      })
        .then((data) => setLimits(data ?? null))
        .catch((e) => console.error('Failed to load profile usage:', e))
        .finally(() => setLoading(false))
    },
    [profileName],
  )

  useEffect(() => {
    fetchUsage()
  }, [fetchUsage])

  // Force-refresh bypasses the 2-minute server-side cache
  const refresh = useCallback(() => fetchUsage(true), [fetchUsage])

  return { limits, loading, refresh }
}
