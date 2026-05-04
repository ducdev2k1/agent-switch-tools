import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useState, useEffect, useCallback, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import type { CredentialProfile, SwitchResult } from '@/lib/types'

export function useCredentialProfiles() {
  const { t } = useTranslation()
  const [profiles, setProfiles] = useState<CredentialProfile[]>([])
  const [loading, setLoading] = useState(false)
  const lastFocusRefreshRef = useRef(0)

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const data = await invoke<CredentialProfile[]>('list_credential_profiles')
      setProfiles(data)
    } catch (e) {
      console.error('Failed to load credential profiles:', e)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    load()
  }, [load])

  // Refresh when window regains focus — covers the case where the user runs
  // `claude /login` outside the app and switches back. Backend reconcile will
  // detect the drift and emit `claude-profile-drift-detected`.
  useEffect(() => {
    const onFocus = () => {
      const now = Date.now()
      if (now - lastFocusRefreshRef.current < 1000) return
      lastFocusRefreshRef.current = now
      load()
    }
    window.addEventListener('focus', onFocus)
    return () => window.removeEventListener('focus', onFocus)
  }, [load])

  // Listen for backend drift detection. Triggered when the user logs into a
  // different Claude account outside the app — the previous profile is auto-saved
  // and the new one becomes active.
  useEffect(() => {
    let unlisten: UnlistenFn | null = null
    listen('claude-profile-drift-detected', () => {
      toast.info(t('profiles.driftDetected'))
      load()
    })
      .then((fn) => {
        unlisten = fn
      })
      .catch((e) => console.error('Failed to listen drift event:', e))
    return () => {
      unlisten?.()
    }
  }, [load, t])

  /** Save current active credentials (auto-detect email from oauthAccount) */
  const saveCurrentAs = async (): Promise<string> => {
    const email = await invoke<string>('save_current_as_profile')
    await load()
    return email
  }

  /** Switch to target profile (backend handles backup of current) */
  const switchTo = async (targetName: string): Promise<SwitchResult> => {
    const result = await invoke<SwitchResult>('switch_credential_profile', {
      targetName,
    })
    await load()
    return result
  }

  /** Rename a saved profile */
  const rename = async (oldName: string, newName: string) => {
    await invoke('rename_credential_profile', { oldName, newName })
    await load()
  }

  /** Delete a saved profile */
  const remove = async (name: string) => {
    await invoke('delete_credential_profile', { name })
    await load()
  }

  /** Check if Claude Code CLI is currently running */
  const checkClaudeRunning = async (): Promise<boolean> => {
    return invoke<boolean>('is_claude_running')
  }

  return {
    profiles,
    loading,
    saveCurrentAs,
    switchTo,
    rename,
    remove,
    checkClaudeRunning,
    refresh: load,
  }
}
