import type { ModelUsage } from '@/lib/types'
import { useTranslation } from 'react-i18next'
import { formatCost, formatTokens, totalTokens } from './usage-format'

export function UsageModelTable({
  models,
  showCost,
}: {
  models: ModelUsage[]
  showCost: boolean
}) {
  const { t } = useTranslation()
  if (models.length === 0) {
    return (
      <p className="py-6 text-center text-sm text-muted-foreground">
        {t('usage.table.no_models')}
      </p>
    )
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b text-left text-xs text-muted-foreground">
            <th className="py-2 pr-4 font-medium">{t('usage.table.model')}</th>
            <th className="py-2 pr-4 font-medium">{t('usage.table.input')}</th>
            <th className="py-2 pr-4 font-medium">{t('usage.table.output')}</th>
            <th className="py-2 pr-4 font-medium">{t('usage.table.cache')}</th>
            <th className="py-2 pr-4 font-medium">{t('usage.table.total')}</th>
            {showCost && (
              <th className="py-2 font-medium">{t('usage.table.cost')}</th>
            )}
          </tr>
        </thead>
        <tbody>
          {models.map((m) => (
            <tr
              key={m.model}
              className="border-b border-border/50"
            >
              <td className="py-2 pr-4 font-medium">{m.model}</td>
              <td className="py-2 pr-4 text-muted-foreground">
                {formatTokens(m.tokens.input)}
              </td>
              <td className="py-2 pr-4 text-muted-foreground">
                {formatTokens(m.tokens.output)}
              </td>
              <td className="py-2 pr-4 text-muted-foreground">
                {formatTokens(m.tokens.cacheRead + m.tokens.cacheCreation)}
              </td>
              <td className="py-2 pr-4">{formatTokens(totalTokens(m.tokens))}</td>
              {showCost && <td className="py-2">{formatCost(m.costUsd)}</td>}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
