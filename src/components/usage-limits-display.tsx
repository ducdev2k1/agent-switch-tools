import type { UsageLimits, UsageBucket } from '@/lib/types'
import { Activity } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface UsageLimitsDisplayProps {
  limits: UsageLimits | null
  loading?: boolean
  compact?: boolean
}

/** Normalize utilization: API returns 0-100 (e.g. 36.0 = 36%) */
function normalizePct(util: number): number {
  return Math.min(Math.round(util), 100)
}

/** Bar color based on percentage (0-100) */
function getBarColor(pct: number): string {
  if (pct >= 80) return 'bg-destructive'
  if (pct >= 50) return 'bg-orange-400'
  return 'bg-blue-500'
}

function getTextColor(pct: number): string {
  if (pct >= 80) return 'text-destructive'
  if (pct >= 50) return 'text-orange-400'
  return 'text-blue-500'
}

/** Format resets_at ISO string to localized relative time */
function useFormatResetsIn() {
  const { t } = useTranslation()
  return (resetsAt: string | null): string | null => {
    if (!resetsAt) return null
    const diff = new Date(resetsAt).getTime() - Date.now()
    if (diff <= 0) return t('common.labels.usage_resetting')
    const hours = Math.floor(diff / 3600000)
    const minutes = Math.floor((diff % 3600000) / 60000)
    if (hours > 0)
      return t('common.labels.usage_resets_in_hours', { count: hours })
    return t('common.labels.usage_resets_in_minutes', { count: minutes })
  }
}

/** Single usage row */
function UsageRow({
  label,
  bucket,
  formatResetsIn,
}: {
  label: string
  bucket: UsageBucket | null
  formatResetsIn: (resetsAt: string | null) => string | null
}) {
  if (!bucket || bucket.utilization === null) return null
  const pct = normalizePct(bucket.utilization)
  const resetText = formatResetsIn(bucket.resetsAt)

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium text-foreground">{label}</span>
        <span
          className={`text-xs font-bold font-mono ${getTextColor(pct)}`}
        >
          {pct}%
        </span>
      </div>
      <div className="h-2 bg-muted rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-500 ${getBarColor(pct)}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      {resetText && (
        <p className="text-[10px] text-muted-foreground">{resetText}</p>
      )}
    </div>
  )
}

export function UsageLimitsDisplay({
  limits,
  loading,
  compact,
}: UsageLimitsDisplayProps) {
  const { t } = useTranslation()
  const formatResetsIn = useFormatResetsIn()

  if (loading) {
    return (
      <div className="text-[11px] text-muted-foreground animate-pulse mt-2">
        {t('common.labels.usage_loading')}
      </div>
    )
  }

  if (!limits) return null

  const hasAny =
    limits.fiveHour?.utilization != null ||
    limits.sevenDay?.utilization != null ||
    limits.sevenDaySonnet?.utilization != null

  if (!hasAny) return null

  if (compact) {
    const fh = limits.fiveHour?.utilization
    const sd = limits.sevenDay?.utilization
    return (
      <span className="flex items-center gap-1.5 text-[11px] font-mono">
        <Activity className="size-3 text-muted-foreground" />
        {fh != null && (
          <span className={getTextColor(normalizePct(fh))}>
            5h:{normalizePct(fh)}%
          </span>
        )}
        {sd != null && (
          <span className={getTextColor(normalizePct(sd))}>
            7d:{normalizePct(sd)}%
          </span>
        )}
      </span>
    )
  }

  return (
    <div className="mt-3 space-y-3">
      <p className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
        {t('common.labels.usage')}
      </p>
      <UsageRow
        label={t('common.labels.usage_session')}
        bucket={limits.fiveHour}
        formatResetsIn={formatResetsIn}
      />
      <UsageRow
        label={t('common.labels.usage_weekly')}
        bucket={limits.sevenDay}
        formatResetsIn={formatResetsIn}
      />
      <UsageRow
        label={t('common.labels.usage_weekly_sonnet')}
        bucket={limits.sevenDaySonnet}
        formatResetsIn={formatResetsIn}
      />
    </div>
  )
}
