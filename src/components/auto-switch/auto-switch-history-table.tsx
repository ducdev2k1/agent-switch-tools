import { Badge } from '@/components/ui/badge'
import type { AutoSwitchLogEntry } from '@/lib/types'
import { useTranslation } from 'react-i18next'

const RESULT_VARIANT: Record<string, 'success' | 'warning' | 'secondary'> = {
  switched: 'success',
  exhausted: 'warning',
}

const pad = (n: number) => String(n).padStart(2, '0')

/** RFC3339 stamp → local `dd/mm/yyyy hh:mm`; falls back to the raw value. */
function formatTimestamp(stamp: string): string {
  const d = new Date(stamp)
  if (Number.isNaN(d.getTime())) return stamp
  return `${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${d.getFullYear()} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

export function AutoSwitchHistoryTable({
  entries,
}: {
  entries: AutoSwitchLogEntry[]
}) {
  const { t } = useTranslation()

  return (
    <div className="max-h-64 overflow-auto rounded border border-border/40">
      <table className="w-full text-xs">
        <thead className="sticky top-0 bg-muted/60 backdrop-blur-sm">
          <tr className="text-left text-muted-foreground">
            <th className="px-3 py-2 font-medium whitespace-nowrap">
              {t('settings.auto_switch.history.col_time')}
            </th>
            <th className="px-3 py-2 font-medium">
              {t('settings.auto_switch.history.col_from')}
            </th>
            <th className="px-3 py-2 font-medium">
              {t('settings.auto_switch.history.col_to')}
            </th>
            <th className="px-3 py-2 font-medium whitespace-nowrap">
              {t('settings.auto_switch.history.col_utilization')}
            </th>
            <th className="px-3 py-2 font-medium whitespace-nowrap">
              {t('settings.auto_switch.history.col_result')}
            </th>
          </tr>
        </thead>
        <tbody>
          {entries.map((entry, i) => (
            <tr
              key={`${entry.timestamp}-${entry.from}-${i}`}
              className="border-t border-border/30 hover:bg-muted/30"
            >
              <td className="px-3 py-1.5 whitespace-nowrap tabular-nums text-muted-foreground">
                {formatTimestamp(entry.timestamp)}
              </td>
              <td className="px-3 py-1.5 whitespace-nowrap">{entry.from}</td>
              <td className="px-3 py-1.5 whitespace-nowrap">{entry.to}</td>
              <td className="px-3 py-1.5 whitespace-nowrap tabular-nums text-muted-foreground">
                {entry.utilization === null
                  ? '—'
                  : `${Math.round(entry.utilization)}%`}
              </td>
              <td className="px-3 py-1.5 whitespace-nowrap">
                <Badge
                  variant={RESULT_VARIANT[entry.reason] ?? 'secondary'}
                  className="h-4 px-1.5 text-[9px] whitespace-nowrap"
                >
                  {t(
                    `settings.auto_switch.result.${entry.reason}`,
                    entry.reason,
                  )}
                </Badge>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
