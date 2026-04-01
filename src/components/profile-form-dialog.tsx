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
      setError("Vui lòng nhập tên")
      return false
    }
    // Không cho phép ký tự đặc biệt trong tên file
    if (/[\/\\:*?"<>|]/.test(trimmed)) {
      setError("Tên không được chứa các ký tự đặc biệt: / \\ : * ? \" < > |")
      return false
    }
    if (trimmed.toLowerCase() === "active") {
      setError("Không thể sử dụng 'Active' làm tên hồ sơ")
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
            {mode === "save" ? "Lưu Tài khoản Hiện tại" : "Đổi tên Hồ sơ"}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="profile-name">Tên Hồ sơ</Label>
            <Input
              id="profile-name"
              placeholder='VD: "Công việc", "Cá nhân", "Khách hàng"'
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
                Thao tác này sẽ lưu thông tin đăng nhập hiện tại của bạn thành một hồ sơ có tên.
                Bạn có thể chuyển đổi lại sau này.
              </p>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => handleOpenChange(false)}>
            Hủy
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            {saving
              ? "Đang lưu..."
              : mode === "save"
                ? "Lưu Hồ sơ"
                : "Đổi tên"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
