import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import type { CredentialProfile } from '@/lib/types'
import {
  AlertTriangle,
  ArrowRightLeft,
  Terminal,
  Mail,
  Building2,
} from 'lucide-react'
import { useTranslation } from 'react-i18next'

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
  const { t } = useTranslation()
  if (!targetProfile) return null

  const isExpired = targetProfile.info.isExpired

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
    >
      <DialogContent onClose={() => onOpenChange(false)}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <ArrowRightLeft className="size-5" />
            {t('dashboard.messages.switch_confirm', {
              name: targetProfile.name,
            })}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-3">
          {/* Claude running warning */}
          {claudeIsRunning && (
            <div className="flex items-start gap-3 rounded-lg border border-border bg-muted/50 p-3">
              <Terminal className="size-5 text-foreground mt-0.5 shrink-0" />
              <div className="text-sm">
                <p className="font-medium text-foreground">
                  {t('dashboard.messages.claude_running_title')}
                </p>
                <p className="text-muted-foreground mt-1">
                  {t('dashboard.messages.claude_running_warning')}
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
                  {t('dashboard.messages.token_expired_title')}
                </p>
                <p className="text-muted-foreground mt-1">
                  {t('dashboard.messages.token_expired_warning')}
                </p>
              </div>
            </div>
          )}

          {/* Target profile info */}
          <div className="rounded-lg border bg-muted/30 p-3">
            <div className="flex items-center gap-2 flex-wrap">
              <span className="text-sm font-medium">
                {targetProfile.oauthAccount?.displayName || targetProfile.name}
              </span>
              {targetProfile.info.subscriptionType && (
                <Badge
                  variant="secondary"
                  className="text-[10px]"
                >
                  {targetProfile.info.subscriptionType}
                </Badge>
              )}
              {targetProfile.info.rateLimitTier && (
                <Badge
                  variant="outline"
                  className="text-[10px] font-mono"
                >
                  {targetProfile.info.rateLimitTier}
                </Badge>
              )}
            </div>
            {targetProfile.oauthAccount?.emailAddress && (
              <div className="flex items-center gap-3 mt-1.5 text-xs text-muted-foreground">
                <span className="flex items-center gap-1">
                  <Mail className="size-3" />
                  {targetProfile.oauthAccount.emailAddress}
                </span>
                {targetProfile.oauthAccount.organizationName && (
                  <span className="flex items-center gap-1">
                    <Building2 className="size-3" />
                    {targetProfile.oauthAccount.organizationName}
                  </span>
                )}
              </div>
            )}
            <p className="text-xs text-muted-foreground mt-1">
              {t('dashboard.messages.preserve_context')}
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={switching}
          >
            {t('common.actions.cancel')}
          </Button>
          <Button
            onClick={onConfirm}
            disabled={switching}
          >
            {switching
              ? t('common.actions.switching')
              : t('common.actions.switch')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
