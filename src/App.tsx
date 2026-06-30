import { ThemeProvider } from '@/components/theme-provider'
import { UpdateNotificationDialog } from '@/components/update-notification-dialog'
import { useAppUpdater } from '@/hooks/use-app-updater'
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

  // App updater — shared between modal (auto) and settings (manual)
  const updater = useAppUpdater()

  return (
    <>
      {page === 'dashboard' ? (
        <Dashboard onOpenSettings={() => setPage('settings')} updater={updater} />
      ) : (
        <SettingsPage onBack={() => setPage('dashboard')} updater={updater} />
      )}

      {/* Update notification modal — shows once per new version */}
      {updater.updateVersion && (
        <UpdateNotificationDialog
          open={updater.showModal}
          onDismiss={updater.dismissModal}
          version={updater.updateVersion}
          body={updater.updateBody}
          installing={updater.installing}
          onInstall={updater.install}
        />
      )}
    </>
  )
}

export default App
