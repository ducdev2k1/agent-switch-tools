import { AddAccountDialog } from '@/components/add-account-dialog'
import { CliStatusBar } from '@/components/cli-status-bar'
import { DeleteConfirmDialog } from '@/components/delete-confirm-dialog'
import { ModeToggle } from '@/components/mode-toggle'
import { ProfileCard } from '@/components/profile-card'
import { SwitchConfirmationDialog } from '@/components/switch-confirmation-dialog'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { useClaudeConfig } from '@/hooks/use-claude-config'
import { useCredentialProfiles } from '@/hooks/use-profiles'
import { useUsageStats } from '@/hooks/use-usage-stats'
import type { CredentialProfile } from '@/lib/types'
import { listen } from '@tauri-apps/api/event'
import { RefreshCw, Save, Shield, UserPlus } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

export function Dashboard() {
  const { t, i18n } = useTranslation()
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

  // Listen for tray quick-switch events
  const handleTraySwitchRef = useCallback(
    async (profileName: string) => {
      const running = await checkClaudeRunning()
      const target = profiles.find((p) => !p.isActive && p.name === profileName)
      if (!target) {
        toast.error(t('dashboard.errors.not_found', { name: profileName }))
        return
      }
      setClaudeIsRunning(running)
      setSwitchTarget(target)
      setSwitchDialogOpen(true)
    },
    [profiles, checkClaudeRunning, t],
  )

  useEffect(() => {
    const unlisten = listen<string>('tray-switch-profile', (event) => {
      handleTraySwitchRef(event.payload)
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [handleTraySwitchRef])

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
  const savedProfiles = profiles.filter((p) => !p.isActive)

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
            <Button
              variant="ghost"
              size="sm"
              onClick={() =>
                i18n.changeLanguage(i18n.language === 'vi' ? 'en' : 'vi')
              }
              className="text-xs font-medium"
            >
              {i18n.language === 'vi' ? 'EN' : 'VI'}
            </Button>
            <ModeToggle />
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
      </header>

      {/* Main content */}
      <main className="mx-auto max-w-3xl px-6 py-6 space-y-6">
        <CliStatusBar
          cliState={cliState}
          usageStats={usageStats}
          loading={cliLoading}
        />

        <Separator />

        {/* Active Account */}
        {activeProfile && (
          <div>
            <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
              {t('common.labels.active_account')}
            </h2>
            <ProfileCard
              profile={activeProfile}
              onSwitch={() => {}}
              onDelete={() => {}}
            />
          </div>
        )}

        {!activeProfile && !profilesLoading && (
          <div className="rounded-lg border border-amber-500/30 bg-amber-500/5 p-4 text-center">
            <p className="text-sm text-amber-700 dark:text-amber-400">
              {t('dashboard.messages.no_credentials')}
            </p>
          </div>
        )}

        <Separator />

        {/* Saved Profiles */}
        <div>
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
              {t('common.labels.saved_profiles', {
                count: savedProfiles.length,
              })}
            </h2>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setAddAccountOpen(true)}
              className="text-xs"
            >
              <UserPlus className="size-3.5" />
              {t('common.actions.add_account')}
            </Button>
          </div>

          {profilesLoading ? (
            <div className="space-y-3">
              {[1, 2].map((i) => (
                <div
                  key={i}
                  className="h-20 rounded-xl border bg-card animate-pulse"
                />
              ))}
            </div>
          ) : savedProfiles.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <Shield className="size-10 text-muted-foreground/40 mb-3" />
              <h3 className="text-base font-semibold mb-1">
                {t('dashboard.messages.no_saved_profiles')}
              </h3>
              <p className="text-sm text-muted-foreground mb-4 max-w-sm">
                {t('dashboard.messages.no_saved_profiles_info')}
              </p>
              {activeProfile && (
                <Button
                  onClick={handleSaveCurrent}
                  variant="outline"
                  size="sm"
                  disabled={saving}
                >
                  <Save className="size-4" />
                  {t('common.actions.save_current')}
                </Button>
              )}
            </div>
          ) : (
            <div className="space-y-3">
              {savedProfiles.map((profile) => (
                <ProfileCard
                  key={profile.name}
                  profile={profile}
                  onSwitch={handleSwitchRequest}
                  onDelete={handleDeleteRequest}
                  isCurrentlyActive={
                    !!activeProfile?.oauthAccount?.accountUuid &&
                    activeProfile.oauthAccount.accountUuid ===
                      profile.oauthAccount?.accountUuid
                  }
                />
              ))}
            </div>
          )}
        </div>
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
