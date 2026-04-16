import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useState } from 'react'
import type { IdeInfo } from '@/lib/types'

export function useInstalledIdes() {
  const [ides, setIdes] = useState<IdeInfo[]>([])
  const [loading, setLoading] = useState(true)

  const load = useCallback(async () => {
    try {
      const data = await invoke<IdeInfo[]>('list_installed_ides')
      console.log('[IDE] Installed IDEs:', JSON.stringify(data))
      setIdes(data)
    } catch (e) {
      console.error('[IDE] Failed to list installed IDEs:', e)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    load()
  }, [load])

  return { ides, loading, refresh: load }
}
