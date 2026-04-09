import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useWebhookConfig } from '@/hooks/use-webhook-config'
import type {
  SessionUsageDetailLevel,
  SessionUsagePeriod,
  SessionUsageSummary,
} from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import {
  BarChart3,
  ChevronDown,
  ChevronRight,
  Send,
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

export function SessionUsageWebhookPanel() {
  const { t } = useTranslation()
  const { config } = useWebhookConfig()

  const [period, setPeriod] = useState<SessionUsagePeriod>('24h')
  const [detailLevel, setDetailLevel] = useState<SessionUsageDetailLevel>('detailed')
  const [sending, setSending] = useState(false)
  const [previewing, setPreviewing] = useState(false)
  const [previewData, setPreviewData] = useState<SessionUsageSummary[] | null>(null)
  const [showPreview, setShowPreview] = useState(false)

  const disabled = !config.enabled || !config.url

  const periodToHours: Record<SessionUsagePeriod, number> = {
    '1h': 1,
    '5h': 5,
    '24h': 24,
    '7d': 168,
  }

  const handleSend = useCallback(async () => {
    setSending(true)
    try {
      const res = await invoke<{ success: boolean; message: string }>(
        'send_session_usage_webhook',
        {
          url: config.url,
          secret: config.secret || null,
          period,
          memberEmail: config.memberEmail || null,
          detailLevel,
        },
      )
      if (res.success) {
        toast.success(t('settings.webhook.session_usage_success', { message: res.message }))
      } else {
        toast.error(t('settings.webhook.session_usage_failed', { error: res.message }))
      }
    } catch (e) {
      toast.error(t('settings.webhook.session_usage_failed', { error: String(e) }))
    } finally {
      setSending(false)
    }
  }, [config, period, detailLevel, t])

  const handlePreview = useCallback(async () => {
    setPreviewing(true)
    try {
      const data = await invoke<SessionUsageSummary[]>('get_session_usage', {
        hoursBack: periodToHours[period],
      })
      setPreviewData(data)
      setShowPreview(true)
    } catch (e) {
      toast.error(String(e))
    } finally {
      setPreviewing(false)
    }
  }, [period])

  return (
    <div className="space-y-4">
      {/* Section header */}
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 px-1">
          {t('settings.webhook.session_usage_title')}
        </h3>
        <div className="rounded-lg border bg-card">
          <div className="px-4 py-3">
            <p className="text-xs text-muted-foreground">
              {t('settings.webhook.session_usage_desc')}
            </p>
          </div>

          <div className="border-t" />

          {/* Period selector */}
          <div className="flex items-center justify-between px-4 py-3">
            <Label className="text-sm">
              {t('settings.webhook.session_usage_period')}
            </Label>
            <Select
              value={period}
              onValueChange={(v) => setPeriod(v as SessionUsagePeriod)}
              disabled={disabled}
            >
              <SelectTrigger className="w-48 h-8 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="1h">{t('settings.webhook.session_usage_period_1h')}</SelectItem>
                <SelectItem value="5h">{t('settings.webhook.session_usage_period_5h')}</SelectItem>
                <SelectItem value="24h">{t('settings.webhook.session_usage_period_24h')}</SelectItem>
                <SelectItem value="7d">{t('settings.webhook.session_usage_period_7d')}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="border-t" />

          {/* Detail level */}
          <div className="flex items-center justify-between px-4 py-3">
            <Label className="text-sm">
              {t('settings.webhook.session_usage_detail')}
            </Label>
            <Select
              value={detailLevel}
              onValueChange={(v) => setDetailLevel(v as SessionUsageDetailLevel)}
              disabled={disabled}
            >
              <SelectTrigger className="w-48 h-8 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="summary">
                  {t('settings.webhook.session_usage_detail_summary')}
                </SelectItem>
                <SelectItem value="detailed">
                  {t('settings.webhook.session_usage_detail_detailed')}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </div>

      {/* Preview toggle */}
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
                        {s.project.replace(/^-home-[^-]+-/, '').split('-').join('/')}
                      </span>
                      {s.branch && (
                        <span className="text-muted-foreground">({s.branch})</span>
                      )}
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

      {/* Send button */}
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={handleSend}
          disabled={disabled || sending}
        >
          <Send className="size-3.5" />
          {sending
            ? t('settings.webhook.session_usage_sending')
            : t('settings.webhook.session_usage_send')}
        </Button>
      </div>
    </div>
  )
}
