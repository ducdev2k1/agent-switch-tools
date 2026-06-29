import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useUsageReport, type UsageRange } from '@/hooks/use-usage-report'
import type { PriceStatus } from '@/lib/types'
import { useState } from 'react'
import { UsageCostChart } from './usage-cost-chart'
import { formatCost, formatTokens } from './usage-format'
import { UsageModelTable } from './usage-model-table'
import { UsageSessionTable } from './usage-session-table'

const RANGES: { label: string; value: UsageRange }[] = [
  { label: '7d', value: 7 },
  { label: '30d', value: 30 },
  { label: '90d', value: 90 },
  { label: 'All', value: 0 },
]

function priceBadge(status: PriceStatus) {
  if (status === 'live') return <Badge variant="success">Live prices</Badge>
  if (status === 'saved') return <Badge variant="secondary">Saved prices</Badge>
  return <Badge variant="outline">Cost hidden</Badge>
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
              {r.label}
            </Button>
          ))}
        </div>
        {report && priceBadge(report.priceStatus)}
      </div>

      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <StatTile
          label="Total cost"
          value={showCost ? formatCost(report?.totalCostUsd ?? 0) : '—'}
        />
        <StatTile
          label="Today cost"
          value={showCost ? formatCost(report?.todayCostUsd ?? 0) : '—'}
        />
        <StatTile
          label="Output tokens"
          value={formatTokens(report?.total.output ?? 0)}
        />
        <StatTile
          label="Cache read"
          value={formatTokens(report?.total.cacheRead ?? 0)}
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">
            {showCost ? 'Cost by day' : 'Tokens by day'}
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
          <CardTitle className="text-sm">By model</CardTitle>
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
          <CardTitle className="text-sm">Recent sessions</CardTitle>
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
        <p className="text-center text-xs text-muted-foreground">Refreshing…</p>
      )}
    </div>
  )
}
