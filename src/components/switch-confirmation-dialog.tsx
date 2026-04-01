import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import type { CredentialProfile } from "@/lib/types"
import { AlertTriangle, ArrowRightLeft, Terminal } from "lucide-react"

interface SwitchConfirmationDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  targetProfile: CredentialProfile | null
  claudeIsRunning: boolean
  /** Called when user confirms the switch */
  onConfirm: () => void
  switching: boolean
}

export function SwitchConfirmationDialog({
  open,
  onOpenChange,
  targetProfile,
  claudeIsRunning,
  onConfirm,
  switching,
}: SwitchConfirmationDialogProps) {
  if (!targetProfile) return null

  const isExpired = targetProfile.info.isExpired

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent onClose={() => onOpenChange(false)}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <ArrowRightLeft className="size-5" />
            Chuyển sang "{targetProfile.name}"?
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-3">
          {/* Claude running warning */}
          {claudeIsRunning && (
            <div className="flex items-start gap-3 rounded-lg border border-border bg-muted/50 p-3">
              <Terminal className="size-5 text-foreground mt-0.5 shrink-0" />
              <div className="text-sm">
                <p className="font-medium text-foreground">
                  Claude Code đang chạy
                </p>
                <p className="text-muted-foreground mt-1">
                  Phiên hoạt động sẽ tiếp tục sử dụng tài khoản hiện tại.
                  Các phiên mới sẽ sử dụng tài khoản đã chuyển đổi.
                </p>
              </div>
            </div>
          )}

          {/* Expired token warning */}
          {isExpired && (
            <div className="flex items-start gap-3 rounded-lg border border-destructive/50 bg-destructive/10 p-3">
              <AlertTriangle className="size-5 text-destructive mt-0.5 shrink-0" />
              <div className="text-sm">
                <p className="font-medium text-destructive">
                  Mã xác thực (Token) đã hết hạn
                </p>
                <p className="text-muted-foreground mt-1">
                  Mã xác thực này đã hết hạn. Nó có thể tự động làm mới khi sử dụng lần tới.
                </p>
              </div>
            </div>
          )}

          {/* Target profile info */}
          <div className="rounded-lg border bg-muted/30 p-3">
            <div className="flex items-center gap-2 flex-wrap">
              <span className="text-sm font-medium">{targetProfile.name}</span>
              {targetProfile.info.subscriptionType && (
                <Badge variant="secondary" className="text-[10px]">
                  {targetProfile.info.subscriptionType}
                </Badge>
              )}
              {targetProfile.info.rateLimitTier && (
                <Badge variant="outline" className="text-[10px] font-mono">
                  {targetProfile.info.rateLimitTier}
                </Badge>
              )}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              Ngữ cảnh dự án, lịch sử và cài đặt của bạn sẽ được bảo toàn.
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={switching}>
            Hủy
          </Button>
          <Button onClick={onConfirm} disabled={switching}>
            {switching ? "Đang chuyển..." : "Chuyển đổi"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
