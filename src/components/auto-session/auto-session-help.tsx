import { HelpCircle } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'

/** Collapsible explainer: what Auto Session does and how to use it. */
export function AutoSessionHelp() {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)

  return (
    <div className="rounded-lg border bg-muted/30">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex w-full items-center gap-2 px-4 py-3 text-left text-sm font-medium"
      >
        <HelpCircle className="size-4 text-primary shrink-0" />
        {t('auto_session.help.title')}
        <span className="ml-auto text-xs text-muted-foreground">
          {open ? '−' : '+'}
        </span>
      </button>

      {open && (
        <div className="space-y-3 px-4 pb-4 text-sm text-muted-foreground">
          <p>{t('auto_session.help.intro')}</p>

          <div>
            <p className="font-medium text-foreground">
              {t('auto_session.help.how_to')}
            </p>
            <ol className="mt-1 list-decimal space-y-1 pl-5">
              <li>{t('auto_session.help.step1')}</li>
              <li>{t('auto_session.help.step2')}</li>
              <li>{t('auto_session.help.step3')}</li>
            </ol>
          </div>

          <p className="rounded border border-border/50 bg-background/50 px-3 py-2 text-xs">
            {t('auto_session.help.note')}
          </p>
        </div>
      )}
    </div>
  )
}
