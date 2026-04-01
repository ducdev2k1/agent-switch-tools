import { useState } from "react"
import { open as shellOpen } from "@tauri-apps/plugin-shell"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Terminal, ExternalLink, CircleCheck } from "lucide-react"

interface AddAccountDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  hasActiveProfile: boolean
  onSaveCurrent: () => void
}

const steps = [
  {
    label: "Lưu tài khoản hiện tại",
    description: 'Bấm "Lưu Hiện tại" để backup account đang dùng trước.',
  },
  {
    label: "Đăng xuất trong terminal",
    description: "Chạy lệnh bên dưới để đăng xuất tài khoản hiện tại.",
    command: "claude auth logout",
  },
  {
    label: "Đăng nhập tài khoản mới",
    description: "Chạy Claude Code, nó sẽ yêu cầu đăng nhập lại.",
    command: "claude",
  },
  {
    label: "Quay lại app & lưu",
    description: 'Bấm "Lưu Hiện tại" để lưu tài khoản mới vừa đăng nhập.',
  },
]

export function AddAccountDialog({
  open,
  onOpenChange,
  hasActiveProfile,
  onSaveCurrent,
}: AddAccountDialogProps) {
  const [copiedIdx, setCopiedIdx] = useState<number | null>(null)

  const copyCommand = async (cmd: string, idx: number) => {
    await navigator.clipboard.writeText(cmd)
    setCopiedIdx(idx)
    setTimeout(() => setCopiedIdx(null), 2000)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent onClose={() => onOpenChange(false)} className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Terminal className="size-5" />
            Thêm tài khoản mới
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          {steps.map((step, i) => (
            <div key={i} className="flex gap-3">
              <div className="shrink-0 mt-0.5">
                <Badge variant="outline" className="size-6 justify-center p-0 text-xs">
                  {i + 1}
                </Badge>
              </div>
              <div className="flex-1 space-y-1.5">
                <p className="text-sm font-medium">{step.label}</p>
                <p className="text-xs text-muted-foreground">{step.description}</p>

                {/* Save current button for step 1 */}
                {i === 0 && hasActiveProfile && (
                  <Button
                    variant="outline"
                    size="sm"
                    className="text-xs mt-1"
                    onClick={onSaveCurrent}
                  >
                    Lưu Hiện tại
                  </Button>
                )}

                {/* Command block with copy */}
                {step.command && (
                  <div className="flex items-center gap-2 mt-1">
                    <code className="flex-1 rounded bg-muted px-2 py-1 text-xs font-mono">
                      {step.command}
                    </code>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="text-xs h-7 px-2"
                      onClick={() => copyCommand(step.command!, i)}
                    >
                      {copiedIdx === i ? (
                        <CircleCheck className="size-3.5 text-emerald-500" />
                      ) : (
                        "Copy"
                      )}
                    </Button>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>

        <DialogFooter className="flex-row gap-2 sm:justify-between">
          <Button
            variant="outline"
            size="sm"
            onClick={() => shellOpen("https://console.anthropic.com")}
          >
            <ExternalLink className="size-3.5" />
            Anthropic Console
          </Button>
          <Button onClick={() => onOpenChange(false)}>Đã hiểu</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
