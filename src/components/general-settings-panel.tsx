import { useTheme } from '@/components/theme-provider'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { useAutoUpdateConfig } from '@/hooks/use-auto-update-config'
import { useAutoStartConfig } from '@/hooks/use-autostart-config'
import { useStartMinimizedConfig } from '@/hooks/use-start-minimized-config'
import type { AppUpdaterState } from '@/lib/types'
import { Download, RefreshCw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

interface GeneralSettingsPanelProps {
  updater: AppUpdaterState
}

export function GeneralSettingsPanel({ updater }: GeneralSettingsPanelProps) {
  const { t, i18n } = useTranslation()
  const { theme, setTheme } = useTheme()
  const {
    enabled: autoStartEnabled,
    loading: autoStartLoading,
    toggle: toggleAutoStart,
  } = useAutoStartConfig()
  const {
    enabled: startMinimizedEnabled,
    loading: startMinimizedLoading,
    toggle: toggleStartMinimized,
  } = useStartMinimizedConfig()
  const { enabled: autoUpdateEnabled, setEnabled: setAutoUpdate } =
    useAutoUpdateConfig()

  const handleCheckForUpdates = async () => {
    const version = await updater.checkForUpdates()
    if (version) {
      toast.success(t('settings.general.update_available', { version }))
    } else {
      toast.info(t('settings.general.up_to_date'))
    }
  }

  return (
    <div className="space-y-6">
      {/* Appearance */}
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 px-1">
          {t('settings.general.appearance')}
        </h3>
        <div className="rounded-lg border bg-card">
          {/* Theme */}
          <div className="flex items-center justify-between px-4 py-3">
            <Label className="text-sm">{t('settings.general.theme')}</Label>
            <Select
              value={theme}
              onValueChange={(v) => setTheme(v as 'light' | 'dark' | 'system')}
            >
              <SelectTrigger className="w-40 h-8 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="light">
                  {t('settings.general.theme_light')}
                </SelectItem>
                <SelectItem value="dark">
                  {t('settings.general.theme_dark')}
                </SelectItem>
                <SelectItem value="system">
                  {t('settings.general.theme_system')}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="border-t" />

          {/* Language */}
          <div className="flex items-center justify-between px-4 py-3">
            <Label className="text-sm">{t('settings.general.language')}</Label>
            <Select
              value={i18n.language}
              onValueChange={(v) => i18n.changeLanguage(v)}
            >
              <SelectTrigger className="w-40 h-8 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="en">English</SelectItem>
                <SelectItem value="vi">Tiếng Việt</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </div>

      {/* Startup */}
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 px-1">
          {t('settings.general.startup')}
        </h3>
        <div className="rounded-lg border bg-card">
          <div className="flex items-center justify-between px-4 py-3">
            <div>
              <Label className="text-sm">
                {t('settings.general.autostart')}
              </Label>
              <p className="text-xs text-muted-foreground mt-0.5">
                {t('settings.general.autostart_desc')}
              </p>
            </div>
            <Switch
              checked={autoStartEnabled}
              onCheckedChange={toggleAutoStart}
              disabled={autoStartLoading}
            />
          </div>

          <div className="border-t" />

          <div className="flex items-center justify-between px-4 py-3">
            <div>
              <Label className="text-sm">
                {t('settings.general.start_minimized')}
              </Label>
              <p className="text-xs text-muted-foreground mt-0.5">
                {t('settings.general.start_minimized_desc')}
              </p>
            </div>
            <Switch
              checked={startMinimizedEnabled}
              onCheckedChange={toggleStartMinimized}
              disabled={startMinimizedLoading}
            />
          </div>
        </div>
      </div>

      {/* Updates */}
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 px-1">
          {t('settings.general.updates')}
        </h3>
        <div className="rounded-lg border bg-card">
          {/* Auto-check toggle */}
          <div className="flex items-center justify-between px-4 py-3">
            <Label className="text-sm">
              {t('settings.general.auto_update')}
            </Label>
            <Switch
              checked={autoUpdateEnabled}
              onCheckedChange={setAutoUpdate}
            />
          </div>

          <div className="border-t" />

          {/* Manual check + install */}
          <div className="flex items-center justify-between px-4 py-3">
            <div>
              <Label className="text-sm">
                {t('settings.general.check_updates')}
              </Label>
              {updater.updateVersion && (
                <p className="text-xs text-blue-600 dark:text-blue-400 mt-0.5">
                  {t('settings.general.update_available', {
                    version: updater.updateVersion,
                  })}
                </p>
              )}
            </div>
            <div className="flex items-center gap-2">
              {updater.updateVersion && (
                <Button
                  size="sm"
                  variant="default"
                  onClick={updater.install}
                  disabled={updater.installing}
                  className="h-7 text-xs"
                >
                  <Download className="size-3.5" />
                  {updater.installing
                    ? t('update_dialog.installing')
                    : t('update_dialog.install_now')}
                </Button>
              )}
              <Button
                size="sm"
                variant="outline"
                onClick={handleCheckForUpdates}
                disabled={updater.checking}
                className="h-7 text-xs"
              >
                <RefreshCw
                  className={`size-3.5 ${updater.checking ? 'animate-spin' : ''}`}
                />
                {t('settings.general.check_now')}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
