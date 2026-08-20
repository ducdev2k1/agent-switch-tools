import { LogPagination } from '@/components/log-pagination'
import { Badge } from '@/components/ui/badge'
import { formatDetailDates, formatStamp } from '@/lib/log-format'
import type { PrimeLogEntry } from '@/lib/types'
import { useTranslation } from 'react-i18next'
import { RESULT_VARIANT } from './prime-result-variant'

interface AutoPrimeLogTableProps {
  rows: PrimeLogEntry[]
  total: number
  offset: number
  limit: number
  onOffsetChange: (offset: number) => void
}

/** Activity log for scheduled priming. Rows arrive already parsed and paged. */
export function AutoPrimeLogTable({
  rows,
  total,
  offset,
  limit,
  onOffsetChange,
}: AutoPrimeLogTableProps) {
  const { t } = useTranslation()

  return (
    <div className="rounded border border-border/40">
      <div className="max-h-64 overflow-auto">
        <table className="w-full text-xs">
          <thead className="sticky top-0 bg-muted/60 backdrop-blur-sm">
            <tr className="text-left text-muted-foreground">
              <th className="px-3 py-2 font-medium whitespace-nowrap">
                {t('auto_session.log_table.time')}
              </th>
              <th className="px-3 py-2 font-medium">
                {t('auto_session.log_table.account')}
              </th>
              <th className="px-3 py-2 font-medium whitespace-nowrap">
                {t('auto_session.log_table.status')}
              </th>
              <th className="px-3 py-2 font-medium">
                {t('auto_session.log_table.detail')}
              </th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row, i) => (
              <tr
                key={`${row.timestamp}-${row.profile}-${i}`}
                className="border-t border-border/30 hover:bg-muted/30"
              >
                <td className="px-3 py-1.5 whitespace-nowrap tabular-nums text-muted-foreground">
                  {formatStamp(row.timestamp)}
                </td>
                <td className="px-3 py-1.5 whitespace-nowrap">{row.profile}</td>
                <td className="px-3 py-1.5 whitespace-nowrap">
                  {row.result && (
                    <Badge
                      variant={RESULT_VARIANT[row.result] ?? 'secondary'}
                      className="h-4 px-1.5 text-[9px] whitespace-nowrap"
                    >
                      {t(`auto_session.result_badge.${row.result}`, row.result)}
                    </Badge>
                  )}
                </td>
                <td className="px-3 py-1.5 text-muted-foreground">
                  {formatDetailDates(row.detail)}
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
