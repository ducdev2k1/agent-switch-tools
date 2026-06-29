import { isIdeQuotaSupported, type IdeType, type UsageLimits } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useState } from 'react'

/**
 * Module-level cache of last-known usage per (ide, profile, active), so remounting on tab
 * switch shows the previous value instantly and refreshes in the background.
 */
const usageCache = new Map<string, UsageLimits | null>()
const cacheKey = (ide: IdeType, profile: string, active: boolean) =>
  `${ide}:${profile}:${active}`

/**
 * Hook to fetch and manage quota/usage limits for a specific IDE profile.
 */
export function useIdeUsage(
  ideType: IdeType,
  profileName: string,
  isActive: boolean,
) {
  const key = cacheKey(ideType, profileName, isActive)
  const [usage, setUsage] = useState<UsageLimits | null>(
    () => usageCache.get(key) ?? null,
  )
  const [loading, setLoading] = useState(() => !usageCache.has(key))
  const [error, setError] = useState<string | null>(null)

  const fetchUsage = useCallback(
    async (forceRefresh = false) => {
      if (!isIdeQuotaSupported(ideType)) {
        setUsage(null)
        return
      }
      // Only flash the skeleton when we have nothing cached yet.
      if (!usageCache.has(key)) setLoading(true)
      setError(null)
      try {
        const result = await invoke<UsageLimits | null>('get_ide_usage', {
          ideType,
          profileName,
          isActive,
          forceRefresh,
        })
        usageCache.set(key, result)
        setUsage(result)
      } catch (e) {
        console.error(`[IDE Usage] Failed to fetch for ${profileName}:`, e)
        setError(String(e))
      } finally {
        setLoading(false)
      }
    },
    [ideType, profileName, isActive, key],
  )

  // Initial fetch
  useEffect(() => {
    fetchUsage()
  }, [fetchUsage])

  return { usage, loading, error, refresh: () => fetchUsage(true) }
}
