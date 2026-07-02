import { DeleteConfirmDialog } from '@/components/delete-confirm-dialog'
import { IdeProfileCard } from '@/components/ide-profile-card'
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
    restartIde,
    refresh,
  } = useIdeProfiles(ideType)

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

  // Switch immediately without confirmation; auto-restart the IDE if it was running.
  const performSwitch = useCallback(
    async (target: IdeProfile) => {
      try {
        const result = await switchTo(target.name)
        toast.success(result.message)
        if (result.ideWasRunning) {
          toast.info(t('ide.messages.restarting_ide', { ide: ideName }), {
            duration: 3000,
          })
          try {
            const msg = await restartIde()
            toast.success(msg)
          } catch {
            toast.warning(
              t('ide.messages.close_ide_warning', { ide: ideName }),
              { duration: 5000 },
            )
          }
        }
      } catch (e) {
        toast.error(t('ide.errors.switch_failed', { error: e }))
      }
    },
    [switchTo, restartIde, ideName, t],
  )

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

  // Tray already switched this IDE's profile in the backend — just reload the list.
  useEffect(() => {
    const unlisten = listen<string>('tray-ide-profile-switched', (event) => {
      if (event.payload.startsWith(`${ideType}:`)) {
        refresh()
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [ideType, refresh])

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
                onSwitch={performSwitch}
                onDelete={handleDeleteRequest}
              />
            ))}
          </div>
        ) : (
          <IdeProfileTable
            profiles={profiles}
            onSwitch={performSwitch}
            onDelete={handleDeleteRequest}
          />
        )}
      </div>

      <DeleteConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        profileName={deletingName}
        onConfirm={handleDeleteConfirm}
      />
    </div>
  )
}
