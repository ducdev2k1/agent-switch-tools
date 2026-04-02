import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import type { ClaudeCliState, UsageStats } from '@/lib/types'
import { Activity, AlertTriangle, FileKey, Monitor } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface CliStatusBarProps {
  cliState: ClaudeCliState | null
  usageStats: UsageStats | null
  loading: boolean
}

export function CliStatusBar({
  cliState,
  usageStats,
  loading,
}: CliStatusBarProps) {
  const { t } = useTranslation()
  if (loading) {
    return (
      <div className="flex items-center gap-4 rounded-lg border bg-card p-4 animate-pulse">
        <div className="h-5 w-32 bg-muted rounded" />
        <div className="h-5 w-24 bg-muted rounded" />
        <div className="h-5 w-20 bg-muted rounded" />
      </div>
    )
  }

  return (
    <div className="flex flex-wrap items-center gap-3 rounded-lg border bg-linear-to-r from-card to-card/80 p-4">
      {/* Model hiện tại */}
      <div className="flex items-center gap-2">
        <Monitor className="size-4 text-muted-foreground" />
        <span className="text-sm text-muted-foreground">
          {t('dashboard.cli.model')}
        </span>
        <Badge
          variant="secondary"
          className="font-mono"
        >
          {cliState?.currentModel || 'N/A'}
        </Badge>
      </div>

      <Separator
        orientation="vertical"
        className="h-5"
      />

      {/* Session count */}
      <div className="flex items-center gap-2">
        <Activity className="size-4 text-muted-foreground" />
        <span className="text-sm text-muted-foreground">
          {t('common.labels.sessions')}
        </span>
        <span className="text-sm font-semibold">
          {usageStats?.totalSessions ?? cliState?.sessionCount ?? 0}
        </span>
        {usageStats && usageStats.recentSessions7d > 0 && (
          <span className="text-xs text-muted-foreground">
            (
            {t('common.labels.this_week', {
              count: usageStats.recentSessions7d,
            })}
            )
          </span>
        )}
      </div>

      <Separator
        orientation="vertical"
        className="h-5"
      />

      {/* .env status */}
      <div className="flex items-center gap-2">
        <FileKey className="size-4 text-muted-foreground" />
        <span className="text-sm text-muted-foreground">
          {t('dashboard.cli.env_status')}
        </span>
        {cliState?.envFileExists ? (
          <Badge
            variant="success"
            className="text-xs"
          >
            {t('dashboard.cli.active_keys', {
              count: cliState.activeKeys.length,
            })}
          </Badge>
        ) : (
          <Badge
            variant="outline"
            className="text-xs"
          >
            {t('common.actions.not_found')}
          </Badge>
        )}
      </div>

      {/* Restriction warning */}
      {usageStats?.hasRestrictions && (
        <>
          <Separator
            orientation="vertical"
            className="h-5"
          />
          <div className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-warning/10 border border-warning/20">
            <AlertTriangle className="size-4 text-warning" />
            <span className="text-xs text-warning font-bold">
              {t('common.labels.restrictions_active')}
            </span>
          </div>
        </>
      )}
    </div>
  )
}
