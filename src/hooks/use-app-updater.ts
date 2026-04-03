import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { useEffect, useState, useCallback } from 'react'

export function useAppUpdater() {
  const [updateVersion, setUpdateVersion] = useState<string | null>(null)
  const [installing, setInstalling] = useState(false)

  useEffect(() => {
    // Silently check for update on startup; errors are ignored (dev mode, offline, no release yet)
    check()
      .then((u) => {
        if (u?.available) setUpdateVersion(u.version)
      })
      .catch(() => {})
  }, [])

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
