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
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b text-left text-xs text-muted-foreground">
            <th className="py-2 pr-4 font-medium">
              {t('auto_session.stats.date')}
            </th>
            <th className="py-2 pr-4 font-medium">
              {t('auto_session.stats.success')}
            </th>
            <th className="py-2 pr-4 font-medium">
              {t('auto_session.stats.hold')}
            </th>
            <th className="py-2 pr-4 font-medium">
              {t('auto_session.stats.failed')}
            </th>
            <th className="py-2 font-medium">
              {t('auto_session.stats.skip')}
            </th>
          </tr>
        </thead>
        <tbody>
          {stats.map((s) => (
            <tr
              key={s.date}
              className="border-b border-border/50"
            >
              <td className="py-2 pr-4">{s.date}</td>
              <td className="py-2 pr-4 text-muted-foreground">{s.success}</td>
              <td className="py-2 pr-4 text-muted-foreground">{s.hold}</td>
              <td className="py-2 pr-4 text-muted-foreground">{s.failed}</td>
              <td className="py-2 text-muted-foreground">{s.skip}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
