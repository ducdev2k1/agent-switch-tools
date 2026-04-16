import { DeleteConfirmDialog } from '@/components/delete-confirm-dialog'
import { IdeProfileCard } from '@/components/ide-profile-card'
import { SwitchConfirmationDialog } from '@/components/switch-confirmation-dialog'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { useIdeProfiles } from '@/hooks/use-ide-profiles'
import type { IdeProfile, IdeType } from '@/lib/types'
import { Monitor, RefreshCw, Save, Shield } from 'lucide-react'
import { useCallback, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

interface IdeDashboardSectionProps {
  ideType: IdeType
  ideName: string
}

export function IdeDashboardSection({
  ideType,
  ideName,
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
  const savedProfiles = profiles.filter((p) => !p.isActive)

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

      {/* Active Account */}
      {activeProfile && (
        <div>
          <h2 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
            {t('common.labels.active_account')}
          </h2>
          <IdeProfileCard
            profile={activeProfile}
            onSwitch={() => {}}
            onDelete={() => {}}
          />
        </div>
      )}

      {!activeProfile && !loading && (
        <div className="rounded-lg border border-amber-500/30 bg-amber-500/5 p-4 text-center">
          <p className="text-sm text-amber-700 dark:text-amber-400">
            {t('ide.errors.not_logged_in', { ide: ideName })}
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
        </div>

        {loading ? (
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
              {t('ide.labels.no_profiles')}
            </h3>
            <p className="text-sm text-muted-foreground mb-4 max-w-sm">
              {t('ide.labels.no_profiles_info')}
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
              <IdeProfileCard
                key={profile.name}
                profile={profile}
                onSwitch={handleSwitchRequest}
                onDelete={handleDeleteRequest}
              />
            ))}
          </div>
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
