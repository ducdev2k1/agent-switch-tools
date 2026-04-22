import { DeleteConfirmDialog } from '@/components/delete-confirm-dialog'
import { IdeProfileCard } from '@/components/ide-profile-card'
import { SwitchConfirmationDialog } from '@/components/switch-confirmation-dialog'
import { Button } from '@/components/ui/button'
import { useIdeProfiles } from '@/hooks/use-ide-profiles'
import type { IdeProfile, IdeType } from '@/lib/types'
import { listen } from '@tauri-apps/api/event'
import { Monitor, RefreshCw, Save, Shield } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import { IdeProfileTable } from '@/components/ide-profile-table'
import type { ViewMode } from '@/components/view-toggle'

interface IdeDashboardSectionProps {
  ideType: IdeType
  ideName: string
  viewMode: ViewMode
}

export function IdeDashboardSection({
  ideType,
  ideName,
  viewMode,
}: IdeDashboardSectionProps) {
  const { t } = useTranslation()
  const {
    profiles,
    loading,
    saveCurrentAs,
    switchTo,
    remove,
    checkIdeRunning,
    restartIde,
    refresh,
  } = useIdeProfiles(ideType)

  const [switchDialogOpen, setSwitchDialogOpen] = useState(false)
  const [switchTarget, setSwitchTarget] = useState<IdeProfile | null>(null)
  const [ideIsRunning, setIdeIsRunning] = useState(false)
  const [switching, setSwitching] = useState(false)
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [deletingName, setDeletingName] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  const activeProfile = profiles.find((p) => p.isActive)

  const handleSaveCurrent = useCallback(async () => {
    setSaving(true)
    try {
      const email = await saveCurrentAs()
      toast.success(t('ide.messages.save_success', { ide: ideName, email }))
    } catch (e) {
      toast.error(t('ide.errors.save_failed', { error: e }))
    } finally {
      setSaving(false)
    }
  }, [saveCurrentAs, ideName, t])

  const handleSwitchRequest = useCallback(
    async (target: IdeProfile) => {
      const running = await checkIdeRunning()
      setIdeIsRunning(running)
      setSwitchTarget(target)
      setSwitchDialogOpen(true)
    },
    [checkIdeRunning],
  )

  const handleSwitchConfirm = useCallback(async () => {
    if (!switchTarget) return
    setSwitching(true)
    try {
      const result = await switchTo(switchTarget.name)
      setSwitchDialogOpen(false)
      toast.success(result.message)
      if (result.ideWasRunning) {
        // Auto-restart IDE after switching
        toast.info(t('ide.messages.restarting_ide', { ide: ideName }), {
          duration: 3000,
        })
        try {
          const msg = await restartIde()
          toast.success(msg)
        } catch (e) {
          toast.warning(t('ide.messages.close_ide_warning', { ide: ideName }), {
            duration: 5000,
          })
        }
      }
    } catch (e) {
      toast.error(t('ide.errors.switch_failed', { error: e }))
    } finally {
      setSwitching(false)
    }
  }, [switchTarget, switchTo, restartIde, ideName, t])

  const handleDeleteRequest = (name: string) => {
    setDeletingName(name)
    setDeleteDialogOpen(true)
  }

  const handleDeleteConfirm = async () => {
    if (!deletingName) return
    try {
      await remove(deletingName)
      toast.success(
        t('ide.messages.delete_success', { ide: ideName, name: deletingName }),
      )
    } catch (e) {
      toast.error(String(e))
    } finally {
      setDeleteDialogOpen(false)
      setDeletingName(null)
    }
  }

  // Listen for tray switch events for this specific IDE
  useEffect(() => {
    const unlisten = listen<string>(
      'tray-switch-ide-profile',
      async (event) => {
        const colonIdx = event.payload.indexOf(':')
        if (colonIdx > 0) {
          const type = event.payload.substring(0, colonIdx)
          const name = event.payload.substring(colonIdx + 1)

          if (type === ideType) {
            const target = profiles.find((p: any) => p.name === name)
            if (target) {
              if (target.isActive) {
                toast.info(
                  t('ide.messages.already_active', { ide: ideName, name }),
                )
                return
              }
              const running = await checkIdeRunning()
              setIdeIsRunning(running)
              setSwitchTarget(target)
              setSwitchDialogOpen(true)
            }
          }
        }
      },
    )

    return () => {
      unlisten.then((fn: any) => fn())
    }
  }, [ideType, profiles, ideName, t, checkIdeRunning])

  return (
    <div className="space-y-6">
      {/* IDE header bar */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Monitor className="size-4" />
          <span className="font-medium">{ideName}</span>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={refresh}
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

      {/* Unified IDE Account List */}
      <div className="space-y-6">
        {loading ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
            {[1, 2, 3].map((i) => (
              <div
                key={i}
                className="h-48 rounded-2xl border bg-card animate-pulse"
              />
            ))}
          </div>
        ) : profiles.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 text-center">
            <Shield className="size-16 text-muted-foreground/20 mb-4" />
            <h3 className="text-xl font-bold mb-2">
              {t('ide.labels.no_profiles')}
            </h3>
            <p className="text-muted-foreground max-w-md mx-auto">
              {t('ide.labels.no_profiles_info')}
            </p>
          </div>
        ) : viewMode === 'grid' ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
            {profiles.map((profile) => (
              <IdeProfileCard
                key={profile.name}
                profile={profile}
                onSwitch={handleSwitchRequest}
                onDelete={handleDeleteRequest}
              />
            ))}
          </div>
        ) : (
          <IdeProfileTable
            profiles={profiles}
            onSwitch={handleSwitchRequest}
            onDelete={handleDeleteRequest}
          />
        )}
      </div>

      {/* Reuse existing dialogs — they work with any profile shape */}
      {switchTarget && (
        <SwitchConfirmationDialog
          open={switchDialogOpen}
          onOpenChange={setSwitchDialogOpen}
          targetProfile={{
            name: switchTarget.name,
            isActive: false,
            info: {
              subscriptionType: switchTarget.membershipType,
              rateLimitTier: null,
              expiresAt: null,
              isExpired: false,
              expiresInHours: null,
              scopes: [],
              organizationUuid: null,
            },
            oauthAccount: switchTarget.email
              ? {
                  accountUuid: null,
                  emailAddress: switchTarget.email,
                  organizationUuid: null,
                  hasExtraUsageEnabled: null,
                  billingType: null,
                  accountCreatedAt: null,
                  subscriptionCreatedAt: null,
                  displayName: switchTarget.displayName,
                  organizationRole: null,
                  workspaceRole: null,
                  organizationName: null,
                }
              : null,
          }}
          claudeIsRunning={ideIsRunning}
          onConfirm={handleSwitchConfirm}
          switching={switching}
        />
      )}

      <DeleteConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        profileName={deletingName}
        onConfirm={handleDeleteConfirm}
      />
    </div>
  )
}
