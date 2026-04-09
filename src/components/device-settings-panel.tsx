import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useDeviceInfo } from '@/hooks/use-device-info'
import { Check, Copy, Save } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

export function DeviceSettingsPanel() {
  const { t } = useTranslation()
  const { deviceInfo, loading, error, renameDevice } = useDeviceInfo()

  const [name, setName] = useState('')
  const [saving, setSaving] = useState(false)
  const [copiedId, setCopiedId] = useState(false)

  // Sync local state when device info loads
  useEffect(() => {
    if (deviceInfo) setName(deviceInfo.deviceName)
  }, [deviceInfo])

  const hasChanges = deviceInfo != null && name.trim() !== deviceInfo.deviceName

  const handleSave = useCallback(async () => {
    const trimmed = name.trim()
    if (!trimmed) return
    setSaving(true)
    try {
      await renameDevice(trimmed)
      toast.success(t('settings.device.saved'))
    } catch (e) {
      toast.error(t('settings.device.save_failed', { error: String(e) }))
    } finally {
      setSaving(false)
    }
  }, [name, renameDevice, t])

  const handleCopyId = useCallback(() => {
    if (!deviceInfo) return
    navigator.clipboard.writeText(deviceInfo.deviceId)
    setCopiedId(true)
    toast.success(t('settings.device.copied'))
    setTimeout(() => setCopiedId(false), 2000)
  }, [deviceInfo, t])

  if (loading) {
    return (
      <div className="text-sm text-muted-foreground">
        {t('common.labels.loading')}
      </div>
    )
  }

  if (error) {
    return (
      <div className="text-sm text-destructive">
        {error}
      </div>
    )
  }

  if (!deviceInfo) return null

  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 px-1">
          {t('settings.device.title')}
        </h3>
        <div className="rounded-lg border bg-card">
          {/* Device ID — read-only + copy */}
          <div className="flex items-center justify-between px-4 py-3">
            <div className="min-w-0 flex-1">
              <Label className="text-xs text-muted-foreground">
                {t('settings.device.device_id')}
              </Label>
              <p className="text-sm font-mono truncate mt-0.5">
                {deviceInfo.deviceId}
              </p>
            </div>
            <button
              type="button"
              onClick={handleCopyId}
              className="ml-2 p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted transition-colors cursor-pointer shrink-0"
            >
              {copiedId ? (
                <Check className="size-3.5" />
              ) : (
                <Copy className="size-3.5" />
              )}
            </button>
          </div>

          <div className="border-t" />

          {/* Hostname — read-only */}
          <div className="px-4 py-3">
            <Label className="text-xs text-muted-foreground">
              {t('settings.device.hostname')}
            </Label>
            <p className="text-sm font-mono mt-0.5">{deviceInfo.hostname}</p>
          </div>

          <div className="border-t" />

          {/* Device Name — editable */}
          <div className="px-4 py-3 space-y-1.5">
            <Label className="text-xs text-muted-foreground">
              {t('settings.device.device_name')}
            </Label>
            <div className="flex items-center gap-2">
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={t('settings.device.device_name_placeholder')}
                className="h-8 text-sm flex-1"
              />
              <Button
                variant="outline"
                size="sm"
                onClick={handleSave}
                disabled={!hasChanges || saving}
                className="shrink-0"
              >
                <Save className="size-3.5" />
                {t('settings.device.save')}
              </Button>
            </div>
          </div>
        </div>

        <p className="text-[11px] text-muted-foreground mt-2 px-1">
          {t('settings.device.description')}
        </p>
      </div>
    </div>
  )
}
