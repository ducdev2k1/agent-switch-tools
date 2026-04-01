import { Dashboard } from "@/pages/dashboard"
import { Toaster } from "sonner"
import { ThemeProvider } from "@/components/theme-provider"

function App() {
  return (
    <ThemeProvider defaultTheme="system" storageKey="claude-ui-theme">
      <Dashboard />
      <Toaster position="bottom-right" richColors />
    </ThemeProvider>
  )
}

export default App
