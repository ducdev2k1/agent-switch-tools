import { invoke } from "@tauri-apps/api/core"
import { useState, useEffect, useCallback } from "react"
import type { CredentialProfile } from "@/lib/types"

export function useCredentialProfiles() {
  const [profiles, setProfiles] = useState<CredentialProfile[]>([])
  const [loading, setLoading] = useState(false)

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const data = await invoke<CredentialProfile[]>("list_credential_profiles")
      setProfiles(data)
    } catch (e) {
      console.error("Failed to load credential profiles:", e)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    load()
  }, [load])

  /**
   * Lưu credential hiện tại thành profile mới
   * Copy .credentials.json → .credentials-[name].json
   */
  const saveCurrentAs = async (name: string) => {
    await invoke("save_current_as_profile", { name })
    await load()
  }

  /**
   * Switch sang profile khác
   * Rename .credentials.json → .credentials-[currentName].json
   * Rename .credentials-[targetName].json → .credentials.json
   */
  const switchTo = async (currentName: string, targetName: string) => {
    await invoke("switch_credential_profile", {
      currentName,
      targetName,
    })
    await load()
  }

  /**
   * Đổi tên profile
   */
  const rename = async (oldName: string, newName: string) => {
    await invoke("rename_credential_profile", { oldName, newName })
    await load()
  }

  /**
   * Xóa profile đã lưu
   */
  const remove = async (name: string) => {
    await invoke("delete_credential_profile", { name })
    await load()
  }

  return {
    profiles,
    loading,
    saveCurrentAs,
    switchTo,
    rename,
    remove,
    refresh: load,
  }
}
