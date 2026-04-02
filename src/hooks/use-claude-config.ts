import { invoke } from '@tauri-apps/api/core'
import { useState, useEffect, useCallback } from 'react'
import type { ClaudeCliState } from '@/lib/types'

export function useClaudeConfig() {
  const [state, setState] = useState<ClaudeCliState | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await invoke<ClaudeCliState>('get_claude_cli_state')
      setState(data)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    refresh()
  }, [refresh])

  return { state, loading, error, refresh }
}
