import { useAutoUpdateConfig } from '@/hooks/use-auto-update-config'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { useCallback, useEffect, useState } from 'react'

export function useAppUpdater() {
  const [updateVersion, setUpdateVersion] = useState<string | null>(null)
  const [installing, setInstalling] = useState(false)
  const { enabled: autoUpdateEnabled } = useAutoUpdateConfig()

  // Check for update on startup only if auto-update is enabled
  useEffect(() => {
    if (!autoUpdateEnabled) return
    check()
      .then((u) => {
        if (u?.available) setUpdateVersion(u.version)
      })
      .catch(() => {})
  }, [autoUpdateEnabled])

  const install = useCallback(async () => {
    setInstalling(true)
    try {
      const u = await check()
      if (u?.available) {
        await u.downloadAndInstall()
        await relaunch()
      }
    } catch {
      setInstalling(false)
    }
  }, [])

  return { updateVersion, installing, install }
}
