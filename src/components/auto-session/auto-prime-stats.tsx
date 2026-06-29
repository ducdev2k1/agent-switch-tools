import type { PrimeDayStat } from '@/lib/types'

export function AutoPrimeStats({ stats }: { stats: PrimeDayStat[] }) {
  if (stats.length === 0) {
    return (
      <p className="py-4 text-center text-sm text-muted-foreground">
        No prime activity yet
      </p>
    )
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b text-left text-xs text-muted-foreground">
            <th className="py-2 pr-4 font-medium">Date</th>
            <th className="py-2 pr-4 font-medium">Success</th>
            <th className="py-2 pr-4 font-medium">Hold</th>
            <th className="py-2 pr-4 font-medium">Failed</th>
            <th className="py-2 font-medium">Skip</th>
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
