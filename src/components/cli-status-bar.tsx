import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import type { ClaudeCliState, UsageStats } from "@/lib/types";
import { Activity, AlertTriangle, FileKey, Monitor } from "lucide-react";

interface CliStatusBarProps {
  cliState: ClaudeCliState | null;
  usageStats: UsageStats | null;
  loading: boolean;
}

export function CliStatusBar({
  cliState,
  usageStats,
  loading,
}: CliStatusBarProps) {
  if (loading) {
    return (
      <div className="flex items-center gap-4 rounded-lg border bg-card p-4 animate-pulse">
        <div className="h-5 w-32 bg-muted rounded" />
        <div className="h-5 w-24 bg-muted rounded" />
        <div className="h-5 w-20 bg-muted rounded" />
      </div>
    );
  }

  return (
    <div className="flex flex-wrap items-center gap-3 rounded-lg border bg-linear-to-r from-card to-card/80 p-4">
      {/* Model hiện tại */}
      <div className="flex items-center gap-2">
        <Monitor className="size-4 text-muted-foreground" />
        <span className="text-sm text-muted-foreground">Mẫu (Model):</span>
        <Badge variant="secondary" className="font-mono">
          {cliState?.currentModel || "N/A"}
        </Badge>
      </div>

      <Separator orientation="vertical" className="h-5" />

      {/* Session count */}
      <div className="flex items-center gap-2">
        <Activity className="size-4 text-muted-foreground" />
        <span className="text-sm text-muted-foreground">Phiên:</span>
        <span className="text-sm font-semibold">
          {usageStats?.totalSessions ?? cliState?.sessionCount ?? 0}
        </span>
        {usageStats && usageStats.recentSessions7d > 0 && (
          <span className="text-xs text-muted-foreground">
            ({usageStats.recentSessions7d} tuần này)
          </span>
        )}
      </div>

      <Separator orientation="vertical" className="h-5" />

      {/* .env status */}
      <div className="flex items-center gap-2">
        <FileKey className="size-4 text-muted-foreground" />
        <span className="text-sm text-muted-foreground">.env:</span>
        {cliState?.envFileExists ? (
          <Badge variant="success" className="text-xs">
            Hoạt động ({cliState.activeKeys.length} khóa)
          </Badge>
        ) : (
          <Badge variant="outline" className="text-xs">
            Không tìm thấy
          </Badge>
        )}
      </div>

      {/* Restriction warning */}
      {usageStats?.hasRestrictions && (
        <>
          <Separator orientation="vertical" className="h-5" />
          <div className="flex items-center gap-1.5">
            <AlertTriangle className="size-4 text-foreground" />
            <span className="text-xs text-foreground font-medium">
              Hạn chế đang hiệu lực
            </span>
          </div>
        </>
      )}
    </div>
  );
}
