import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { UsageLimitsDisplay } from '@/components/usage-limits-display'
import { useIdeUsage } from '@/hooks/use-ide-usage'
import { isIdeQuotaSupported, type IdeProfile } from '@/lib/types'
import { ArrowRightLeft, Clock, Crown, RefreshCw, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface IdeProfileTableProps {
  profiles: IdeProfile[]
  onSwitch: (profile: IdeProfile) => void
  onDelete: (name: string) => void
}

export function IdeProfileTable({
  profiles,
  onSwitch,
  onDelete,
}: IdeProfileTableProps) {
  const { t } = useTranslation()

  return (
    <div className="w-full border border-border/40 rounded-lg overflow-hidden bg-card/30">
      <table className="w-full text-left border-collapse">
        <thead>
          <tr className="border-b border-border/40 bg-muted/30">
            <th className="px-4 py-3 text-[11px] font-bold uppercase tracking-wider text-muted-foreground w-[25%]">
              {t('common.labels.email')}
            </th>
            <th className="px-4 py-3 text-[11px] font-bold uppercase tracking-wider text-muted-foreground w-[15%]">
              {t('common.labels.membership')}
            </th>
            <th className="px-4 py-3 text-[11px] font-bold uppercase tracking-wider text-muted-foreground w-[40%]">
              {t('common.labels.model_quota')}
            </th>
            <th className="px-4 py-3 text-[11px] font-bold uppercase tracking-wider text-muted-foreground w-[10%] text-right">
              {t('common.labels.status')}
            </th>
            <th className="px-4 py-3 text-[11px] font-bold uppercase tracking-wider text-muted-foreground w-[10%] text-right">
              {t('common.labels.actions')}
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-border/30">
          {profiles.map((profile) => (
            <IdeProfileRow
              key={profile.name}
              profile={profile}
              onSwitch={onSwitch}
              onDelete={onDelete}
            />
          ))}
        </tbody>
      </table>
    </div>
  )
}

function IdeProfileRow({
  profile,
  onSwitch,
  onDelete,
}: {
  profile: IdeProfile
  onSwitch: (profile: IdeProfile) => void
  onDelete: (name: string) => void
}) {
  const { t } = useTranslation()
  const { name, isActive, email, displayName, ideType, membershipType } =
    profile
  const {
    usage,
    loading: usageLoading,
    refresh: refreshUsage,
  } = useIdeUsage(ideType, name, isActive)

  return (
    <tr
      className={`group transition-colors hover:bg-muted/20 ${isActive ? 'bg-success/5' : ''}`}
    >
      {/* Email / Status */}
      <td className="px-4 py-3 align-top">
        <div className="flex items-center gap-3">
          <div className="size-4 rounded border border-muted-foreground/30 shrink-0 flex items-center justify-center bg-muted/20">
            {isActive && <div className="size-2 rounded-sm bg-success/80" />}
          </div>
          <div className="flex items-center gap-2 min-w-0">
            <span className="text-sm font-bold truncate text-foreground/90">
              {email || displayName || name}
            </span>
            {isActive && (
              <Badge
                variant="success"
                className="h-4 px-1.5 text-[8px] font-bold uppercase tracking-tighter"
              >
                {t('common.labels.active')}
              </Badge>
            )}
          </div>
        </div>
      </td>

      {/* Membership Column */}
      <td className="px-4 py-3 align-top">
        {membershipType && (
          <Badge
            variant="secondary"
            className="h-5 px-2 text-[10px] font-semibold bg-secondary/50"
          >
            <Crown className="size-3 mr-1.5 text-primary/70" />
            {membershipType.replace(/\b\w/g, (c) => c.toUpperCase())}
          </Badge>
        )}
      </td>

      {/* Model Quota */}
      <td className="px-4 py-3">
        <div className="max-w-[320px] -mt-4">
          <UsageLimitsDisplay
            limits={usage}
            loading={usageLoading}
            unsupported={!isIdeQuotaSupported(ideType)}
          />
        </div>
      </td>

      {/* Status */}
      <td className="px-4 py-3 align-top text-right">
        <div className="text-[10px] font-medium text-muted-foreground/50 tabular-nums flex items-center justify-end gap-1.5">
          <Clock className="size-3" />
          {isActive ? new Date().toLocaleDateString() : '2025/12/15'}
        </div>
      </td>

      {/* Actions */}
      <td className="px-4 py-3 align-top text-right">
        <div className="flex items-center justify-end gap-0.5 opacity-40 group-hover:opacity-100 transition-opacity">
          <Button
            variant="ghost"
            size="icon"
            className="size-8 text-muted-foreground/60 hover:text-foreground"
            title={t('common.actions.refresh')}
            onClick={(e) => {
              e.stopPropagation()
              refreshUsage()
            }}
            disabled={usageLoading}
          >
            <RefreshCw
              className={`size-4 ${usageLoading ? 'animate-spin' : ''}`}
            />
          </Button>

          {!isActive && (
            <Button
              variant="ghost"
              size="icon"
              className="size-8 text-muted-foreground/60 hover:text-primary"
              title={t('common.actions.switch')}
              onClick={() => onSwitch(profile)}
            >
              <ArrowRightLeft className="size-4" />
            </Button>
          )}

          <Button
            variant="ghost"
            size="icon"
            className="size-8 text-muted-foreground/60 hover:text-destructive"
            title={t('common.actions.delete')}
            onClick={() => onDelete(name)}
          >
            <Trash2 className="size-4" />
          </Button>
        </div>
      </td>
    </tr>
  )
}
