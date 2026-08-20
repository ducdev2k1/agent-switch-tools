import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import {
  SWITCH_HISTORY_PAGE_SIZE,
  useAutoSwitchConfig,
} from '@/hooks/use-auto-switch-config'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { AutoSwitchHistoryTable } from './auto-switch-history-table'

const THRESHOLD_MIN = 50
const THRESHOLD_MAX = 99
const COOLDOWN_MIN = 5
const COOLDOWN_MAX = 120

/** Parses a typed integer and clamps it; unparseable input keeps `fallback`. */
function clampInput(
  raw: string,
  min: number,
  max: number,
  fallback: number,
): number {
  const parsed = Number.parseInt(raw, 10)
  if (Number.isNaN(parsed)) return fallback
  return Math.min(max, Math.max(min, parsed))
}

interface NumberRowProps {
  id: string
  label: string
  hint: string
  suffix: string
  value: number
  min: number
  max: number
  disabled: boolean
  onCommit: (value: number) => void
}

/**
 * Numeric setting row. Keeps the raw text while typing and only clamps and
 * persists on blur, so partial input is never rejected mid-edit. Callers remount
 * it (via `key`) whenever the persisted value changes.
 */
function NumberRow({
  id,
  label,
  hint,
  suffix,
  value,
  min,
  max,
  disabled,
  onCommit,
}: NumberRowProps) {
  const [draft, setDraft] = useState(String(value))

  const commit = () => {
    const next = clampInput(draft, min, max, value)
    setDraft(String(next))
    if (next !== value) onCommit(next)
  }

  return (
    <div className="flex items-center justify-between gap-4 px-4 py-3">
      <div>
        <Label
          className="text-sm"
          htmlFor={id}
        >
          {label}
        </Label>
        <p className="text-xs text-muted-foreground mt-0.5">{hint}</p>
      </div>
      <div className="flex shrink-0 items-center gap-2">
        <Input
          id={id}
          type="number"
          min={min}
          max={max}
          value={draft}
          disabled={disabled}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commit}
          className="h-8 w-20 text-xs"
        />
        <span className="text-xs text-muted-foreground">{suffix}</span>
      </div>
    </div>
  )
}

export function AutoSwitchPanel() {
  const { t } = useTranslation()
  const {
    config,
    history,
    historyTotal,
    historyOffset,
    setHistoryOffset,
    loading,
    save,
  } = useAutoSwitchConfig()

  // Threshold and cooldown only matter while the rule is armed.
  const numbersDisabled = loading || !config.enabled

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1 px-1">
          {t('settings.auto_switch.title')}
        </h3>
        <p className="text-xs text-muted-foreground mb-2 px-1">
          {t('settings.auto_switch.description')}
        </p>
        <div className="rounded-lg border bg-card">
          <div className="flex items-center justify-between gap-4 px-4 py-3">
            <div>
              <Label className="text-sm">
                {t('settings.auto_switch.enabled')}
              </Label>
              <p className="text-xs text-muted-foreground mt-0.5">
                {t('settings.auto_switch.enabled_hint')}
              </p>
            </div>
            <Switch
              checked={config.enabled}
              disabled={loading}
              onCheckedChange={(enabled) => void save({ ...config, enabled })}
            />
          </div>

          <div className="border-t" />

          <NumberRow
            key={`threshold-${config.threshold}`}
            id="auto-switch-threshold"
            label={t('settings.auto_switch.threshold')}
            hint={t('settings.auto_switch.threshold_hint')}
            suffix="%"
            value={config.threshold}
            min={THRESHOLD_MIN}
            max={THRESHOLD_MAX}
            disabled={numbersDisabled}
            onCommit={(threshold) => void save({ ...config, threshold })}
          />

          <div className="border-t" />

          <NumberRow
            key={`cooldown-${config.cooldownMinutes}`}
            id="auto-switch-cooldown"
            label={t('settings.auto_switch.cooldown')}
            hint={t('settings.auto_switch.cooldown_hint')}
            suffix={t('settings.auto_switch.minutes')}
            value={config.cooldownMinutes}
            min={COOLDOWN_MIN}
            max={COOLDOWN_MAX}
            disabled={numbersDisabled}
            onCommit={(cooldownMinutes) =>
              void save({ ...config, cooldownMinutes })
            }
          />
        </div>
      </div>

      <div className="rounded-lg border border-warning/30 bg-warning/5 px-4 py-3 text-sm">
        {t('settings.auto_switch.restart_warning')}
      </div>

      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 px-1">
          {t('settings.auto_switch.history.title')}
        </h3>
        {historyTotal === 0 ? (
          <p className="px-1 text-sm text-muted-foreground">
            {t('settings.auto_switch.history.empty')}
          </p>
        ) : (
          <AutoSwitchHistoryTable
            entries={history}
            total={historyTotal}
            offset={historyOffset}
            limit={SWITCH_HISTORY_PAGE_SIZE}
            onOffsetChange={setHistoryOffset}
          />
        )}
      </div>
    </div>
  )
}
