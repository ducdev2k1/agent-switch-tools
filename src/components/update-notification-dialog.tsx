import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Download } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface UpdateNotificationDialogProps {
  open: boolean
  onDismiss: () => void
  version: string
  body?: string | null
  installing: boolean
  onInstall: () => void
}

/** Modal shown once when a new app version is available */
export function UpdateNotificationDialog({
  open,
  onDismiss,
  version,
  body,
  installing,
  onInstall,
}: UpdateNotificationDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog open={open} onOpenChange={(v) => !v && onDismiss()}>
      <DialogContent className="sm:max-w-md" onClose={onDismiss}>
        <DialogHeader>
          <DialogTitle>{t('update_dialog.title')}</DialogTitle>
          <p className="text-sm text-muted-foreground">
            {t('update_dialog.description', { version })}
          </p>
        </DialogHeader>

        <div className="flex items-center gap-2 rounded-lg border bg-muted/50 px-4 py-3 text-sm">
          <Download className="size-4 text-blue-500 shrink-0" />
          <span>
            {t('update_dialog.version_label')}:{' '}
            <span className="font-mono font-semibold">v{version}</span>
          </span>
        </div>

        {body && body.trim() && (
          <div className="space-y-1.5">
            <p className="text-xs font-semibold text-muted-foreground">
              {t('update_dialog.whats_new')}
            </p>
            <div className="max-h-48 overflow-y-auto whitespace-pre-line rounded-lg border bg-muted/30 px-4 py-3 text-sm text-muted-foreground">
              {body}
            </div>
          </div>
        )}

        <DialogFooter className="gap-2 sm:gap-0">
          <Button variant="outline" size="sm" onClick={onDismiss}>
            {t('update_dialog.later')}
          </Button>
          <Button size="sm" onClick={onInstall} disabled={installing}>
            <Download className="size-3.5" />
            {installing
              ? t('update_dialog.installing')
              : t('update_dialog.install_now')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
