import { LogPagination } from '@/components/log-pagination'
import { Badge } from '@/components/ui/badge'
import { formatStamp } from '@/lib/log-format'
import type { AutoSwitchLogEntry } from '@/lib/types'
import { useTranslation } from 'react-i18next'

const RESULT_VARIANT: Record<string, 'success' | 'warning' | 'secondary'> = {
  switched: 'success',
  exhausted: 'warning',
}

interface AutoSwitchHistoryTableProps {
  entries: AutoSwitchLogEntry[]
  total: number
  offset: number
  limit: number
  onOffsetChange: (offset: number) => void
}

export function AutoSwitchHistoryTable({
  entries,
  total,
  offset,
  limit,
  onOffsetChange,
}: AutoSwitchHistoryTableProps) {
  const { t } = useTranslation()

  return (
    <div className="rounded border border-border/40">
      <div className="max-h-64 overflow-auto">
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
                  {formatStamp(entry.timestamp)}
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
      <LogPagination
        offset={offset}
        limit={limit}
        total={total}
        onOffsetChange={onOffsetChange}
      />
    </div>
  )
}
