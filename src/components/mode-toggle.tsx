import { useTheme } from '@/components/theme-provider'
import { Button } from '@/components/ui/button'
import { Laptop, Moon, Sun } from 'lucide-react'

export function ModeToggle() {
  const { theme, setTheme } = useTheme()

  const cycleTheme = () => {
    if (theme === 'light') setTheme('dark')
    else if (theme === 'dark') setTheme('system')
    else setTheme('light')
  }

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={cycleTheme}
      title={`Giao diện: ${theme === 'light' ? 'Sáng' : theme === 'dark' ? 'Tối' : 'Hệ thống'} (Nhấp để chuyển đổi)`}
      className="size-8"
    >
      {theme === 'light' && <Sun className="size-4" />}
      {theme === 'dark' && <Moon className="size-4" />}
      {theme === 'system' && <Laptop className="size-4" />}
      <span className="sr-only">Chuyển đổi giao diện</span>
    </Button>
  )
}
