import { useState, useEffect, useCallback } from "react"
import { toast } from "sonner"
import { listen } from "@tauri-apps/api/event"
import { useCredentialProfiles } from "@/hooks/use-profiles"
import { useClaudeConfig } from "@/hooks/use-claude-config"
import { useUsageStats } from "@/hooks/use-usage-stats"
import { CliStatusBar } from "@/components/cli-status-bar"
import { ProfileCard } from "@/components/profile-card"
import { SaveProfileDialog } from "@/components/profile-form-dialog"
import { SwitchConfirmationDialog } from "@/components/switch-confirmation-dialog"
import { DeleteConfirmDialog } from "@/components/delete-confirm-dialog"
import { AddAccountDialog } from "@/components/add-account-dialog"
import { Button } from "@/components/ui/button"
import { ModeToggle } from "@/components/mode-toggle"
import { Separator } from "@/components/ui/separator"
import type { CredentialProfile } from "@/lib/types"
import { Save, RefreshCw, Shield, UserPlus } from "lucide-react"

export function Dashboard() {
  const {
    profiles,
    loading: profilesLoading,
    saveCurrentAs,
    switchTo,
    rename,
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
      // Find matching profile from current list
      const target = profiles.find((p) => !p.isActive && p.name === profileName)
      if (!target) {
        toast.error(`Hồ sơ "${profileName}" không tìm thấy`)
        return
      }
      setClaudeIsRunning(running)
      setSwitchTarget(target)
      setSwitchDialogOpen(true)
    },
    [profiles, checkClaudeRunning]
  )

  useEffect(() => {
    const unlisten = listen<string>("tray-switch-profile", (event) => {
      handleTraySwitchRef(event.payload)
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [handleTraySwitchRef])

  // Dialog states
  const [saveDialogOpen, setSaveDialogOpen] = useState(false)
  const [renameDialogOpen, setRenameDialogOpen] = useState(false)
  const [renamingProfile, setRenamingProfile] = useState<CredentialProfile | null>(null)
  const [switchDialogOpen, setSwitchDialogOpen] = useState(false)
  const [switchTarget, setSwitchTarget] = useState<CredentialProfile | null>(null)
  const [claudeIsRunning, setClaudeIsRunning] = useState(false)
  const [switching, setSwitching] = useState(false)
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [deletingName, setDeletingName] = useState<string | null>(null)
  const [addAccountOpen, setAddAccountOpen] = useState(false)

  const activeProfile = profiles.find((p) => p.isActive)
  const savedProfiles = profiles.filter((p) => !p.isActive)

  // Save current credentials as named profile
  const handleSaveCurrent = async (name: string) => {
    try {
      await saveCurrentAs(name)
      toast.success(`Đã lưu thành "${name}"`)
    } catch (e) {
      toast.error(`Lưu thất bại: ${e}`)
      throw e
    }
  }

  // Initiate switch: check Claude running → show confirmation dialog
  const handleSwitchRequest = async (target: CredentialProfile) => {
    if (activeProfile) {
      const hasBackup = savedProfiles.some(
        (p) => p.info.organizationUuid === activeProfile.info.organizationUuid
      )
      if (!hasBackup && activeProfile.name === "Active") {
        setSaveDialogOpen(true)
        toast.info("Vui lòng lưu tài khoản hiện tại trước, sau đó mới chuyển đổi.")
        return
      }
    }

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
        toast.warning("Vui lòng khởi động lại Claude Code để sử dụng thông tin đăng nhập mới.", { duration: 5000 })
      }
    } catch (e) {
      toast.error(`Chuyển đổi thất bại: ${e}`)
    } finally {
      setSwitching(false)
    }
  }

  // Rename
  const handleRename = (profile: CredentialProfile) => {
    setRenamingProfile(profile)
    setRenameDialogOpen(true)
  }

  const handleRenameSubmit = async (newName: string) => {
    if (!renamingProfile) return
    try {
      await rename(renamingProfile.name, newName)
      toast.success(`Đã đổi tên thành "${newName}"`)
    } catch (e) {
      toast.error(`Đổi tên thất bại: ${e}`)
      throw e
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
      toast.success(`Đã xóa "${deletingName}"`)
    } catch (e) {
      toast.error(`Xóa thất bại: ${e}`)
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
              Quản lý Tài khoản Claude
            </h1>
          </div>
          <div className="flex items-center gap-2">
            <ModeToggle />
            <Button variant="ghost" size="icon" onClick={handleRefresh} className="size-8">
              <RefreshCw className="size-4" />
            </Button>
            <Button onClick={() => setSaveDialogOpen(true)} size="sm" disabled={!activeProfile}>
              <Save className="size-4" />
              Lưu Hiện tại
            </Button>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="mx-auto max-w-3xl px-6 py-6 space-y-6">
        <CliStatusBar cliState={cliState} usageStats={usageStats} loading={cliLoading} />

        <Separator />

        {/* Active Account */}
        {activeProfile && (
          <div>
            <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
              Tài khoản đang hoạt động
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
              Không tìm thấy thông tin đăng nhập. Vui lòng đăng nhập vào Claude CLI trước.
            </p>
          </div>
        )}

        <Separator />

        {/* Saved Profiles */}
        <div>
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
              Hồ sơ đã lưu ({savedProfiles.length})
            </h2>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setAddAccountOpen(true)}
              className="text-xs"
            >
              <UserPlus className="size-3.5" />
              Thêm tài khoản
            </Button>
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
              <h3 className="text-base font-semibold mb-1">Không có hồ sơ đã lưu</h3>
              <p className="text-sm text-muted-foreground mb-4 max-w-sm">
                Lưu thông tin đăng nhập hiện tại để tạo hồ sơ.
                Sau đó đăng nhập bằng tài khoản khác và lưu lại.
              </p>
              {activeProfile && (
                <Button onClick={() => setSaveDialogOpen(true)} variant="outline" size="sm">
                  <Save className="size-4" />
                  Lưu Tài khoản Hiện tại
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
                  onRename={handleRename}
                  onDelete={handleDeleteRequest}
                  isCurrentlyActive={
                    activeProfile?.info.organizationUuid === profile.info.organizationUuid
                  }
                />
              ))}
            </div>
          )}
        </div>
      </main>

      {/* Dialogs */}
      <SaveProfileDialog
        open={saveDialogOpen}
        onOpenChange={setSaveDialogOpen}
        onSave={handleSaveCurrent}
        mode="save"
      />

      <SaveProfileDialog
        open={renameDialogOpen}
        onOpenChange={setRenameDialogOpen}
        onSave={handleRenameSubmit}
        mode="rename"
        initialName={renamingProfile?.name}
      />

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
        onSaveCurrent={() => {
          setAddAccountOpen(false)
          setSaveDialogOpen(true)
        }}
      />
    </div>
  )
}
