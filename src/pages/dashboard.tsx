import { AddAccountDialog } from '@/components/add-account-dialog'
import { DeleteConfirmDialog } from '@/components/delete-confirm-dialog'
import { IdeDashboardSection } from '@/components/ide-dashboard-section'
import { ProfileCard } from '@/components/profile-card'
import { ProfileTable } from '@/components/profile-table'
import { IdeLogo } from '@/components/ide-logo'
import { UsageView } from '@/components/usage/usage-view'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ViewToggle, type ViewMode } from '@/components/view-toggle'
import { useClaudeConfig } from '@/hooks/use-claude-config'
import { useInstalledIdes } from '@/hooks/use-installed-ides'
import { useCredentialProfiles } from '@/hooks/use-profiles'
import type { AppUpdaterState, CredentialProfile, IdeType } from '@/lib/types'
import { listen } from '@tauri-apps/api/event'
import {
  BarChart3,
  Braces,
  RefreshCw,
  Save,
  Settings,
  Shield,
} from 'lucide-react'
import { useCallback, useEffect, useMemo, useState } from 'react'

/** Antigravity ships as three variants; we group them under one top-level tab. */
const ANTIGRAVITY_VARIANTS: IdeType[] = [
  'antigravity',
  'antigravity-ide',
  'antigravity-cli',
]
const ANTIGRAVITY_GROUP = 'antigravity-group'
/** Temporarily hidden top-level IDE tabs (remove an entry to re-enable). */
const HIDDEN_IDES: IdeType[] = ['cursor', 'windsurf']
/** Short labels for the variant sub-tabs (the parent tab already says "Antigravity"). */
const ANTIGRAVITY_SUBLABEL: Record<string, string> = {
  antigravity: 'Desktop',
  'antigravity-ide': 'IDE',
  'antigravity-cli': 'CLI',
}
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

interface DashboardProps {
  onOpenSettings: () => void
  updater: AppUpdaterState
}

export function Dashboard({ onOpenSettings, updater }: DashboardProps) {
  const { t } = useTranslation()
  const {
    profiles,
    loading: profilesLoading,
    saveCurrentAs,
    switchTo,
    remove,
    refresh,
  } = useCredentialProfiles()

  const { refresh: refreshCli } = useClaudeConfig()
  const { updateVersion, installing, install } = updater
  const { ides: installedIdes } = useInstalledIdes()
  const [activeTab, setActiveTab] = useState('claude-code')
  const [viewMode, setViewMode] = useState<ViewMode>('grid')

  // Split installed agents: Antigravity variants live under one grouped tab; the rest stay top-level.
  const installedAntigravity = useMemo(
    () =>
      installedIdes.filter(
        (ide) => ide.isInstalled && ANTIGRAVITY_VARIANTS.includes(ide.ideType),
      ),
    [installedIdes],
  )
  const installedOther = useMemo(
    () =>
      installedIdes.filter(
        (ide) =>
          ide.isInstalled &&
          !ANTIGRAVITY_VARIANTS.includes(ide.ideType) &&
          !HIDDEN_IDES.includes(ide.ideType),
      ),
    [installedIdes],
  )
  const [antigravitySubTab, setAntigravitySubTab] = useState<string>('')

  // Default the Antigravity sub-tab to the first installed variant.
  useEffect(() => {
    if (
      installedAntigravity.length > 0 &&
      !installedAntigravity.some((ide) => ide.ideType === antigravitySubTab)
    ) {
      setAntigravitySubTab(installedAntigravity[0].ideType)
    }
  }, [installedAntigravity, antigravitySubTab])

  // Switch immediately without confirmation; warn afterwards if Claude was running.
  const performSwitch = useCallback(
    async (target: CredentialProfile) => {
      try {
        const result = await switchTo(target.name)
        await refreshCli()
        toast.success(result.message)
        if (result.claudeWasRunning) {
          toast.warning(t('dashboard.messages.restart_claude_warning'), {
            duration: 5000,
          })
        }
      } catch (e) {
        toast.error(t('dashboard.errors.switch_failed', { error: e }))
      }
    },
    [switchTo, refreshCli, t],
  )

  // Tray already performed the switch in the backend — just refresh and notify.
  useEffect(() => {
    const unlistenSwitched = listen<string>(
      'tray-profile-switched',
      async (event) => {
        toast.success(t('dashboard.success.switched', { name: event.payload }))
        await Promise.all([refresh(), refreshCli()])
      },
    )
    const unlistenError = listen<string>('tray-switch-error', (event) => {
      toast.error(t('dashboard.errors.switch_failed', { error: event.payload }))
    })
    return () => {
      unlistenSwitched.then((fn) => fn())
      unlistenError.then((fn) => fn())
    }
  }, [refresh, refreshCli, t])

  // Tray IDE switch done in backend (format: "ideType:profileName") — open the matching tab.
  useEffect(() => {
    const unlisten = listen<string>('tray-ide-profile-switched', (event) => {
      const colonIdx = event.payload.indexOf(':')
      if (colonIdx > 0) {
        const ideType = event.payload.substring(0, colonIdx) as IdeType
        const name = event.payload.substring(colonIdx + 1)
        // Antigravity variants live under the grouped tab + a variant sub-tab.
        if (ANTIGRAVITY_VARIANTS.includes(ideType)) {
          setActiveTab(ANTIGRAVITY_GROUP)
          setAntigravitySubTab(ideType)
        } else {
          setActiveTab(ideType)
        }
        toast.success(t('ide.messages.switch_success', { name, ide: ideType }))
      }
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [t])

  // Dialog states
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [deletingName, setDeletingName] = useState<string | null>(null)
  const [addAccountOpen, setAddAccountOpen] = useState(false)
  const [saving, setSaving] = useState(false)

  const activeProfile = profiles.find((p) => p.isActive)

  // Save current credentials (auto-detect email)
  const handleSaveCurrent = async () => {
    setSaving(true)
    try {
      const email = await saveCurrentAs()
      toast.success(t('dashboard.success.account_saved', { email }))
    } catch (e) {
      toast.error(t('dashboard.errors.save_failed', { error: e }))
    } finally {
      setSaving(false)
    }
  }

  // Delete — show confirmation dialog
  const handleDeleteRequest = (name: string) => {
    setDeletingName(name)
    setDeleteDialogOpen(true)
  }

  const handleDeleteConfirm = async () => {
    if (!deletingName) return
    try {
      await remove(deletingName)
      toast.success(t('dashboard.success.deleted', { name: deletingName }))
    } catch (e) {
      toast.error(t('dashboard.errors.delete_failed', { error: e }))
    } finally {
      setDeleteDialogOpen(false)
      setDeletingName(null)
    }
  }

  // Refresh all
  const handleRefresh = async () => {
    await Promise.all([refresh(), refreshCli()])
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-40 border-b bg-background/95 backdrop-blur supports-backdrop-filter:bg-background/60">
        <div className="flex h-14 items-center justify-between px-6">
          <div className="flex items-center gap-3">
            <Shield className="size-5 text-primary" />
            <h1 className="text-lg font-bold tracking-tight">
              {t('dashboard.header.title')}
            </h1>
          </div>
          <div className="flex items-center gap-2">
            <ViewToggle
              mode={viewMode}
              onChange={setViewMode}
            />
            <Button
              variant="ghost"
              size="icon"
              onClick={onOpenSettings}
              className="size-8"
              title={t('settings.title')}
            >
              <Settings className="size-4" />
            </Button>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="mx-auto max-w-5xl px-6 py-6 space-y-6">
        {/* Update available banner */}
        {updateVersion && (
          <div className="flex items-center justify-between rounded-lg border border-blue-500/30 bg-blue-500/5 px-4 py-2.5 text-sm">
            <span className="text-blue-700 dark:text-blue-400">
              Update available:{' '}
              <span className="font-bold">v{updateVersion}</span>
            </span>
            <Button
              size="sm"
              variant="outline"
              onClick={install}
              disabled={installing}
              className="h-7 text-xs border-blue-500/40 text-blue-700 dark:text-blue-400 hover:bg-blue-500/10"
            >
              {installing ? 'Installing...' : 'Install & Restart'}
            </Button>
          </div>
        )}

        {/* IDE Tabs */}
        <Tabs
          value={activeTab}
          onValueChange={setActiveTab}
        >
          <TabsList className="w-full justify-start">
            <TabsTrigger
              value="claude-code"
              className="px-2.5"
              title="Claude Code"
              aria-label="Claude Code"
            >
              <IdeLogo
                name="claude"
                className="size-4"
              />
            </TabsTrigger>
            {installedOther.map((ide) => (
              <TabsTrigger
                key={ide.ideType}
                value={ide.ideType}
                className="px-2.5"
                title={ide.displayName}
                aria-label={ide.displayName}
              >
                <IdeLogo
                  name={ide.ideType}
                  className="size-4"
                />
              </TabsTrigger>
            ))}
            {installedAntigravity.length > 0 && (
              <TabsTrigger
                value={ANTIGRAVITY_GROUP}
                className="px-2.5"
                title="Antigravity"
                aria-label="Antigravity"
              >
                <IdeLogo
                  name="antigravity"
                  className="size-4"
                />
              </TabsTrigger>
            )}
            <TabsTrigger
              value="usage"
              className="px-2.5"
              title={t('usage.tab')}
              aria-label={t('usage.tab')}
            >
              <BarChart3 className="size-4" />
            </TabsTrigger>
          </TabsList>

          {/* Claude Code tab */}
          <TabsContent
            value="claude-code"
            className="mt-4 space-y-6"
          >
            {/* Action bar for Claude Code */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Braces className="size-4" />
                <span className="font-medium">Claude Code</span>
              </div>
              <div className="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={handleRefresh}
                  className="size-8"
                >
                  <RefreshCw className="size-4" />
                </Button>
                <Button
                  onClick={handleSaveCurrent}
                  size="sm"
                  disabled={!activeProfile || saving}
                >
                  <Save className="size-4" />
                  {saving
                    ? t('common.actions.saving')
                    : t('common.actions.save_current')}
                </Button>
              </div>
            </div>

            <Separator />

            {/* Unified Account List */}
            <div className="space-y-6">
              {profilesLoading ? (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
                  {[1, 2, 3].map((i) => (
                    <div
                      key={i}
                      className="h-48 rounded-2xl border bg-card/50 animate-pulse"
                    />
                  ))}
                </div>
              ) : profiles.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-20 text-center">
                  <Shield className="size-16 text-muted-foreground/20 mb-4" />
                  <h3 className="text-xl font-bold mb-2">
                    {t('dashboard.labels.no_profiles')}
                  </h3>
                  <p className="text-muted-foreground max-w-md mx-auto">
                    {t('dashboard.labels.no_profiles_info')}
                  </p>
                </div>
              ) : viewMode === 'grid' ? (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
                  {profiles.map((profile) => (
                    <ProfileCard
                      key={profile.name}
                      profile={profile}
                      onSwitch={performSwitch}
                      onDelete={handleDeleteRequest}
                      onProfilesChanged={refresh}
                    />
                  ))}
                </div>
              ) : (
                <ProfileTable
                  profiles={profiles}
                  onSwitch={performSwitch}
                  onDelete={handleDeleteRequest}
                  onProfilesChanged={refresh}
                />
              )}
            </div>
          </TabsContent>

          {/* Non-Antigravity IDE tabs — one per installed IDE */}
          {installedOther.map((ide) => (
            <TabsContent
              key={ide.ideType}
              value={ide.ideType}
              className="mt-4"
            >
              <IdeDashboardSection
                ideType={ide.ideType}
                ideName={ide.displayName}
                viewMode={viewMode}
              />
            </TabsContent>
          ))}

          {/* Antigravity grouped tab — nested sub-tabs per variant (Desktop / IDE / CLI) */}
          {installedAntigravity.length > 0 && (
            <TabsContent
              value={ANTIGRAVITY_GROUP}
              className="mt-4"
            >
              <Tabs
                value={antigravitySubTab}
                onValueChange={setAntigravitySubTab}
              >
                <TabsList>
                  {installedAntigravity.map((ide) => (
                    <TabsTrigger
                      key={ide.ideType}
                      value={ide.ideType}
                      className="text-xs px-3"
                    >
                      {ANTIGRAVITY_SUBLABEL[ide.ideType] ?? ide.displayName}
                    </TabsTrigger>
                  ))}
                </TabsList>
                {installedAntigravity.map((ide) => (
                  <TabsContent
                    key={ide.ideType}
                    value={ide.ideType}
                    className="mt-4"
                  >
                    <IdeDashboardSection
                      ideType={ide.ideType}
                      ideName={ide.displayName}
                      viewMode={viewMode}
                    />
                  </TabsContent>
                ))}
              </Tabs>
            </TabsContent>
          )}

          {/* Usage / cost analytics tab */}
          <TabsContent
            value="usage"
            className="mt-4"
          >
            <UsageView />
          </TabsContent>
        </Tabs>
      </main>

      {/* Dialogs */}
      <DeleteConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        profileName={deletingName}
        onConfirm={handleDeleteConfirm}
      />

      <AddAccountDialog
        open={addAccountOpen}
        onOpenChange={setAddAccountOpen}
        hasActiveProfile={!!activeProfile}
        onSaveCurrent={handleSaveCurrent}
      />
    </div>
  )
}
