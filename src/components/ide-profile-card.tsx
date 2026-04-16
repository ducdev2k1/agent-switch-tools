import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import type { IdeProfile } from '@/lib/types'
import { ArrowRightLeft, Crown, Mail, Trash2, User } from 'lucide-react'
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
  const { name, isActive, email, membershipType, displayName } = profile

  return (
    <Card
      className={`transition-all duration-300 hover:shadow-lg group ${
        isActive
          ? 'border-success/50 bg-success/5 shadow-[0_0_15px_rgba(34,197,94,0.1)] dark:shadow-[0_0_15px_rgba(34,197,94,0.05)]'
          : 'hover:border-primary/50'
      }`}
    >
      <CardContent className="p-4">
        <div className="flex items-start gap-4">
          {/* Status indicator */}
          <div className="mt-1.5 shrink-0">
            <div
              className={`size-3.5 rounded-full transition-all duration-300 ${
                isActive
                  ? 'bg-success shadow-[0_0_10px_rgba(34,197,94,0.8)] animate-pulse'
                  : 'bg-muted-foreground/30'
              }`}
            />
          </div>

          {/* Content */}
          <div className="min-w-0 flex-1">
            <div className="flex items-start justify-between gap-4">
              <div className="min-w-0 flex-1">
                {/* Name + badges */}
                <div className="flex items-center gap-2 flex-wrap">
                  <h3 className="font-bold text-base tracking-tight group-hover:text-primary transition-colors">
                    {displayName || name}
                  </h3>
                  {isActive && (
                    <Badge
                      variant="success"
                      className="text-[10px] font-bold uppercase tracking-wider px-2 py-0"
                    >
                      {t('common.labels.active')}
                    </Badge>
                  )}
                </div>

                {/* Email */}
                {email && (
                  <div className="flex items-center gap-3 mt-1.5 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Mail className="size-3" />
                      {email}
                    </span>
                  </div>
                )}

                {/* Membership / display name info */}
                <div className="flex items-center gap-1.5 mt-2.5 flex-wrap">
                  {membershipType && (
                    <Badge
                      variant="secondary"
                      className="text-[10px] bg-secondary/50 font-medium"
                    >
                      <Crown className="size-3 mr-1 text-primary/70" />
                      {membershipType.replace(/\b\w/g, (c) => c.toUpperCase())}
                    </Badge>
                  )}
                  {displayName && displayName !== name && (
                    <Badge
                      variant="outline"
                      className="text-[10px] font-medium border-primary/20"
                    >
                      <User className="size-3 mr-1 text-primary/70" />
                      {displayName}
                    </Badge>
                  )}
                </div>
              </div>

              {/* Actions (saved profiles only) */}
              {!isActive && (
                <div className="flex items-center gap-1.5 shrink-0 opacity-80 group-hover:opacity-100 transition-opacity">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onSwitch(profile)}
                    className="h-8 text-xs font-bold hover:bg-primary hover:text-primary-foreground border-primary/20"
                  >
                    <ArrowRightLeft className="size-3.5 mr-1" />
                    {t('common.actions.switch')}
                  </Button>
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
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
