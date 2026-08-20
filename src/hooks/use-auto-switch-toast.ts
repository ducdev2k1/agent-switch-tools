import { listen } from '@tauri-apps/api/event'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

interface SwitchPerformedPayload {
  from: string
  to: string
  utilization: number
}

interface SwitchExhaustedPayload {
  profile: string
  utilization: number
}

/**
 * Surfaces Auto Switch Rule outcomes as toasts. Mounted at the app root, not in
 * the settings panel: the rule fires from a background worker, so the user is
 * almost never on that panel when it happens.
 */
export function useAutoSwitchToast() {
  const { t } = useTranslation()

  useEffect(() => {
    const performed = listen<SwitchPerformedPayload>(
      'auto-switch-performed',
      ({ payload }) => {
        toast.success(
          t('settings.auto_switch.toast.switched', {
            from: payload.from,
            to: payload.to,
            utilization: Math.round(payload.utilization),
          }),
        )
      },
    )
    const exhausted = listen<SwitchExhaustedPayload>(
      'auto-switch-exhausted',
      ({ payload }) => {
        toast.warning(
          t('settings.auto_switch.toast.exhausted', {
            profile: payload.profile,
            utilization: Math.round(payload.utilization),
          }),
        )
      },
    )
    return () => {
      performed.then((fn) => fn())
      exhausted.then((fn) => fn())
    }
  }, [t])
}
