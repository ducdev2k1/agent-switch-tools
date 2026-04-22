import { isIdeQuotaSupported, type IdeType, type UsageLimits } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useState } from 'react'

/**
 * Hook to fetch and manage quota/usage limits for a specific IDE profile.
 */
export function useIdeUsage(
  ideType: IdeType,
  profileName: string,
  isActive: boolean,
) {
  const [usage, setUsage] = useState<UsageLimits | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const fetchUsage = useCallback(
    async (forceRefresh = false) => {
      if (!isIdeQuotaSupported(ideType)) {
        setUsage(null)
        return
      }
      setLoading(true)
      setError(null)
      try {
        const result = await invoke<UsageLimits | null>('get_ide_usage', {
          ideType,
          profileName,
          isActive,
          forceRefresh,
        })
        setUsage(result)
      } catch (e) {
        console.error(`[IDE Usage] Failed to fetch for ${profileName}:`, e)
        setError(String(e))
      } finally {
        setLoading(false)
      }
    },
    [ideType, profileName, isActive],
  )

  // Initial fetch
  useEffect(() => {
    fetchUsage()
  }, [fetchUsage])

  return { usage, loading, error, refresh: () => fetchUsage(true) }
}
