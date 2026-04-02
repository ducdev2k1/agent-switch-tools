import { ThemeProvider } from '@/components/theme-provider'
import { Dashboard } from '@/pages/dashboard'
import { Toaster } from 'sonner'

function App() {
  return (
    <ThemeProvider
      defaultTheme="system"
      storageKey="claude-ui-theme"
    >
      <Dashboard />
      <Toaster
        position="bottom-right"
        richColors
      />
    </ThemeProvider>
  )
}

export default App
