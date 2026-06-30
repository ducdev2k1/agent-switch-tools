import { useAutoUpdateConfig } from '@/hooks/use-auto-update-config'
import { settingsStore } from '@/lib/settings-store'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { useCallback, useEffect, useState } from 'react'

const DISMISSED_VERSION_KEY = 'update_dismissed_version'

export function useAppUpdater() {
  const [updateVersion, setUpdateVersion] = useState<string | null>(null)
  const [updateBody, setUpdateBody] = useState<string | null>(null)
  const [showModal, setShowModal] = useState(false)
  const [installing, setInstalling] = useState(false)
  const [checking, setChecking] = useState(false)
  const { enabled: autoUpdateEnabled } = useAutoUpdateConfig()

  // Auto-check on startup — show modal once per version
  useEffect(() => {
    if (!autoUpdateEnabled) return

    check()
      .then(async (u) => {
        if (!u?.available) return
        setUpdateVersion(u.version)
        setUpdateBody(u.body ?? null)

        // Only show modal if this version wasn't dismissed before
        const dismissed = await settingsStore.get<string>(DISMISSED_VERSION_KEY)
        if (dismissed !== u.version) {
          setShowModal(true)
        }
      })
      .catch(() => {})
  }, [autoUpdateEnabled])

  // Manual check (from Settings page)
  const checkForUpdates = useCallback(async () => {
    setChecking(true)
    try {
      const u = await check()
      if (u?.available) {
        setUpdateVersion(u.version)
        setUpdateBody(u.body ?? null)
        return u.version
      }
      return null
    } catch {
      return null
    } finally {
      setChecking(false)
    }
  }, [])

  // Dismiss modal — remember this version so it won't auto-show again
  const dismissModal = useCallback(async () => {
    setShowModal(false)
    if (updateVersion) {
      await settingsStore.set(DISMISSED_VERSION_KEY, updateVersion)
      await settingsStore.save()
    }
  }, [updateVersion])

  // Install update and relaunch
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

  return {
    updateVersion,
    updateBody,
    showModal,
    dismissModal,
    installing,
    install,
    checking,
    checkForUpdates,
  }
}
