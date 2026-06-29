import type { TokenBreakdown } from '@/lib/types'

/** Sum of all token categories. */
export function totalTokens(t: TokenBreakdown): number {
  return t.input + t.output + t.cacheRead + t.cacheCreation
}

/** Compact token count, e.g. 1.2M / 345K / 980. */
export function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return `${n}`
}

/** USD cost, with extra precision for sub-cent amounts. */
export function formatCost(value: number | null): string {
  if (value == null) return '—'
  if (value > 0 && value < 0.01) return `$${value.toFixed(4)}`
  return `$${value.toFixed(2)}`
}
