import type { SessionUsageSummary } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import {
  BarChart3,
  ChevronDown,
  ChevronRight,
} from 'lucide-react'
import { useCallback, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

/** Format large token numbers: 1234567 → "1.2M", 12345 → "12.3K" */
function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return String(n)
}

/** Collapsible preview of recent session token usage (last 24h). */
export function SessionUsageWebhookPanel() {
  const { t } = useTranslation()

  const [previewing, setPreviewing] = useState(false)
  const [previewData, setPreviewData] = useState<SessionUsageSummary[] | null>(null)
  const [showPreview, setShowPreview] = useState(false)

  const handlePreview = useCallback(async () => {
    setPreviewing(true)
    try {
      const data = await invoke<SessionUsageSummary[]>('get_session_usage', {
        hoursBack: 24,
      })
      setPreviewData(data)
      setShowPreview(true)
    } catch (e) {
      toast.error(String(e))
    } finally {
      setPreviewing(false)
    }
  }, [])

  return (
    <div>
      <button
        type="button"
        onClick={handlePreview}
        disabled={previewing}
        className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-muted-foreground hover:text-foreground transition-colors cursor-pointer px-1"
      >
        {showPreview ? (
          <ChevronDown className="size-3.5" />
        ) : (
          <ChevronRight className="size-3.5" />
        )}
        {previewing
          ? t('common.labels.loading')
          : t('settings.webhook.session_usage_preview')}
      </button>
      {showPreview && previewData && (
        <div className="mt-2 rounded-lg border bg-muted/50 p-3 text-xs space-y-2 max-h-60 overflow-y-auto">
          {previewData.length === 0 ? (
            <p className="text-muted-foreground">
              {t('settings.webhook.session_usage_no_data')}
            </p>
          ) : (
            <>
              <p className="text-muted-foreground font-medium">
                {t('settings.webhook.session_usage_sessions_found', {
                  count: previewData.length,
                })}
              </p>
              {previewData.map((s) => (
                <div
                  key={s.sessionId}
                  className="flex items-center justify-between gap-2 py-1 border-b border-border/50 last:border-0"
                >
                  <div className="flex items-center gap-2 min-w-0">
                    <BarChart3 className="size-3 shrink-0 text-muted-foreground" />
                    <span className="truncate font-mono text-[10px]">
                      {s.sessionId.slice(0, 8)}
                    </span>
                  </div>
                  <span className="shrink-0 text-muted-foreground">
                    {t('settings.webhook.session_usage_tokens', {
                      input: formatTokens(s.totalInputTokens),
                      output: formatTokens(s.totalOutputTokens),
                    })}
                  </span>
                </div>
              ))}
            </>
          )}
        </div>
      )}
    </div>
  )
}
