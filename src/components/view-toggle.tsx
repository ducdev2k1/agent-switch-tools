import { Button } from '@/components/ui/button'
import { LayoutGrid, List } from 'lucide-react'

export type ViewMode = 'grid' | 'list'

interface ViewToggleProps {
  mode: ViewMode
  onChange: (mode: ViewMode) => void
}

export function ViewToggle({ mode, onChange }: ViewToggleProps) {
  return (
    <div className="flex items-center gap-1 bg-muted/30 p-1 rounded-lg border border-border/40">
      <Button
        variant={mode === 'grid' ? 'secondary' : 'ghost'}
        size="icon"
        className="size-8"
        onClick={() => onChange('grid')}
      >
        <LayoutGrid className="size-4" />
      </Button>
      <Button
        variant={mode === 'list' ? 'secondary' : 'ghost'}
        size="icon"
        className="size-8"
        onClick={() => onChange('list')}
      >
        <List className="size-4" />
      </Button>
    </div>
  )
}
