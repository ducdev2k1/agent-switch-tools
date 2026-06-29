import { AutoSessionView } from '@/components/auto-session/auto-session-view'
import { DeviceSettingsPanel } from '@/components/device-settings-panel'
import { GeneralSettingsPanel } from '@/components/general-settings-panel'
import { SessionUsageWebhookPanel } from '@/components/session-usage-webhook-panel'
import { WebhookSettingsPanel } from '@/components/webhook-settings-panel'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import type { AppUpdaterState } from '@/lib/types'
import { ArrowLeft, Info, Settings, Timer, Webhook } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface SettingsPageProps {
  onBack: () => void
  updater: AppUpdaterState
}

export function SettingsPage({ onBack, updater }: SettingsPageProps) {
  const { t } = useTranslation()

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-40 border-b bg-background/95 backdrop-blur supports-backdrop-filter:bg-background/60">
        <div className="flex h-14 items-center gap-3 px-6">
          <Button
            variant="ghost"
            size="icon"
            onClick={onBack}
            className="size-8"
          >
            <ArrowLeft className="size-4" />
          </Button>
          <h1 className="text-lg font-bold tracking-tight">
            {t('settings.title')}
          </h1>
        </div>
      </header>

      {/* Content: sidebar tabs + panel */}
      <div className="mx-auto max-w-5xl px-6 py-6">
        <Tabs
          defaultValue="general"
          orientation="vertical"
          className="flex gap-6"
        >
          {/* Sidebar — sticky so it stays visible when content scrolls */}
          <TabsList className="sticky top-20 flex h-auto w-48 shrink-0 flex-col items-stretch gap-1 self-start bg-transparent p-0">
            <TabsTrigger
              value="general"
              className="justify-start gap-2 px-3 py-2 data-[state=active]:bg-accent"
            >
              <Settings className="size-4" />
              {t('settings.tabs.general')}
            </TabsTrigger>
            <TabsTrigger
              value="auto-session"
              className="justify-start gap-2 px-3 py-2 data-[state=active]:bg-accent"
            >
              <Timer className="size-4" />
              {t('settings.tabs.auto_session')}
            </TabsTrigger>
            <TabsTrigger
              value="webhook"
              className="justify-start gap-2 px-3 py-2 data-[state=active]:bg-accent"
            >
              <Webhook className="size-4" />
              {t('settings.tabs.webhook')}
            </TabsTrigger>
            <TabsTrigger
              value="about"
              className="justify-start gap-2 px-3 py-2 data-[state=active]:bg-accent"
            >
              <Info className="size-4" />
              {t('settings.tabs.about')}
            </TabsTrigger>
          </TabsList>

          {/* Panels */}
          <div className="flex-1 min-w-0">
            <TabsContent
              value="general"
              className="mt-0"
            >
              <GeneralSettingsPanel updater={updater} />
            </TabsContent>

            <TabsContent
              value="auto-session"
              className="mt-0"
            >
              <AutoSessionView />
            </TabsContent>

            <TabsContent
              value="webhook"
              className="mt-0 space-y-6"
            >
              <DeviceSettingsPanel />
              <WebhookSettingsPanel />
              <SessionUsageWebhookPanel />
            </TabsContent>

            <TabsContent
              value="about"
              className="mt-0"
            >
              <AboutPanel />
            </TabsContent>
          </div>
        </Tabs>
      </div>
    </div>
  )
}

/** About tab: app info, author, license, links */
function AboutPanel() {
  const { t } = useTranslation()

  return (
    <div className="space-y-4">
      <div>
        <h2 className="text-base font-semibold mb-2">Agent Switch Tools</h2>
        <p className="text-sm text-muted-foreground">
          {t('settings.about.description')}
        </p>
      </div>
      <div className="space-y-2 text-sm">
        <div className="flex gap-2">
          <span className="text-muted-foreground w-28 shrink-0">
            {t('settings.about.version')}:
          </span>
          <span className="font-mono">v{__APP_VERSION__}</span>
        </div>
        <div className="flex gap-2">
          <span className="text-muted-foreground w-28 shrink-0">
            {t('settings.about.author')}:
          </span>
          <span>ducdev2k1</span>
        </div>
        <div className="flex gap-2">
          <span className="text-muted-foreground w-28 shrink-0">
            {t('settings.about.license')}:
          </span>
          <span>MIT</span>
        </div>
        <div className="flex gap-2">
          <span className="text-muted-foreground w-28 shrink-0">
            {t('settings.about.source_code')}:
          </span>
          <a
            href="https://github.com/ducdev2k1/agent-switch-tools"
            target="_blank"
            rel="noopener noreferrer"
            className="text-primary hover:underline"
          >
            GitHub
          </a>
        </div>
      </div>
    </div>
  )
}
