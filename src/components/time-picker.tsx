import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

const HOURS = Array.from({ length: 24 }, (_, i) => String(i).padStart(2, '0'))
const MINUTES = Array.from({ length: 60 }, (_, i) => String(i).padStart(2, '0'))

interface TimePickerProps {
  /** "HH:MM" (24h). */
  value: string
  onChange: (value: string) => void
  disabled?: boolean
  className?: string
}

/**
 * Hour/minute time picker built from shadcn Select — avoids the native
 * `input[type=time]` which renders inconsistently inside the WebKitGTK webview
 * and could leave the value (and the derived reset hint) stale.
 */
export function TimePicker({
  value,
  onChange,
  disabled,
  className,
}: TimePickerProps) {
  const [hh = '09', mm = '00'] = value.split(':')

  return (
    <div className={`flex items-center gap-1 ${className ?? ''}`}>
      <Select
        value={hh}
        onValueChange={(h) => onChange(`${h}:${mm}`)}
        disabled={disabled}
      >
        <SelectTrigger className="h-8 w-16 text-xs">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {HOURS.map((h) => (
            <SelectItem key={h} value={h}>
              {h}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <span className="text-muted-foreground">:</span>
      <Select
        value={mm}
        onValueChange={(m) => onChange(`${hh}:${m}`)}
        disabled={disabled}
      >
        <SelectTrigger className="h-8 w-16 text-xs">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {MINUTES.map((m) => (
            <SelectItem key={m} value={m}>
              {m}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
