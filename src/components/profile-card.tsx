import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { UsageLimitsDisplay } from '@/components/usage-limits-display'
import { useProfileUsage } from '@/hooks/use-usage-stats'
import type { CredentialProfile } from '@/lib/types'
import {
  ArrowRightLeft,
  Building2,
  Crown,
  Mail,
  RefreshCw,
  Trash2,
  Zap,
} from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface ProfileCardProps {
  profile: CredentialProfile
  onSwitch: (profile: CredentialProfile) => void
  onDelete: (name: string) => void
  isCurrentlyActive?: boolean
}

/** Human-readable subscription name */
function formatSubscription(sub: string | null, t: any): string {
  if (!sub) return t('common.actions.unknown')
  return sub.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
}

export function ProfileCard({
  profile,
  onSwitch,
  onDelete,
  isCurrentlyActive = false,
}: ProfileCardProps) {
  const { t } = useTranslation()
  const { name, isActive, info } = profile
  const {
    limits: usageLimits,
    loading: usageLoading,
    refresh: refreshUsage,
  } = useProfileUsage(name, isActive)
  const expired = info.isExpired

  // Health color indicator logic
  const expiringSoon = false

  return (
    <Card
      className={`transition-all duration-300 hover:shadow-lg group ${
        isActive
          ? 'border-success/50 bg-success/5 shadow-[0_0_15px_rgba(34,197,94,0.1)] dark:shadow-[0_0_15px_rgba(34,197,94,0.05)]'
          : isCurrentlyActive
            ? 'border-primary/30 bg-accent/30'
            : expired
              ? 'border-destructive/30 bg-destructive/5'
              : expiringSoon
                ? 'border-warning/30 bg-warning/5'
                : 'hover:border-primary/50'
      }`}
    >
      <CardContent className="p-4">
        <div className="flex items-start gap-4">
          {/* Left: indicator */}
          <div className="mt-1.5 shrink-0">
            <div
              className={`size-3.5 rounded-full transition-all duration-300 ${
                isActive
                  ? 'bg-success shadow-[0_0_10px_rgba(34,197,94,0.8)] animate-pulse'
                  : isCurrentlyActive
                    ? 'bg-primary/50'
                    : expired
                      ? 'bg-destructive/60'
                      : expiringSoon
                        ? 'bg-warning/80'
                        : 'bg-muted-foreground/30'
              }`}
            />
          </div>

          {/* Right Content */}
          <div className="min-w-0 flex-1">
            {/* Top row: Info & Actions */}
            <div className="flex items-start justify-between gap-4">
              {/* Info */}
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2 flex-wrap">
                  <h3 className="font-bold text-base tracking-tight group-hover:text-primary transition-colors">
                    {profile.oauthAccount?.displayName || name}
                  </h3>
                  {isActive && (
                    <Badge
                      variant="success"
                      className="text-[10px] font-bold uppercase tracking-wider px-2 py-0"
                    >
                      {t('common.labels.active')}
                    </Badge>
                  )}
                  {isCurrentlyActive && !isActive && (
                    <Badge
                      variant="secondary"
                      className="text-[10px] font-bold uppercase tracking-wider px-2 py-0"
                    >
                      {t('common.labels.matched')}
                    </Badge>
                  )}
                  {expired && (
                    <Badge
                      variant="destructive"
                      className="text-[10px] font-bold uppercase tracking-wider px-2 py-0"
                    >
                      {t('common.labels.expired')}
                    </Badge>
                  )}
                </div>

                {/* Email + Org */}
                {profile.oauthAccount?.emailAddress && (
                  <div className="flex items-center gap-3 mt-1.5 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Mail className="size-3" />
                      {profile.oauthAccount.emailAddress}
                    </span>
                    {profile.oauthAccount.organizationName && (
                      <span className="flex items-center gap-1">
                        <Building2 className="size-3" />
                        {profile.oauthAccount.organizationName}
                      </span>
                    )}
                  </div>
                )}

                {/* Extras indicators */}
                <div className="flex items-center gap-1.5 mt-2.5 flex-wrap">
                  {info.subscriptionType && (
                    <Badge
                      variant="secondary"
                      className="text-[10px] bg-secondary/50 font-medium"
                    >
                      <Crown className="size-3 mr-1 text-primary/70" />
                      {formatSubscription(info.subscriptionType, t)}
                    </Badge>
                  )}
                  {info.rateLimitTier && (
                    <Badge
                      variant="outline"
                      className="text-[10px] font-mono font-medium border-primary/20"
                    >
                      <Zap className="size-3 mr-1 text-primary/70" />
                      {info.rateLimitTier}
                    </Badge>
                  )}
                </div>

                {/* Scopes summary */}
                {info.scopes.length > 0 && (
                  <p className="text-[11px] text-muted-foreground mt-2 font-medium flex items-center gap-1">
                    <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/30" />
                    {t('common.labels.scopes', { count: info.scopes.length })}
                    <span className="text-muted-foreground/40 mx-1">|</span>
                    {info.organizationUuid ||
                    profile.oauthAccount?.organizationUuid ||
                    profile.oauthAccount?.organizationName
                      ? t('common.labels.organization_account')
                      : t('common.labels.personal_account')}
                  </p>
                )}
              </div>

              {/* Actions */}
              {!isActive && (
                <div className="flex items-center gap-1.5 shrink-0 opacity-80 group-hover:opacity-100 transition-opacity">
                  {!isCurrentlyActive && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => onSwitch(profile)}
                      className="h-8 text-xs font-bold hover:bg-primary hover:text-primary-foreground border-primary/20"
                    >
                      <ArrowRightLeft className="size-3.5 mr-1" />
                      {t('common.actions.switch')}
                    </Button>
                  )}
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => onDelete(name)}
                    className="size-8 text-destructive/70 hover:text-destructive hover:bg-destructive/10"
                  >
                    <Trash2 className="size-3.5" />
                  </Button>
                </div>
              )}
            </div>

            {/* Bottom row: Usage limits */}
            <div className="relative mt-2">
              <button
                onClick={(e) => {
                  e.stopPropagation()
                  refreshUsage()
                }}
                disabled={usageLoading}
                title={t('common.actions.refresh')}
                className="absolute right-0 top-2 p-0.5 text-muted-foreground/40 hover:text-muted-foreground disabled:opacity-30 transition-colors z-10 cursor-pointer"
              >
                <RefreshCw
                  className={`size-3 ${usageLoading ? 'animate-spin' : ''}`}
                />
              </button>
              <UsageLimitsDisplay
                limits={usageLimits}
                loading={usageLoading}
              />
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
