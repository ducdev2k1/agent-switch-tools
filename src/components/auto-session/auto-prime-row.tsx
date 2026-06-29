import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import type { AutoPrimeSetting, PrimeResult } from '@/lib/types'
import { useState } from 'react'

const TIME_RE = /^([01]\d|2[0-3]):[0-5]\d$/

/** Approximate reset clock = scheduled time + 5h (same-day wrap). */
function resetHint(time: string): string {
  const m = TIME_RE.exec(time)
  if (!m) return ''
  const h = (Number(time.slice(0, 2)) + 5) % 24
  return `${String(h).padStart(2, '0')}:${time.slice(3)}`
}

const RESULT_VARIANT: Record<
  string,
  'success' | 'destructive' | 'secondary' | 'warning'
> = {
  success: 'success',
  failed: 'destructive',
  hold: 'warning',
  skip: 'secondary',
}

export function AutoPrimeRow({
  name,
  setting,
  onSave,
  onPrimeNow,
}: {
  name: string
  setting?: AutoPrimeSetting
  onSave: (name: string, enabled: boolean, time: string) => void
  onPrimeNow: (name: string) => Promise<PrimeResult>
}) {
  const [time, setTime] = useState(setting?.time || '09:00')
  const [enabled, setEnabled] = useState(setting?.enabled ?? false)
  const [priming, setPriming] = useState(false)
  const valid = TIME_RE.test(time)

  const toggle = (next: boolean) => {
    setEnabled(next)
    if (valid) onSave(name, next, time)
  }

  const handlePrime = async () => {
    setPriming(true)
    try {
      await onPrimeNow(name)
    } finally {
      setPriming(false)
    }
  }

  return (
    <div className="flex items-center gap-3 rounded-lg border p-3">
      <div className="min-w-0 flex-1">
        <div className="truncate font-medium">{name}</div>
        {valid && (
          <div className="text-xs text-muted-foreground">
            {time} → reset ~{resetHint(time)}
          </div>
        )}
        {setting?.lastResult && (
          <Badge
            variant={RESULT_VARIANT[setting.lastResult] ?? 'secondary'}
            className="mt-1"
          >
            {setting.lastResult}
          </Badge>
        )}
      </div>
      <Input
        type="time"
        value={time}
        onChange={(e) => setTime(e.target.value)}
        onBlur={() => {
          if (valid && enabled) onSave(name, enabled, time)
        }}
        className="w-28"
      />
      <Switch
        checked={enabled}
        onCheckedChange={toggle}
      />
      <Button
        variant="outline"
        size="sm"
        onClick={handlePrime}
        disabled={priming}
      >
        {priming ? '…' : 'Prime now'}
      </Button>
    </div>
  )
}
