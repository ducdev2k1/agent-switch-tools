import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { CHANGELOG } from '@/lib/changelog'
import { Sparkles } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface ChangelogDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

/** Scrollable history of release highlights, bundled for offline use. */
export function ChangelogDialog({ open, onOpenChange }: ChangelogDialogProps) {
  const { t, i18n } = useTranslation()
  const isVi = i18n.language.startsWith('vi')

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="sm:max-w-lg"
        onClose={() => onOpenChange(false)}
      >
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Sparkles className="size-4 text-primary" />
            {t('changelog.title')}
          </DialogTitle>
          <p className="text-sm text-muted-foreground">
            {t('changelog.description')}
          </p>
        </DialogHeader>

        <div className="max-h-[60vh] space-y-5 overflow-y-auto pr-1">
          {CHANGELOG.map((entry, idx) => {
            const items = isVi ? entry.vi : entry.en
            return (
              <div key={entry.version}>
                <div className="flex items-center gap-2">
                  <span className="font-mono text-sm font-semibold">
                    v{entry.version}
                  </span>
                  {idx === 0 && (
                    <span className="rounded bg-primary/10 px-1.5 py-0.5 text-[10px] font-semibold text-primary">
                      {t('changelog.latest')}
                    </span>
                  )}
                  <span className="text-xs text-muted-foreground">
                    {entry.date}
                  </span>
                </div>
                <ul className="mt-2 list-disc space-y-1 pl-5 text-sm text-muted-foreground">
                  {items.map((line, i) => (
                    <li key={i}>{line}</li>
                  ))}
                </ul>
              </div>
            )
          })}
        </div>
      </DialogContent>
    </Dialog>
  )
}
