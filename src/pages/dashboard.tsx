import { useState } from "react"
import { useCredentialProfiles } from "@/hooks/use-profiles"
import { useClaudeConfig } from "@/hooks/use-claude-config"
import { useUsageStats } from "@/hooks/use-usage-stats"
import { CliStatusBar } from "@/components/cli-status-bar"
import { ProfileCard } from "@/components/profile-card"
import { SaveProfileDialog } from "@/components/profile-form-dialog"
import { Button } from "@/components/ui/button"
import { ModeToggle } from "@/components/mode-toggle"
import { Separator } from "@/components/ui/separator"
import type { CredentialProfile } from "@/lib/types"
import { Save, RefreshCw, Shield } from "lucide-react"

export function Dashboard() {
  const {
    profiles,
    loading: profilesLoading,
    saveCurrentAs,
    switchTo,
    rename,
    remove,
    refresh,
  } = useCredentialProfiles()

  const {
    state: cliState,
    loading: cliLoading,
    refresh: refreshCli,
  } = useClaudeConfig()

  const { stats: usageStats } = useUsageStats()

  const [saveDialogOpen, setSaveDialogOpen] = useState(false)
  const [renameDialogOpen, setRenameDialogOpen] = useState(false)
  const [renamingProfile, setRenamingProfile] = useState<CredentialProfile | null>(null)
  const [switching, setSwitching] = useState<string | null>(null)

  const activeProfile = profiles.find((p) => p.isActive)
  const savedProfiles = profiles.filter((p) => !p.isActive)

  // Lưu credential hiện tại thành profile mới
  const handleSaveCurrent = async (name: string) => {
    await saveCurrentAs(name)
  }

  // Switch sang profile khác
  const handleSwitch = async (target: CredentialProfile) => {
    if (!activeProfile) return

    // Cần hỏi tên để lưu active hiện tại (nếu chưa có saved copy)
    const hasBackup = savedProfiles.some(
      (p) => p.info.organizationUuid === activeProfile.info.organizationUuid
    )

    if (!hasBackup) {
      // Active chưa được lưu → cần đặt tên trước
      const promptName = window.prompt(
        "Save current active credentials as (enter a name):",
        "Default"
      )
      if (!promptName) return
      await saveCurrentAs(promptName)
      // Refresh để lấy tên vừa lưu
      await refresh()
    }

    setSwitching(target.name)
    try {
      // Tìm tên backup của active hiện tại
      const currentBackupName =
        savedProfiles.find(
          (p) => p.info.organizationUuid === activeProfile.info.organizationUuid
        )?.name || "Default"

      await switchTo(currentBackupName, target.name)
      await refreshCli()
    } catch (e) {
      console.error("Failed to switch:", e)
      alert(`Switch failed: ${e}`)
    } finally {
      setSwitching(null)
    }
  }

  // Rename profile
  const handleRename = (profile: CredentialProfile) => {
    setRenamingProfile(profile)
    setRenameDialogOpen(true)
  }

  const handleRenameSubmit = async (newName: string) => {
    if (!renamingProfile) return
    await rename(renamingProfile.name, newName)
  }

  // Delete profile
  const handleDelete = async (name: string) => {
    if (window.confirm(`Delete profile "${name}"? This will remove the saved credentials file.`)) {
      await remove(name)
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
              Claude Account Manager
            </h1>
          </div>
          <div className="flex items-center gap-2">
            <ModeToggle />
            <Button variant="ghost" size="icon" onClick={handleRefresh} className="size-8">
              <RefreshCw className="size-4" />
            </Button>
            <Button onClick={() => setSaveDialogOpen(true)} size="sm" disabled={!activeProfile}>
              <Save className="size-4" />
              Save Current
            </Button>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="mx-auto max-w-3xl px-6 py-6 space-y-6">
        {/* CLI Status Bar */}
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
              Active Account
            </h2>
            <ProfileCard
              profile={activeProfile}
              onSwitch={() => {}}
              onRename={() => {}}
              onDelete={() => {}}
            />
          </div>
        )}

        {!activeProfile && !profilesLoading && (
          <div className="rounded-lg border border-amber-500/30 bg-amber-500/5 p-4 text-center">
            <p className="text-sm text-amber-700 dark:text-amber-400">
              No active credentials found. Log in to Claude CLI first.
            </p>
          </div>
        )}

        <Separator />

        {/* Saved Profiles */}
        <div>
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
              Saved Profiles ({savedProfiles.length})
            </h2>
          </div>

          {profilesLoading ? (
            <div className="space-y-3">
              {[1, 2].map((i) => (
                <div key={i} className="h-20 rounded-xl border bg-card animate-pulse" />
              ))}
            </div>
          ) : savedProfiles.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <Shield className="size-10 text-muted-foreground/40 mb-3" />
              <h3 className="text-base font-semibold mb-1">No saved profiles</h3>
              <p className="text-sm text-muted-foreground mb-4 max-w-sm">
                Save your current credentials to create a named profile.
                Then log in with a different account and save that too.
              </p>
              {activeProfile && (
                <Button onClick={() => setSaveDialogOpen(true)} variant="outline" size="sm">
                  <Save className="size-4" />
                  Save Current Account
                </Button>
              )}
            </div>
          ) : (
            <div className="space-y-3">
              {savedProfiles.map((profile) => (
                <ProfileCard
                  key={profile.name}
                  profile={profile}
                  onSwitch={handleSwitch}
                  onRename={handleRename}
                  onDelete={handleDelete}
                />
              ))}
            </div>
          )}
        </div>

        {/* Switching indicator */}
        {switching && (
          <div className="fixed bottom-6 right-6 rounded-lg border bg-card px-4 py-3 shadow-lg flex items-center gap-2 z-50">
            <RefreshCw className="size-4 animate-spin" />
            <span className="text-sm">Switching to {switching}...</span>
          </div>
        )}
      </main>

      {/* Save Current Dialog */}
      <SaveProfileDialog
        open={saveDialogOpen}
        onOpenChange={setSaveDialogOpen}
        onSave={handleSaveCurrent}
        mode="save"
      />

      {/* Rename Dialog */}
      <SaveProfileDialog
        open={renameDialogOpen}
        onOpenChange={setRenameDialogOpen}
        onSave={handleRenameSubmit}
        mode="rename"
        initialName={renamingProfile?.name}
      />
    </div>
  )
}
