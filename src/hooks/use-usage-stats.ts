import { invoke } from "@tauri-apps/api/core"
import { useState, useEffect } from "react"
import type { UsageStats } from "@/lib/types"

export function useUsageStats() {
  const [stats, setStats] = useState<UsageStats | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    setLoading(true)
    invoke<UsageStats>("get_usage_stats")
      .then(setStats)
      .catch((e) => console.error("Failed to load usage stats:", e))
      .finally(() => setLoading(false))
  }, [])

  return { stats, loading }
}
