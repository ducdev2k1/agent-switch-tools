import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { UsageLimitsDisplay } from '@/components/usage-limits-display'
import { useProfileUsage, useTokenRefresh } from '@/hooks/use-usage-stats'
import type { CredentialProfile } from '@/lib/types'
import {
  ArrowRightLeft,
  Clock,
  Crown,
  KeyRound,
  RefreshCw,
  Trash2,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

interface ProfileTableProps {
  profiles: CredentialProfile[]
  onSwitch: (profile: CredentialProfile) => void
  onDelete: (name: string) => void
  onProfilesChanged?: () => Promise<void> | void
}

/** Human-readable subscription name */
function formatSubscription(sub: string | null, t: TFunction): string {
  if (!sub) return t('common.actions.unknown')
  return sub.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
}

export function ProfileTable({
  profiles,
  onSwitch,
  onDelete,
  onProfilesChanged,
}: ProfileTableProps) {
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
            <th className="px-4 py-3 text-[11px] font-bold uppercase tracking-wider text-muted-foreground w-[35%]">
              {t('common.labels.model_quota')}
            </th>
            <th className="px-4 py-3 text-[11px] font-bold uppercase tracking-wider text-muted-foreground w-[15%] text-right">
              {t('common.labels.expires_at')}
            </th>
            <th className="px-4 py-3 text-[11px] font-bold uppercase tracking-wider text-muted-foreground w-[10%] text-right">
              {t('common.labels.actions')}
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-border/30">
          {profiles.map((profile) => (
            <ProfileRow
              key={profile.name}
              profile={profile}
              onSwitch={onSwitch}
              onDelete={onDelete}
              onProfilesChanged={onProfilesChanged}
            />
          ))}
        </tbody>
      </table>
    </div>
  )
}

function ProfileRow({
  profile,
  onSwitch,
  onDelete,
  onProfilesChanged,
}: {
  profile: CredentialProfile
  onSwitch: (profile: CredentialProfile) => void
  onDelete: (name: string) => void
  onProfilesChanged?: () => Promise<void> | void
}) {
  const { t } = useTranslation()
  const { name, isActive, info } = profile
  const {
    limits: usageLimits,
    loading: usageLoading,
    refresh: refreshUsage,
  } = useProfileUsage(name, isActive)
  const { refreshToken, refreshing } = useTokenRefresh()
  const expired = info.isExpired

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
              {profile.oauthAccount?.emailAddress || name}
            </span>
            {isActive && (
              <Badge
                variant="success"
                className="h-4 px-1.5 text-[8px] font-bold uppercase tracking-tighter"
              >
                {t('common.labels.active')}
              </Badge>
            )}
            {expired && (
              <Badge
                variant="destructive"
                className="h-4 px-1.5 text-[8px] font-bold uppercase tracking-tighter"
              >
                {t('common.labels.expired')}
              </Badge>
            )}
          </div>
        </div>
      </td>

      {/* Membership Column */}
      <td className="px-4 py-3 align-top">
        {info.subscriptionType && (
          <Badge
            variant="secondary"
            className="h-5 px-2 text-[10px] font-semibold bg-secondary/50"
          >
            <Crown className="size-3 mr-1.5 text-primary/70" />
            {formatSubscription(info.subscriptionType, t)}
          </Badge>
        )}
      </td>

      {/* Model Quota */}
      <td className="px-4 py-3">
        <div className="max-w-[320px] -mt-4">
          <UsageLimitsDisplay
            limits={usageLimits}
            loading={usageLoading}
          />
        </div>
      </td>

      {/* Expires At */}
      <td className="px-4 py-3 align-top text-right">
        <div className="text-[10px] font-medium text-muted-foreground/50 tabular-nums flex items-center justify-end gap-1.5">
          <Clock className="size-3" />
          {info.expiresAt
            ? new Date(info.expiresAt).toLocaleDateString()
            : '2025/12/15'}
        </div>
      </td>

      {/* Actions */}
      <td className="px-4 py-3 align-top text-right">
        <div className="flex items-center justify-end gap-0.5">
          {expired && (
            <Button
              variant="ghost"
              size="icon"
              className="size-8 text-primary hover:text-primary-foreground hover:bg-primary"
              onClick={async (e) => {
                e.stopPropagation()
                const result = await refreshToken(name, isActive)
                if (result.success) {
                  toast.success(t('common.messages.token_refreshed'))
                  await onProfilesChanged?.()
                  refreshUsage()
                } else {
                  toast.error(
                    t('common.messages.token_refresh_failed', {
                      message: result.message,
                    }),
                  )
                }
              }}
              disabled={refreshing}
            >
              <KeyRound
                className={`size-4 ${refreshing ? 'animate-spin' : ''}`}
              />
            </Button>
          )}

          <Button
            variant="ghost"
            size="icon"
            className="size-8 text-muted-foreground/60 hover:text-foreground opacity-40 group-hover:opacity-100 transition-opacity"
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
              className="size-8 text-muted-foreground/60 hover:text-primary opacity-40 group-hover:opacity-100 transition-opacity"
              onClick={() => onSwitch(profile)}
            >
              <ArrowRightLeft className="size-4" />
            </Button>
          )}

          <Button
            variant="ghost"
            size="icon"
            className="size-8 text-muted-foreground/60 hover:text-destructive opacity-40 group-hover:opacity-100 transition-opacity"
            onClick={() => onDelete(name)}
          >
            <Trash2 className="size-4" />
          </Button>
        </div>
      </td>
    </tr>
  )
}
