import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useUsageReport, type UsageRange } from '@/hooks/use-usage-report'
import type { PriceStatus } from '@/lib/types'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { UsageCostChart } from './usage-cost-chart'
import { formatCost, formatTokens } from './usage-format'
import { UsageModelTable } from './usage-model-table'
import { UsageSessionTable } from './usage-session-table'

type TFunc = (key: string) => string

const RANGES: { key: string; value: UsageRange }[] = [
  { key: 'today', value: 1 },
  { key: '7d', value: 7 },
  { key: '30d', value: 30 },
  { key: '90d', value: 90 },
  { key: 'all', value: 0 },
]

function priceBadge(status: PriceStatus, t: TFunc) {
  if (status === 'live')
    return <Badge variant="success">{t('usage.price_status.live')}</Badge>
  if (status === 'saved')
    return <Badge variant="secondary">{t('usage.price_status.saved')}</Badge>
  return <Badge variant="outline">{t('usage.price_status.hidden')}</Badge>
}

function StatTile({ label, value }: { label: string; value: string }) {
  return (
    <Card>
      <CardContent className="pt-4">
        <div className="text-xs text-muted-foreground">{label}</div>
        <div className="mt-1 text-2xl font-bold tracking-tight">{value}</div>
      </CardContent>
    </Card>
  )
}

export function UsageView() {
  const { t } = useTranslation()
  const [range, setRange] = useState<UsageRange>(7)
  const { report, loading } = useUsageReport(range)
  const showCost = report?.priceStatus !== 'hidden'

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex gap-1">
          {RANGES.map((r) => (
            <Button
              key={r.value}
              variant={range === r.value ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setRange(r.value)}
            >
              {t(`usage.range.${r.key}`)}
            </Button>
          ))}
        </div>
        {report && priceBadge(report.priceStatus, t)}
      </div>

      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <StatTile
          label={t('usage.tiles.total_cost')}
          value={showCost ? formatCost(report?.totalCostUsd ?? 0) : '—'}
        />
        <StatTile
          label={t('usage.tiles.today_cost')}
          value={showCost ? formatCost(report?.todayCostUsd ?? 0) : '—'}
        />
        <StatTile
          label={t('usage.tiles.output_tokens')}
          value={formatTokens(report?.total.output ?? 0)}
        />
        <StatTile
          label={t('usage.tiles.cache_read')}
          value={formatTokens(report?.total.cacheRead ?? 0)}
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">
            {showCost
              ? t('usage.chart.cost_by_day')
              : t('usage.chart.tokens_by_day')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {report ? (
            <UsageCostChart
              daily={report.daily}
              showCost={showCost}
            />
          ) : (
            <div className="h-48 animate-pulse rounded bg-muted/40" />
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">{t('usage.by_model')}</CardTitle>
        </CardHeader>
        <CardContent>
          {report && (
            <UsageModelTable
              models={report.byModel}
              showCost={showCost}
            />
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">
            {t('usage.recent_sessions')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {report && (
            <UsageSessionTable
              sessions={report.sessions}
              showCost={showCost}
            />
          )}
        </CardContent>
      </Card>

      {loading && (
        <p className="text-center text-xs text-muted-foreground">
          {t('usage.refreshing')}
        </p>
      )}
    </div>
  )
}
