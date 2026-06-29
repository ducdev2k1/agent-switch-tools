import type { SessionUsageSummary } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import {
  BarChart3,
  ChevronDown,
  ChevronRight,
  GitBranch,
  MessageSquare,
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

/**
 * Claude Code encodes a project path into a folder name by replacing path
 * separators with "-" (e.g. "-home-ducnd-my-project-claude-tools").
 * Keep the last two segments so the readable project name shows up.
 */
function shortenProject(raw: string): string {
  const parts = raw.split('-').filter(Boolean)
  if (parts.length <= 2) return parts.join('-') || raw
  return parts.slice(-2).join('-')
}

/** Drop the "claude-" prefix so the model reads e.g. "opus-4-8". */
function shortenModel(model: string): string {
  return model.replace(/^claude-/, '')
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
                  className="flex flex-col gap-1 py-1.5 border-b border-border/50 last:border-0"
                >
                  <div className="flex items-center justify-between gap-2">
                    <div className="flex items-center gap-1.5 min-w-0">
                      <BarChart3 className="size-3 shrink-0 text-muted-foreground" />
                      <span
                        className="truncate font-medium"
                        title={s.project}
                      >
                        {shortenProject(s.project)}
                      </span>
                    </div>
                    <span className="shrink-0 text-muted-foreground">
                      {t('settings.webhook.session_usage_tokens', {
                        input: formatTokens(s.totalInputTokens),
                        output: formatTokens(s.totalOutputTokens),
                      })}
                    </span>
                  </div>
                  <div className="flex items-center gap-2 flex-wrap pl-4.5 text-[10px] text-muted-foreground">
                    {s.model && (
                      <span className="font-mono">{shortenModel(s.model)}</span>
                    )}
                    <span className="flex items-center gap-1">
                      <GitBranch className="size-2.5" />
                      {s.branch || t('settings.webhook.session_usage_no_branch')}
                    </span>
                    <span className="flex items-center gap-1">
                      <MessageSquare className="size-2.5" />
                      {t('settings.webhook.session_usage_messages', {
                        count: s.messageCount,
                      })}
                    </span>
                    <span
                      className="font-mono opacity-60"
                      title={s.sessionId}
                    >
                      {s.sessionId.slice(0, 8)}
                    </span>
                  </div>
                </div>
              ))}
            </>
          )}
        </div>
      )}
    </div>
  )
}
