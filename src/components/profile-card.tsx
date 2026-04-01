import { Card, CardContent } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import type { CredentialProfile } from "@/lib/types"
import {
  ArrowRightLeft,
  Pencil,
  Trash2,
  Crown,
  Zap,
  Clock,
} from "lucide-react"

interface ProfileCardProps {
  profile: CredentialProfile
  onSwitch: (profile: CredentialProfile) => void
  onRename: (profile: CredentialProfile) => void
  onDelete: (name: string) => void
}

/** Human-readable subscription name */
function formatSubscription(sub: string | null): string {
  if (!sub) return "Không rõ"
  return sub.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase())
}

/** Format expiry as relative time string */
function formatExpiry(hoursLeft: number | null, isExpired: boolean): string | null {
  if (hoursLeft === null) return null
  if (isExpired) {
    const ago = Math.abs(hoursLeft)
    if (ago < 1) return `Expired ${Math.round(ago * 60)}m ago`
    if (ago < 24) return `Expired ${Math.round(ago)}h ago`
    return `Expired ${Math.round(ago / 24)}d ago`
  }
  if (hoursLeft < 1) return `Expires in ${Math.round(hoursLeft * 60)}m`
  if (hoursLeft < 24) return `Expires in ${Math.round(hoursLeft)}h`
  return `Expires in ${Math.round(hoursLeft / 24)}d`
}

export function ProfileCard({
  profile,
  onSwitch,
  onRename,
  onDelete,
}: ProfileCardProps) {
  const { name, isActive, info } = profile
  const expired = info.isExpired
  const expiryText = formatExpiry(info.expiresInHours, expired)

  // Health color: green=active, amber=expiring soon (<24h), red=expired
  const expiringSoon = !expired && info.expiresInHours !== null && info.expiresInHours < 24

  return (
    <Card
      className={`transition-all duration-200 hover:shadow-md ${
        isActive
          ? "border-emerald-500/50 bg-emerald-500/5 shadow-emerald-500/10"
          : expired
            ? "border-destructive/30 bg-destructive/5"
            : expiringSoon
              ? "border-amber-500/30 bg-amber-500/5"
              : "hover:border-primary/30"
      }`}
    >
      <CardContent className="p-4">
        <div className="flex items-start justify-between gap-4">
          {/* Left: indicator + info */}
          <div className="flex items-start gap-3 min-w-0 flex-1">
            <div className="mt-1.5 shrink-0">
              <div
                className={`size-3 rounded-full ${
                  isActive
                    ? "bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]"
                    : expired
                      ? "bg-destructive/60"
                      : expiringSoon
                        ? "bg-amber-500/60"
                        : "bg-muted-foreground/30"
                }`}
              />
            </div>

            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2 flex-wrap">
                <h3 className="font-semibold text-base truncate">{name}</h3>
                {isActive && (
                  <Badge variant="success" className="text-[10px] uppercase tracking-wider">
                    Hoạt động
                  </Badge>
                )}
                {expired && (
                  <Badge variant="destructive" className="text-[10px] uppercase tracking-wider">
                    Hết hạn
                  </Badge>
                )}
                {expiringSoon && (
                  <Badge variant="warning" className="text-[10px] uppercase tracking-wider">
                    Expiring Soon
                  </Badge>
                )}
              </div>

              {/* Subscription & tier badges */}
              <div className="flex items-center gap-1.5 mt-2 flex-wrap">
                {info.subscriptionType && (
                  <Badge variant="secondary" className="text-[10px]">
                    <Crown className="size-3 mr-0.5" />
                    {formatSubscription(info.subscriptionType)}
                  </Badge>
                )}
                {info.rateLimitTier && (
                  <Badge variant="outline" className="text-[10px] font-mono">
                    <Zap className="size-3 mr-0.5" />
                    {info.rateLimitTier}
                  </Badge>
                )}
                {expiryText && (
                  <Badge
                    variant={expired ? "destructive" : expiringSoon ? "warning" : "outline"}
                    className="text-[10px]"
                  >
                    <Clock className="size-3 mr-0.5" />
                    {expiryText}
                  </Badge>
                )}
              </div>

              {/* Scopes summary */}
              {info.scopes.length > 0 && (
                <p className="text-[11px] text-muted-foreground mt-1.5 truncate">
                  {info.scopes.length} quyền (scopes) · {info.organizationUuid ? "Tổ chức" : "Cá nhân"}
                </p>
              )}
            </div>
          </div>

          {/* Right: actions (only for saved profiles) */}
          {!isActive && (
            <div className="flex items-center gap-1 shrink-0">
              <Button
                variant="outline"
                size="sm"
                onClick={() => onSwitch(profile)}
                className="text-xs"
              >
                <ArrowRightLeft className="size-3.5" />
                Chuyển đổi
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => onRename(profile)}
                className="size-8"
              >
                <Pencil className="size-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => onDelete(name)}
                className="size-8 text-destructive hover:text-destructive"
              >
                <Trash2 className="size-3.5" />
              </Button>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
