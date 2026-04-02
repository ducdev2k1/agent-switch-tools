import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface DeleteConfirmDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  profileName: string | null
  onConfirm: () => void
}

export function DeleteConfirmDialog({
  open,
  onOpenChange,
  profileName,
  onConfirm,
}: DeleteConfirmDialogProps) {
  const { t } = useTranslation()
  if (!profileName) return null

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
    >
      <DialogContent onClose={() => onOpenChange(false)}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Trash2 className="size-5 text-destructive" />
            {t('common.actions.delete')} "{profileName}"?
          </DialogTitle>
        </DialogHeader>

        <p className="text-sm text-muted-foreground">
          {t('dashboard.messages.delete_confirm', { name: profileName })}
        </p>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
          >
            {t('common.actions.cancel')}
          </Button>
          <Button
            variant="destructive"
            onClick={onConfirm}
          >
            {t('common.actions.delete')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
