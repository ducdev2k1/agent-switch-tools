import type { SessionUsage } from '@/lib/types'
import { formatCost, formatTokens, totalTokens } from './usage-format'

export function UsageSessionTable({
  sessions,
  showCost,
}: {
  sessions: SessionUsage[]
  showCost: boolean
}) {
  if (sessions.length === 0) {
    return (
      <p className="py-6 text-center text-sm text-muted-foreground">
        No sessions in this range
      </p>
    )
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b text-left text-xs text-muted-foreground">
            <th className="py-2 pr-4 font-medium">Date</th>
            <th className="py-2 pr-4 font-medium">Project</th>
            <th className="py-2 pr-4 font-medium">Model</th>
            <th className="py-2 pr-4 font-medium">Tokens</th>
            {showCost && <th className="py-2 font-medium">Cost</th>}
          </tr>
        </thead>
        <tbody>
          {sessions.map((s) => (
            <tr
              key={s.id}
              className="border-b border-border/50"
            >
              <td className="py-2 pr-4 text-muted-foreground">{s.date}</td>
              <td className="py-2 pr-4 max-w-[180px] truncate">{s.project}</td>
              <td className="py-2 pr-4 text-muted-foreground">{s.model}</td>
              <td className="py-2 pr-4">{formatTokens(totalTokens(s.tokens))}</td>
              {showCost && <td className="py-2">{formatCost(s.costUsd)}</td>}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
