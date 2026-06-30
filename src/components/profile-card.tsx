import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { UsageLimitsDisplay } from '@/components/usage-limits-display'
import { useProfileUsage, useTokenRefresh } from '@/hooks/use-usage-stats'
import type { CredentialProfile } from '@/lib/types'
import {
  ArrowRightLeft,
  Building2,
  Clock,
  Crown,
  KeyRound,
  RefreshCw,
  ShieldCheck,
  Trash2,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

interface ProfileCardProps {
  profile: CredentialProfile
  onSwitch: (profile: CredentialProfile) => void
  onDelete: (name: string) => void
  onProfilesChanged?: () => Promise<void> | void
}

/** Human-readable subscription name */
function formatSubscription(sub: string | null, t: TFunction): string {
  if (!sub) return t('common.actions.unknown')
  return sub.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
}

export function ProfileCard({
  profile,
  onSwitch,
  onDelete,
  onProfilesChanged,
}: ProfileCardProps) {
  const { t } = useTranslation()
  const { name, isActive, info } = profile
  const acc = profile.oauthAccount
  const title = acc?.displayName || acc?.emailAddress || name
  // Show the email separately only when the display name occupies the title.
  const subParts = [
    acc?.displayName ? acc?.emailAddress : null,
    acc?.organizationName,
  ].filter(Boolean)
  const {
    limits: usageLimits,
    loading: usageLoading,
    refresh: refreshUsage,
  } = useProfileUsage(name, isActive)
  const { refreshToken, refreshing } = useTokenRefresh()
  const expired = info.isExpired

  return (
    <Card
      className={`relative transition-all duration-300 hover:shadow-md group overflow-hidden border-border/40 ${
        isActive
          ? 'bg-success/5 border-success/30 shadow-[0_4px_20px_rgba(34,197,94,0.08)]'
          : 'bg-card/50 hover:bg-card hover:border-primary/30'
      }`}
    >
      {/* Active side indicator */}
      {isActive && (
        <div className="absolute left-0 top-0 bottom-0 w-1 bg-success shadow-[0_0_10px_rgba(34,197,94,1)]" />
      )}

      <CardContent className="p-4 flex flex-col h-full">
        {/* Top Section: Identity */}
        <div className="flex items-start justify-between gap-2 mb-3">
          <div className="flex items-start gap-2.5 min-w-0">
            {/* Decorative Checkbox */}
            <div className="size-4 mt-0.5 rounded border border-muted-foreground/30 shrink-0 flex items-center justify-center bg-muted/20">
              {isActive && <div className="size-2 rounded-sm bg-success/80" />}
            </div>

            <div className="flex flex-col min-w-0">
              <span className="text-sm font-bold truncate text-foreground/90 group-hover:text-primary transition-colors">
                {title}
              </span>

              {subParts.length > 0 && (
                <span className="text-[11px] text-muted-foreground/80 truncate">
                  {subParts.join(' · ')}
                </span>
              )}

              <div className="flex items-center gap-1.5 mt-1.5 flex-wrap">
                {info.subscriptionType && (
                  <Badge
                    variant="secondary"
                    className="h-4 px-1.5 text-[8px] font-bold bg-secondary/50"
                  >
                    <Crown className="size-2 mr-1 text-primary/70" />
                    {formatSubscription(info.subscriptionType, t)}
                  </Badge>
                )}
                {info.rateLimitTier && (
                  <Badge
                    variant="outline"
                    className="h-4 px-1.5 text-[8px] font-mono lowercase tracking-tight"
                  >
                    {info.rateLimitTier}
                  </Badge>
                )}
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

              {/* Scopes + account type */}
              <div className="flex items-center gap-2.5 mt-1.5 text-[10px] text-muted-foreground/70 flex-wrap">
                {info.scopes?.length > 0 && (
                  <span className="flex items-center gap-1">
                    <ShieldCheck className="size-2.5" />
                    {t('common.labels.scopes', { count: info.scopes.length })}
                  </span>
                )}
                <span className="flex items-center gap-1">
                  <Building2 className="size-2.5" />
                  {info.organizationUuid
                    ? t('common.labels.organization_account')
                    : t('common.labels.personal_account')}
                </span>
              </div>
            </div>
          </div>

          <div className="shrink-0 flex items-center gap-1 mt-0.5">
            {expired && (
              <Button
                variant="ghost"
                size="icon"
                className="size-7 text-primary hover:text-primary-foreground hover:bg-primary"
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
                  className={`size-3.5 ${refreshing ? 'animate-spin' : ''}`}
                />
              </Button>
            )}
          </div>
        </div>

        {/* Middle Section: Usage */}
        <div className="flex-1">
          <UsageLimitsDisplay
            limits={usageLimits}
            loading={usageLoading}
          />
        </div>

        {/* Bottom Section: Meta & Actions */}
        <div className="mt-5 flex items-center justify-between pt-3 border-t border-border/30">
          <div className="flex flex-col gap-0.5">
            <div className="text-[9px] font-medium text-muted-foreground/40 tabular-nums flex items-center gap-1">
              <Clock className="size-2.5" />
              {info.expiresAt
                ? new Date(info.expiresAt).toLocaleString()
                : '2025/12/15 16:30'}
            </div>
          </div>

          <div className="flex items-center gap-0.5">
            <Button
              variant="ghost"
              size="icon"
              className="size-8 text-muted-foreground/60 hover:text-foreground"
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
                onClick={() => onSwitch(profile)}
              >
                <ArrowRightLeft className="size-4" />
              </Button>
            )}

            <Button
              variant="ghost"
              size="icon"
              className="size-8 text-muted-foreground/60 hover:text-destructive"
              onClick={() => onDelete(name)}
            >
              <Trash2 className="size-4" />
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
