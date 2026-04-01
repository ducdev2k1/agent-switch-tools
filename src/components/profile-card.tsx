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
  isCurrentlyActive?: boolean
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
    if (ago < 1) return `Hết hạn ${Math.round(ago * 60)} phút trước`
    if (ago < 24) return `Hết hạn ${Math.round(ago)} giờ trước`
    return `Hết hạn ${Math.round(ago / 24)} ngày trước`
  }
  if (hoursLeft < 1) return `Hết hạn sau ${Math.round(hoursLeft * 60)} phút`
  if (hoursLeft < 24) return `Hết hạn sau ${Math.round(hoursLeft)} giờ`
  return `Hết hạn sau ${Math.round(hoursLeft / 24)} ngày`
}

export function ProfileCard({
  profile,
  onSwitch,
  onRename,
  onDelete,
  isCurrentlyActive = false,
}: ProfileCardProps) {
  const { name, isActive, info } = profile
  const expired = info.isExpired
  const expiryText = formatExpiry(info.expiresInHours, expired)

  // Health color indicator logic
  const expiringSoon = !expired && info.expiresInHours !== null && info.expiresInHours < 24

  return (
    <Card
      className={`transition-all duration-300 hover:shadow-lg group ${
        isActive
          ? "border-success/50 bg-success/5 shadow-[0_0_15px_rgba(34,197,94,0.1)] dark:shadow-[0_0_15px_rgba(34,197,94,0.05)]"
          : isCurrentlyActive
            ? "border-primary/30 bg-accent/30"
            : expired
              ? "border-destructive/30 bg-destructive/5"
              : expiringSoon
                ? "border-warning/30 bg-warning/5"
                : "hover:border-primary/50"
      }`}
    >
      <CardContent className="p-4">
        <div className="flex items-start justify-between gap-4">
          {/* Left: indicator + info */}
          <div className="flex items-start gap-4 min-w-0 flex-1">
            <div className="mt-1.5 shrink-0">
              <div
                className={`size-3.5 rounded-full transition-all duration-300 ${
                  isActive
                    ? "bg-success shadow-[0_0_10px_rgba(34,197,94,0.8)] animate-pulse"
                    : isCurrentlyActive
                      ? "bg-primary/50"
                      : expired
                        ? "bg-destructive/60"
                        : expiringSoon
                          ? "bg-warning/80"
                          : "bg-muted-foreground/30"
                }`}
              />
            </div>

            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2 flex-wrap">
                <h3 className="font-bold text-base tracking-tight group-hover:text-primary transition-colors">
                  {name}
                </h3>
                {isActive && (
                  <Badge variant="success" className="text-[10px] font-bold uppercase tracking-wider px-2 py-0">
                    Hoạt động
                  </Badge>
                )}
                {isCurrentlyActive && !isActive && (
                  <Badge variant="secondary" className="text-[10px] font-bold uppercase tracking-wider px-2 py-0">
                    Trùng khớp
                  </Badge>
                )}
                {expired && (
                  <Badge variant="destructive" className="text-[10px] font-bold uppercase tracking-wider px-2 py-0">
                    Hết hạn
                  </Badge>
                )}
                {expiringSoon && (
                  <Badge variant="warning" className="text-[10px] font-bold uppercase tracking-wider px-2 py-0">
                    Sắp hết hạn
                  </Badge>
                )}
              </div>

              {/* Extras indicators */}
              <div className="flex items-center gap-1.5 mt-2.5 flex-wrap">
                {info.subscriptionType && (
                  <Badge variant="secondary" className="text-[10px] bg-secondary/50 font-medium">
                    <Crown className="size-3 mr-1 text-primary/70" />
                    {formatSubscription(info.subscriptionType)}
                  </Badge>
                )}
                {info.rateLimitTier && (
                  <Badge variant="outline" className="text-[10px] font-mono font-medium border-primary/20">
                    <Zap className="size-3 mr-1 text-primary/70" />
                    {info.rateLimitTier}
                  </Badge>
                )}
                {expiryText && (
                  <Badge
                    variant={expired ? "destructive" : expiringSoon ? "warning" : "outline"}
                    className="text-[10px] font-medium"
                  >
                    <Clock className="size-3 mr-1" />
                    {expiryText}
                  </Badge>
                )}
              </div>

              {/* Scopes summary */}
              {info.scopes.length > 0 && (
                <p className="text-[11px] text-muted-foreground mt-2 font-medium flex items-center gap-1">
                  <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/30" />
                  {info.scopes.length} quyền (scopes)
                  <span className="text-muted-foreground/40 mx-1">|</span>
                  {info.organizationUuid ? "Tài khoản Tổ chức" : "Tài khoản Cá nhân"}
                </p>
              )}
            </div>
          </div>

          {/* Right: actions */}
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
                  Chuyển đổi
                </Button>
              )}
              <Button
                variant="ghost"
                size="icon"
                onClick={() => onRename(profile)}
                className="size-8 hover:bg-accent"
              >
                <Pencil className="size-3.5 text-muted-foreground" />
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
      </CardContent>
    </Card>
  )
}
