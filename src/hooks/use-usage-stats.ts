import type { RefreshResult, UsageLimits, UsageStats } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useRef, useState } from 'react'

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
    invoke<UsageLimits | null>('get_usage_limits', { forceRefresh: force })
      .then((data) => {
        if (data) setLimits(data)
      })
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

export function useProfileUsage(profileName: string | null, isActive = false) {
  const [limits, setLimits] = useState<UsageLimits | null>(null)
  const [loading, setLoading] = useState(false)
  const prevIsActive = useRef(isActive)
  const hasDataRef = useRef(false)
  const lastFetchedAt = useRef(0)

  const fetchUsage = useCallback(
    (forceRefresh = false) => {
      if (!profileName) return
      // Show the loader only for the first load or an explicit refresh —
      // silent background updates must not blank the quota display.
      if (forceRefresh || !hasDataRef.current) setLoading(true)
      lastFetchedAt.current = Date.now()
      invoke<UsageLimits | null>('get_profile_usage', {
        profileName,
        forceRefresh,
        isActive,
      })
        .then((data) => {
          if (data) {
            hasDataRef.current = true
            setLimits(data)
          }
        })
        .catch((e) => console.error('Failed to load profile usage:', e))
        .finally(() => setLoading(false))
    },
    [profileName, isActive],
  )

  // Force-refresh when isActive status changes (account switch)
  useEffect(() => {
    if (prevIsActive.current !== isActive) {
      prevIsActive.current = isActive
      hasDataRef.current = false
      setLimits(null)
      fetchUsage(true)
    } else {
      fetchUsage()
    }
  }, [fetchUsage, isActive])

  // Refresh on window focus, silently. Throttled to the backend cache TTL so
  // rapid focus switches never reach the Anthropic API (the backend also
  // guards with its own 120s cache — force_refresh is the only bypass).
  useEffect(() => {
    const onFocus = () => {
      if (Date.now() - lastFetchedAt.current < THROTTLE_MS) return
      fetchUsage()
    }
    window.addEventListener('focus', onFocus)
    return () => window.removeEventListener('focus', onFocus)
  }, [fetchUsage])

  // Auto-update from background worker (every 5 min for all profiles)
  useEffect(() => {
    let mounted = true
    const unlisten = listen<Record<string, UsageLimits>>(
      'all-profiles-usage-updated',
      (event) => {
        if (mounted && profileName && event.payload[profileName]) {
          hasDataRef.current = true
          lastFetchedAt.current = Date.now()
          setLimits(event.payload[profileName])
        }
      },
    )
    return () => {
      mounted = false
      unlisten.then((fn) => fn())
    }
  }, [profileName])

  // Force-refresh bypasses the 2-minute server-side cache
  const refresh = useCallback(() => fetchUsage(true), [fetchUsage])

  return { limits, loading, refresh }
}

/** Refresh an expired OAuth token for any profile (active or saved) */
export function useTokenRefresh() {
  const [refreshing, setRefreshing] = useState(false)

  const refreshToken = useCallback(
    async (profileName: string, isActive: boolean): Promise<RefreshResult> => {
      setRefreshing(true)
      try {
        const result = isActive
          ? await invoke<RefreshResult>('refresh_active_token')
          : await invoke<RefreshResult>('refresh_profile_token', {
              profileName,
            })
        return result
      } catch (e) {
        return { success: false, message: String(e) }
      } finally {
        setRefreshing(false)
      }
    },
    [],
  )

  return { refreshToken, refreshing }
}
