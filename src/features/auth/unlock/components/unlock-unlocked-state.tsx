import { Link } from "@tanstack/react-router";
import { LoaderCircle, LogOut } from "lucide-react";
import type { RestoreAuthStateResponseDto } from "@/bindings";
import { Button } from "@/components/ui/button";

type UnlockUnlockedStateProps = {
  restoreState: RestoreAuthStateResponseDto | null;
  biometricSupported: boolean;
  biometricEnabled: boolean;
  isLoggingOut: boolean;
  onLogout: () => void;
};

export function UnlockUnlockedState({
  restoreState,
  biometricSupported,
  biometricEnabled,
  isLoggingOut,
  onLogout,
}: UnlockUnlockedStateProps) {
  return (
    <div className="space-y-4">
      <div className="rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
        当前 Vault 已是解锁状态，无需再次输入 master password。
      </div>
      {restoreState?.status === "locked" && (
        <div className="rounded-lg border border-blue-200 bg-blue-50 px-3 py-2 text-sm text-blue-800">
          当前仅恢复了本地解锁状态，后端登录会话尚未恢复。
        </div>
      )}
      {biometricSupported && biometricEnabled && (
        <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">
          Touch ID：已启用
        </div>
      )}
      <Button asChild className="w-full">
        <Link to="/vault">查看 Vault 数据</Link>
      </Button>
      <Button
        type="button"
        variant="outline"
        className="w-full"
        disabled={isLoggingOut}
        onClick={onLogout}
      >
        {isLoggingOut ? <LoaderCircle className="animate-spin" /> : <LogOut />}
        {isLoggingOut ? "正在登出..." : "登出"}
      </Button>
    </div>
  );
}
