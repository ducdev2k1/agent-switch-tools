import { invoke } from "@tauri-apps/api/core"
import { useState, useEffect, useCallback } from "react"
import type { ClaudeCliState } from "@/lib/types"

export function useClaudeConfig() {
  const [state, setState] = useState<ClaudeCliState | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await invoke<ClaudeCliState>("get_claude_cli_state")
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

  const switchProfile = async (envVars: {
    anthropicApiKey?: string
    geminiApiKey?: string
    openaiApiKey?: string
  }) => {
    await invoke("switch_active_profile", {
      envVars: {
        anthropic_api_key: envVars.anthropicApiKey || null,
        gemini_api_key: envVars.geminiApiKey || null,
        openai_api_key: envVars.openaiApiKey || null,
      },
    })
    await refresh()
  }

  return { state, loading, error, refresh, switchProfile }
}
