import { Badge } from '@/components/ui/badge'
import { useTranslation } from 'react-i18next'
import { RESULT_VARIANT } from './prime-result-variant'

interface LogRow {
  time: string
  account: string
  status: string
  detail: string
}

const pad = (n: number) => String(n).padStart(2, '0')

/** `Date` → `dd/mm/yyyy hh:mm` in local time. */
function formatDate(d: Date): string {
  return `${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${d.getFullYear()} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

/** Log stamp `YYYY-MM-DD HH:MM:SS` (already local time) → `dd/mm/yyyy hh:mm`. */
function formatStamp(stamp: string): string {
  const m = /^(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})/.exec(stamp)
  if (!m) return stamp
  return `${m[3]}/${m[2]}/${m[1]} ${m[4]}:${m[5]}`
}

/** Rewrite embedded ISO datetimes (UTC) inside free-form detail text to local dd/mm/yyyy hh:mm. */
function formatDetailDates(detail: string): string {
  return detail.replace(
    /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})/g,
    (iso) => {
      const d = new Date(iso)
      return Number.isNaN(d.getTime()) ? iso : formatDate(d)
    },
  )
}

/** Parse `stamp | account | status | detail` lines, newest first. */
function parseLog(log: string): LogRow[] {
  return log
    .split('\n')
    .map((l) => l.trim())
    .filter(Boolean)
    .map((line) => {
      const parts = line.split(' | ')
      if (parts.length < 3)
        return { time: '', account: '', status: '', detail: line }
      return {
        time: formatStamp(parts[0]),
        account: parts[1],
        status: parts[2],
        detail: formatDetailDates(parts.slice(3).join(' | ')),
      }
    })
    .reverse()
}

export function AutoPrimeLogTable({ log }: { log: string }) {
  const { t } = useTranslation()
  const rows = parseLog(log)

  return (
    <div className="max-h-64 overflow-auto rounded border border-border/40">
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
              key={`${row.time}-${row.account}-${i}`}
              className="border-t border-border/30 hover:bg-muted/30"
            >
              <td className="px-3 py-1.5 whitespace-nowrap tabular-nums text-muted-foreground">
                {row.time}
              </td>
              <td className="px-3 py-1.5 whitespace-nowrap">{row.account}</td>
              <td className="px-3 py-1.5 whitespace-nowrap">
                {row.status && (
                  <Badge
                    variant={RESULT_VARIANT[row.status] ?? 'secondary'}
                    className="h-4 px-1.5 text-[9px] whitespace-nowrap"
                  >
                    {t(`auto_session.result_badge.${row.status}`, row.status)}
                  </Badge>
                )}
              </td>
              <td className="px-3 py-1.5 text-muted-foreground">
                {row.detail}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
