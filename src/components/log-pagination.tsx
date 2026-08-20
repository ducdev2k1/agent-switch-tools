import { Button } from '@/components/ui/button'
import { ChevronLeft, ChevronRight } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface LogPaginationProps {
  offset: number
  limit: number
  total: number
  onOffsetChange: (offset: number) => void
}

/**
 * Prev/next footer for the activity-log tables. Renders nothing when everything
 * already fits on one page, so short logs stay visually clean.
 */
export function LogPagination({
  offset,
  limit,
  total,
  onOffsetChange,
}: LogPaginationProps) {
  const { t } = useTranslation()
  if (total <= limit) return null

  const from = offset + 1
  const to = Math.min(offset + limit, total)

  return (
    <div className="flex items-center justify-between px-3 py-2 text-xs text-muted-foreground">
      <span className="tabular-nums">
        {t('common.pagination.range', { from, to, total })}
      </span>
      <div className="flex gap-1">
        <Button
          variant="ghost"
          size="icon"
          className="size-6"
          disabled={offset === 0}
          onClick={() => onOffsetChange(Math.max(0, offset - limit))}
          aria-label={t('common.pagination.prev')}
        >
          <ChevronLeft className="size-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="size-6"
          disabled={to >= total}
          onClick={() => onOffsetChange(offset + limit)}
          aria-label={t('common.pagination.next')}
        >
          <ChevronRight className="size-3.5" />
        </Button>
      </div>
    </div>
  )
}
