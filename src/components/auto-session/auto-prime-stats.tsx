import type { PrimeDayStat } from '@/lib/types'
import { useTranslation } from 'react-i18next'

export function AutoPrimeStats({ stats }: { stats: PrimeDayStat[] }) {
  const { t } = useTranslation()
  if (stats.length === 0) {
    return (
      <p className="py-4 text-center text-sm text-muted-foreground">
        {t('auto_session.no_activity')}
      </p>
    )
  }

  return (
    <div className="max-h-64 overflow-auto rounded border border-border/40">
      <table className="w-full text-xs">
        <thead className="sticky top-0 bg-muted/60 backdrop-blur-sm">
          <tr className="text-left text-muted-foreground">
            <th className="px-3 py-2 font-medium whitespace-nowrap">
              {t('auto_session.stats.date')}
            </th>
            <th className="px-3 py-2 font-medium">
              {t('auto_session.stats.success')}
            </th>
            <th className="px-3 py-2 font-medium">
              {t('auto_session.stats.hold')}
            </th>
            <th className="px-3 py-2 font-medium">
              {t('auto_session.stats.failed')}
            </th>
            <th className="px-3 py-2 font-medium">
              {t('auto_session.stats.skip')}
            </th>
          </tr>
        </thead>
        <tbody>
          {stats.map((s) => (
            <tr
              key={s.date}
              className="border-t border-border/30 hover:bg-muted/30"
            >
              <td className="px-3 py-1.5 whitespace-nowrap tabular-nums text-muted-foreground">
                {s.date}
              </td>
              <td className="px-3 py-1.5">{s.success}</td>
              <td className="px-3 py-1.5 text-muted-foreground">{s.hold}</td>
              <td className="px-3 py-1.5 text-muted-foreground">{s.failed}</td>
              <td className="px-3 py-1.5 text-muted-foreground">{s.skip}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
