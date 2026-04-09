import type { DeviceInfo } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useState } from 'react'

export function useDeviceInfo() {
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    invoke<DeviceInfo>('get_device_info')
      .then(setDeviceInfo)
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false))
  }, [])

  const renameDevice = useCallback(async (name: string) => {
    const updated = await invoke<DeviceInfo>('rename_device', { name })
    setDeviceInfo(updated)
    return updated
  }, [])

  return { deviceInfo, loading, error, renameDevice }
}
