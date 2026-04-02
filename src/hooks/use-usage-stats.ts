import { invoke } from '@tauri-apps/api/core'
import { useState, useEffect, useCallback } from 'react'
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

export function useUsageLimits() {
  const [limits, setLimits] = useState<UsageLimits | null>(null)
  const [loading, setLoading] = useState(false)

  const refresh = useCallback(() => {
    setLoading(true)
    invoke<UsageLimits | null>('get_usage_limits')
      .then((data) => setLimits(data ?? null))
      .catch((e) => console.error('Failed to load usage limits:', e))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    refresh()
  }, [refresh])

  return { limits, loading, refresh }
}

export function useProfileUsage(profileName: string | null) {
  const [limits, setLimits] = useState<UsageLimits | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (!profileName) return
    setLoading(true)
    invoke<UsageLimits | null>('get_profile_usage', { profileName })
      .then((data) => setLimits(data ?? null))
      .catch((e) => console.error('Failed to load profile usage:', e))
      .finally(() => setLoading(false))
  }, [profileName])

  return { limits, loading }
}
