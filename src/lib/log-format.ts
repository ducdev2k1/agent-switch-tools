/** Formatting shared by the activity-log tables (scheduled priming, auto switch). */

const pad = (n: number) => String(n).padStart(2, '0')

/** `Date` → `dd/mm/yyyy hh:mm` in local time. */
export function formatDate(d: Date): string {
  return `${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${d.getFullYear()} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

/** Log stamp `YYYY-MM-DD HH:MM:SS` (already local time) → `dd/mm/yyyy hh:mm`. */
export function formatStamp(stamp: string): string {
  const m = /^(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})/.exec(stamp)
  if (!m) return stamp
  return `${m[3]}/${m[2]}/${m[1]} ${m[4]}:${m[5]}`
}

/** Rewrite embedded ISO datetimes (UTC) inside free-form detail text to local time. */
export function formatDetailDates(detail: string): string {
  return detail.replace(
    /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})/g,
    (iso) => {
      const d = new Date(iso)
      return Number.isNaN(d.getTime()) ? iso : formatDate(d)
    },
  )
}
