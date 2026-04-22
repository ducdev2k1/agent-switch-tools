import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { UsageLimitsDisplay } from '@/components/usage-limits-display'
import { useIdeUsage } from '@/hooks/use-ide-usage'
import { isIdeQuotaSupported, type IdeProfile } from '@/lib/types'
import { ArrowRightLeft, Clock, Crown, RefreshCw, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface IdeProfileCardProps {
  profile: IdeProfile
  onSwitch: (profile: IdeProfile) => void
  onDelete: (name: string) => void
}

export function IdeProfileCard({
  profile,
  onSwitch,
  onDelete,
}: IdeProfileCardProps) {
  const { t } = useTranslation()
  const { name, isActive, email, displayName, ideType, membershipType } =
    profile
  const {
    usage,
    loading: usageLoading,
    refresh: refreshUsage,
  } = useIdeUsage(ideType, name, isActive)

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
        {/* Top Section: Checkbox & Email */}
        <div className="flex items-start justify-between gap-2 mb-3">
          <div className="flex items-center gap-2.5 min-w-0">
            {/* Decorative Checkbox */}
            <div className="size-4 rounded border border-muted-foreground/30 shrink-0 flex items-center justify-center bg-muted/20">
              {isActive && <div className="size-2 rounded-sm bg-success/80" />}
            </div>

            <div className="flex flex-col min-w-0">
              <span className="text-sm font-bold truncate text-foreground/90 group-hover:text-primary transition-colors">
                {email || displayName || name}
              </span>

              <div className="flex items-center gap-1.5 mt-1 flex-wrap">
                {membershipType && (
                  <Badge
                    variant="secondary"
                    className="h-4 px-1.5 text-[8px] font-bold bg-secondary/50"
                  >
                    <Crown className="size-2 mr-1 text-primary/70" />
                    {membershipType.replace(/\b\w/g, (c) => c.toUpperCase())}
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
              </div>
            </div>
          </div>
        </div>

        {/* Middle Section: Usage */}
        <div className="flex-1">
          <UsageLimitsDisplay
            limits={usage}
            loading={usageLoading}
            unsupported={!isIdeQuotaSupported(ideType)}
          />
        </div>

        {/* Bottom Section: Meta & Actions */}
        <div className="mt-5 flex items-center justify-between pt-3 border-t border-border/30">
          <div className="flex flex-col gap-0.5">
            <div className="text-[9px] font-medium text-muted-foreground/40 tabular-nums flex items-center gap-1">
              <Clock className="size-2.5" />
              {isActive
                ? new Date().toISOString().split('T')[0]
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
