import { useState } from "react"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

interface SaveProfileDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSave: (name: string) => Promise<void>
  /** Nếu có initialName = đang rename, không phải tạo mới */
  mode: "save" | "rename"
  initialName?: string
}

export function SaveProfileDialog({
  open,
  onOpenChange,
  onSave,
  mode,
  initialName = "",
}: SaveProfileDialogProps) {
  const [name, setName] = useState(initialName)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState("")

  // Reset form khi mở dialog
  const handleOpenChange = (isOpen: boolean) => {
    if (isOpen) {
      setName(initialName)
      setError("")
    }
    onOpenChange(isOpen)
  }

  const validate = (): boolean => {
    const trimmed = name.trim()
    if (!trimmed) {
      setError("Name is required")
      return false
    }
    // Không cho phép ký tự đặc biệt trong tên file
    if (/[\/\\:*?"<>|]/.test(trimmed)) {
      setError("Name cannot contain special characters: / \\ : * ? \" < > |")
      return false
    }
    if (trimmed.toLowerCase() === "active") {
      setError("Cannot use 'Active' as profile name")
      return false
    }
    setError("")
    return true
  }

  const handleSave = async () => {
    if (!validate()) return

    setSaving(true)
    try {
      await onSave(name.trim())
      onOpenChange(false)
    } catch (e) {
      setError(String(e))
    } finally {
      setSaving(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent onClose={() => handleOpenChange(false)}>
        <DialogHeader>
          <DialogTitle>
            {mode === "save" ? "Save Current Account" : "Rename Profile"}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="profile-name">Profile Name</Label>
            <Input
              id="profile-name"
              placeholder='e.g. "Work", "Personal", "Client"'
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSave()}
              autoFocus
            />
            {error && (
              <p className="text-xs text-destructive">{error}</p>
            )}
            {mode === "save" && (
              <p className="text-xs text-muted-foreground">
                This will save your current active credentials as a named profile.
                You can switch back to it later.
              </p>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => handleOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            {saving
              ? "Saving..."
              : mode === "save"
                ? "Save Profile"
                : "Rename"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
