import { useTheme } from '@/components/theme-provider'
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
import { useTranslation } from 'react-i18next'

export function GeneralSettingsPanel() {
  const { t, i18n } = useTranslation()
  const { theme, setTheme } = useTheme()
  const { enabled: autoUpdateEnabled, setEnabled: setAutoUpdate } =
    useAutoUpdateConfig()

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

      {/* Updates */}
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 px-1">
          {t('settings.general.updates')}
        </h3>
        <div className="rounded-lg border bg-card">
          <div className="flex items-center justify-between px-4 py-3">
            <Label className="text-sm">
              {t('settings.general.auto_update')}
            </Label>
            <Switch
              checked={autoUpdateEnabled}
              onCheckedChange={setAutoUpdate}
            />
          </div>
        </div>
      </div>
    </div>
  )
}
