import { AddAccountDialog } from '@/components/add-account-dialog'
import { CliStatusBar } from '@/components/cli-status-bar'
import { DeleteConfirmDialog } from '@/components/delete-confirm-dialog'
import { IdeDashboardSection } from '@/components/ide-dashboard-section'
import { ProfileCard } from '@/components/profile-card'
import { ProfileTable } from '@/components/profile-table'
import { SwitchConfirmationDialog } from '@/components/switch-confirmation-dialog'
import { UsageView } from '@/components/usage/usage-view'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ViewToggle, type ViewMode } from '@/components/view-toggle'
import { useClaudeConfig } from '@/hooks/use-claude-config'
import { useInstalledIdes } from '@/hooks/use-installed-ides'
import { useCredentialProfiles } from '@/hooks/use-profiles'
import { useUsageStats } from '@/hooks/use-usage-stats'
import type { AppUpdaterState, CredentialProfile } from '@/lib/types'
import { listen } from '@tauri-apps/api/event'
import {
  BarChart3,
  Braces,
  MonitorSmartphone,
  MousePointer2,
  Orbit,
  RefreshCw,
  Save,
  Settings,
  Shield,
  Wind,
} from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

interface DashboardProps {
  onOpenSettings: () => void
  updater: AppUpdaterState
}

/** Distinct tab icon per IDE so the icon-only tab bar stays distinguishable. */
function ideIcon(ideType: string): LucideIcon {
  switch (ideType) {
    case 'cursor':
      return MousePointer2
    case 'windsurf':
      return Wind
    case 'antigravity':
      return Orbit
    default:
      return MonitorSmartphone
  }
}

export function Dashboard({ onOpenSettings, updater }: DashboardProps) {
  const { t } = useTranslation()
  const {
    profiles,
    loading: profilesLoading,
    saveCurrentAs,
    switchTo,
    remove,
    checkClaudeRunning,
    refresh,
  } = useCredentialProfiles()

  const {
    state: cliState,
    loading: cliLoading,
    refresh: refreshCli,
  } = useClaudeConfig()

  const { stats: usageStats } = useUsageStats()
  const { updateVersion, installing, install } = updater
  const { ides: installedIdes } = useInstalledIdes()
  const [activeTab, setActiveTab] = useState('claude-code')
  const [viewMode, setViewMode] = useState<ViewMode>('grid')

  const [pendingTrayProfile, setPendingTrayProfile] = useState<string | null>(
    null,
  )

  // Listen for tray quick-switch events
  const handleTraySwitchRef = useCallback(
    async (profileName: string) => {
      const running = await checkClaudeRunning()
      const target = profiles.find((p) => p.name === profileName)
      if (!target) {
        // If profile not found yet (maybe loading), save for later
        if (profiles.length === 0) {
          setPendingTrayProfile(profileName)
        } else {
          toast.error(t('dashboard.errors.not_found', { name: profileName }))
        }
        return
      }
      if (target.isActive) {
        toast.info(
          t('dashboard.messages.already_active', { name: profileName }),
        )
        return
      }
      setClaudeIsRunning(running)
      setSwitchTarget(target)
      setSwitchDialogOpen(true)
    },
    [profiles, checkClaudeRunning, t],
  )

  // Watch for pending tray switch once profiles load
  useEffect(() => {
    if (pendingTrayProfile && profiles.length > 0) {
      handleTraySwitchRef(pendingTrayProfile)
      setPendingTrayProfile(null)
    }
  }, [profiles, pendingTrayProfile, handleTraySwitchRef])

  useEffect(() => {
    const unlisten = listen<string>('tray-switch-profile', (event) => {
      handleTraySwitchRef(event.payload)
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [handleTraySwitchRef])

  // Listen for tray IDE quick-switch events (format: "ideType:profileName")
  useEffect(() => {
    const unlisten = listen<string>('tray-switch-ide-profile', (event) => {
      const colonIdx = event.payload.indexOf(':')
      if (colonIdx > 0) {
        const ideType = event.payload.substring(0, colonIdx)
        setActiveTab(ideType)
        // The IdeDashboardSection will handle the actual switch via its own UI
        toast.info(`Switched to ${ideType} tab. Select profile to switch.`)
      }
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  // Dialog states
  const [switchDialogOpen, setSwitchDialogOpen] = useState(false)
  const [switchTarget, setSwitchTarget] = useState<CredentialProfile | null>(
    null,
  )
  const [claudeIsRunning, setClaudeIsRunning] = useState(false)
  const [switching, setSwitching] = useState(false)
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

  // Initiate switch: check Claude running → show confirmation dialog
  const handleSwitchRequest = async (target: CredentialProfile) => {
    const running = await checkClaudeRunning()
    setClaudeIsRunning(running)
    setSwitchTarget(target)
    setSwitchDialogOpen(true)
  }

  // Confirm switch
  const handleSwitchConfirm = async () => {
    if (!switchTarget) return
    setSwitching(true)
    try {
      const result = await switchTo(switchTarget.name)
      await refreshCli()
      setSwitchDialogOpen(false)
      toast.success(result.message)
      if (result.claudeWasRunning) {
        toast.warning(t('dashboard.messages.restart_claude_warning'), {
          duration: 5000,
        })
      }
    } catch (e) {
      toast.error(t('dashboard.errors.switch_failed', { error: e }))
    } finally {
      setSwitching(false)
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
              <Braces className="size-4" />
            </TabsTrigger>
            {installedIdes
              .filter((ide) => ide.isInstalled)
              .map((ide) => {
                const Icon = ideIcon(ide.ideType)
                return (
                  <TabsTrigger
                    key={ide.ideType}
                    value={ide.ideType}
                    className="px-2.5"
                    title={ide.displayName}
                    aria-label={ide.displayName}
                  >
                    <Icon className="size-4" />
                  </TabsTrigger>
                )
              })}
            <TabsTrigger
              value="usage"
              className="px-2.5"
              title="Usage"
              aria-label="Usage"
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

            <CliStatusBar
              cliState={cliState}
              usageStats={usageStats}
              loading={cliLoading}
            />

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
                      onSwitch={handleSwitchRequest}
                      onDelete={handleDeleteRequest}
                      onProfilesChanged={refresh}
                    />
                  ))}
                </div>
              ) : (
                <ProfileTable
                  profiles={profiles}
                  onSwitch={handleSwitchRequest}
                  onDelete={handleDeleteRequest}
                  onProfilesChanged={refresh}
                />
              )}
            </div>
          </TabsContent>

          {/* IDE tabs — one per installed IDE */}
          {installedIdes
            .filter((ide) => ide.isInstalled)
            .map((ide) => (
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
      <SwitchConfirmationDialog
        open={switchDialogOpen}
        onOpenChange={setSwitchDialogOpen}
        targetProfile={switchTarget}
        claudeIsRunning={claudeIsRunning}
        onConfirm={handleSwitchConfirm}
        switching={switching}
      />

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
