import type { UsageBucket, UsageLimits } from '@/lib/types'
import { Activity } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface UsageLimitsDisplayProps {
  limits: UsageLimits | null
  loading?: boolean
  compact?: boolean
  unsupported?: boolean
}

/** Normalize utilization: API returns 0-100 (e.g. 36.0 = 36%) */
function normalizePct(util: number): number {
  return Math.min(Math.round(util), 100)
}

/** Bar color based on percentage (0-100). If `remainingBased`, low = red. */
function getBarColor(pct: number, remainingBased = false): string {
  if (remainingBased) {
    if (pct <= 20) return 'bg-destructive'
    if (pct <= 50) return 'bg-orange-400'
    return 'bg-emerald-500'
  }
  if (pct >= 80) return 'bg-destructive'
  if (pct >= 50) return 'bg-orange-400'
  return 'bg-blue-500'
}

function getTextColor(pct: number, remainingBased = false): string {
  if (remainingBased) {
    if (pct <= 20) return 'text-destructive'
    if (pct <= 50) return 'text-orange-400'
    return 'text-emerald-500'
  }
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

/** Format ISO string to local 12h clock (e.g. "3:45 PM") */
function formatResetsAt(resetsAt: string | null): string | null {
  if (!resetsAt) return null
  const d = new Date(resetsAt)
  if (isNaN(d.getTime())) return null
  return d.toLocaleTimeString(undefined, {
    hour: 'numeric',
    minute: '2-digit',
    hour12: true,
  })
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
  const remainingBased = bucket.remainingBased ?? false
  const resetText = formatResetsIn(bucket.resetsAt)
  const absText = formatResetsAt(bucket.resetsAt)

  return (
    <div className="space-y-1">
      <div className="flex items-end justify-between leading-none mb-1">
        <span className="text-[11px] font-semibold text-muted-foreground/80">
          {label}
        </span>
        <span
          className={`text-[11px] font-bold font-mono ${getTextColor(pct, remainingBased)}`}
        >
          {pct}%
        </span>
      </div>
      <div className="h-1.5 bg-muted/40 rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-700 ease-out ${getBarColor(pct, remainingBased)}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      {resetText && (
        <div className="flex justify-end items-center gap-1.5">
          <p className="text-[9px] font-medium text-muted-foreground/50 tabular-nums">
            {(() => {
              const stripped = resetText.replace(/[^0-9hm]/g, '').trim()
              return stripped ? `R: ${stripped}` : resetText
            })()}
          </p>
          {absText && (
            <p className="text-[9px] font-medium text-muted-foreground/40 tabular-nums">
              ({absText})
            </p>
          )}
        </div>
      )}
    </div>
  )
}

export function UsageLimitsDisplay({
  limits,
  loading,
  compact,
  unsupported,
}: UsageLimitsDisplayProps) {
  const { t } = useTranslation()
  const formatResetsIn = useFormatResetsIn()

  if (unsupported) {
    if (compact) return null
    return (
      <div className="mt-4 px-0.5">
        <p className="text-[10px] font-medium text-muted-foreground/60 italic">
          {t('common.labels.usage_unsupported')}
        </p>
      </div>
    )
  }

  if (loading) {
    return (
      <div className="flex flex-col gap-2 mt-4">
        <div className="h-4 w-24 bg-muted/30 rounded animate-pulse" />
        <div className="h-1.5 w-full bg-muted/30 rounded animate-pulse" />
      </div>
    )
  }

  if (!limits) return null

  const dynamicBuckets = limits.buckets?.filter(
    (b) => b.utilization != null,
  ) ?? []
  const hasDynamic = dynamicBuckets.length > 0

  const hasLegacy =
    limits.fiveHour?.utilization != null ||
    limits.sevenDay?.utilization != null ||
    limits.sevenDaySonnet?.utilization != null

  if (!hasDynamic && !hasLegacy) return null

  if (compact) {
    const first = hasDynamic
      ? dynamicBuckets[0]?.utilization
      : limits.fiveHour?.utilization
    const second = hasDynamic
      ? dynamicBuckets[1]?.utilization
      : limits.sevenDay?.utilization
    return (
      <span className="flex items-center gap-2 text-[10px] font-mono">
        <Activity className="size-2.5 text-muted-foreground/60" />
        {first != null && (
          <span className={getTextColor(normalizePct(first))}>
            {normalizePct(first)}%
          </span>
        )}
        {second != null && (
          <span className={getTextColor(normalizePct(second))}>
            {normalizePct(second)}%
          </span>
        )}
      </span>
    )
  }

  // Dynamic bucket mode (Antigravity and future providers)
  if (hasDynamic) {
    return (
      <div className="mt-4 space-y-3 px-0.5">
        {dynamicBuckets.map((b, i) => (
          <UsageRow
            key={b.label ?? i}
            label={b.label ?? `#${i + 1}`}
            bucket={b}
            formatResetsIn={formatResetsIn}
          />
        ))}
      </div>
    )
  }

  return (
    <div className="mt-4 space-y-3 px-0.5">
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
