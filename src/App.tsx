import { ThemeProvider } from '@/components/theme-provider'
import { useWebhookSender } from '@/hooks/use-webhook-sender'
import { Dashboard } from '@/pages/dashboard'
import { SettingsPage } from '@/pages/settings-page'
import { useState } from 'react'
import { Toaster } from 'sonner'

type Page = 'dashboard' | 'settings'

function App() {
  const [page, setPage] = useState<Page>('dashboard')

  return (
    <ThemeProvider
      defaultTheme="system"
      storageKey="claude-ui-theme"
    >
      <AppContent
        page={page}
        setPage={setPage}
      />
      <Toaster
        position="bottom-right"
        richColors
      />
    </ThemeProvider>
  )
}

/** Inner component so hooks run inside ThemeProvider context */
function AppContent({
  page,
  setPage,
}: {
  page: Page
  setPage: (p: Page) => void
}) {
  // Mount webhook sender globally for startup/on_change triggers
  useWebhookSender()

  return page === 'dashboard' ? (
    <Dashboard onOpenSettings={() => setPage('settings')} />
  ) : (
    <SettingsPage onBack={() => setPage('dashboard')} />
  )
}

export default App
