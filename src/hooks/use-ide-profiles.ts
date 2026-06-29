import type { IdeProfile, IdeSwitchResult, IdeType } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useState } from 'react'

/**
 * Module-level cache of last-known profiles per IDE, so switching tabs (which unmounts
 * the section) shows data instantly and refreshes in the background — no skeleton flash,
 * no layout jump.
 */
const profilesCache = new Map<IdeType, IdeProfile[]>()

export function useIdeProfiles(ideType: IdeType) {
  const [profiles, setProfiles] = useState<IdeProfile[]>(
    () => profilesCache.get(ideType) ?? [],
  )
  // Only show the loading skeleton on the very first load (no cached data yet).
  const [loading, setLoading] = useState(() => !profilesCache.has(ideType))

  const load = useCallback(async () => {
    if (!profilesCache.has(ideType)) setLoading(true)
    try {
      const data = await invoke<IdeProfile[]>('list_ide_profiles', { ideType })
      profilesCache.set(ideType, data)
      setProfiles(data)
    } catch (e) {
      console.error(`[IDE] Failed to load ${ideType} profiles:`, e)
    } finally {
      setLoading(false)
    }
  }, [ideType])

  useEffect(() => {
    load()
  }, [load])

  /** Save current active IDE account as a named profile */
  const saveCurrentAs = async (): Promise<string> => {
    const email = await invoke<string>('save_current_ide_profile', { ideType })
    await load()
    return email
  }

  /** Switch to a different IDE profile */
  const switchTo = async (targetName: string): Promise<IdeSwitchResult> => {
    const result = await invoke<IdeSwitchResult>('switch_ide_profile', {
      ideType,
      targetName,
    })
    await load()
    return result
  }

  /** Rename a saved IDE profile */
  const rename = async (oldName: string, newName: string) => {
    await invoke('rename_ide_profile', { ideType, oldName, newName })
    await load()
  }

  /** Delete a saved IDE profile */
  const remove = async (name: string) => {
    await invoke('delete_ide_profile', { ideType, name })
    await load()
  }

  /** Check if this IDE process is currently running */
  const checkIdeRunning = async (): Promise<boolean> => {
    return invoke<boolean>('is_ide_running', { ideType })
  }

  /** Restart the IDE (kill + relaunch) */
  const restartIde = async (): Promise<string> => {
    return invoke<string>('restart_ide', { ideType })
  }

  return {
    profiles,
    loading,
    saveCurrentAs,
    switchTo,
    rename,
    remove,
    checkIdeRunning,
    restartIde,
    refresh: load,
  }
}
