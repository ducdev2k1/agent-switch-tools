import type { DayUsage } from '@/lib/types'
import { useTranslation } from 'react-i18next'
import {
  Bar,
  BarChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { formatCost, formatTokens, totalTokens } from './usage-format'

const MAX_BARS = 45

interface ChartDatum {
  date: string
  value: number
  tokens: number
  cost: number | null
}

interface TooltipPayload {
  payload: ChartDatum
}

function ChartTooltip({
  active,
  payload,
  showCost,
  tokensLabel,
}: {
  active?: boolean
  payload?: TooltipPayload[]
  showCost: boolean
  tokensLabel: string
}) {
  if (!active || !payload?.length) return null
  const d = payload[0].payload
  return (
    <div className="rounded-md border bg-popover px-3 py-2 text-xs shadow-md">
      <div className="font-medium">{d.date}</div>
      <div className="text-muted-foreground">
        {formatTokens(d.tokens)} {tokensLabel}
      </div>
      {showCost && (
        <div className="text-muted-foreground">{formatCost(d.cost)}</div>
      )}
    </div>
  )
}

export function UsageCostChart({
  daily,
  showCost,
}: {
  daily: DayUsage[]
  showCost: boolean
}) {
  const { t } = useTranslation()
  // Keep the chart readable: only the most recent MAX_BARS days.
  const data: ChartDatum[] = daily.slice(-MAX_BARS).map((d) => ({
    date: d.date.slice(5),
    value: showCost ? (d.costUsd ?? 0) : totalTokens(d.tokens),
    tokens: totalTokens(d.tokens),
    cost: d.costUsd,
  }))

  if (data.length === 0) {
    return (
      <div className="flex h-48 items-center justify-center text-sm text-muted-foreground">
        {t('usage.chart.no_data')}
      </div>
    )
  }

  return (
    <ResponsiveContainer
      width="100%"
      height={220}
    >
      <BarChart
        data={data}
        margin={{ top: 8, right: 8, left: 8, bottom: 0 }}
      >
        <XAxis
          dataKey="date"
          tick={{ fontSize: 10 }}
          interval="preserveStartEnd"
        />
        <YAxis
          tick={{ fontSize: 10 }}
          width={48}
          tickFormatter={(v: number) => (showCost ? `$${v}` : formatTokens(v))}
        />
        <Tooltip
          cursor={{ fill: 'var(--muted)' }}
          content={
            <ChartTooltip
              showCost={showCost}
              tokensLabel={t('usage.chart.tokens_suffix')}
            />
          }
        />
        <Bar
          dataKey="value"
          fill="var(--primary)"
          radius={[3, 3, 0, 0]}
        />
      </BarChart>
    </ResponsiveContainer>
  )
}
