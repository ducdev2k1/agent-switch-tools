import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Trash2 } from "lucide-react"

interface DeleteConfirmDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  profileName: string | null
  onConfirm: () => void
}

export function DeleteConfirmDialog({
  open,
  onOpenChange,
  profileName,
  onConfirm,
}: DeleteConfirmDialogProps) {
  if (!profileName) return null

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent onClose={() => onOpenChange(false)}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Trash2 className="size-5 text-destructive" />
            Xóa "{profileName}"?
          </DialogTitle>
        </DialogHeader>

        <p className="text-sm text-muted-foreground">
          Thao tác này sẽ xóa vĩnh viễn tệp thông tin đăng nhập đã lưu. Hành động này không thể hoàn tác.
        </p>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Hủy
          </Button>
          <Button variant="destructive" onClick={onConfirm}>
            Xóa
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
