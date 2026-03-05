import {
  Eye,
  EyeOff,
  Fingerprint,
  KeyRound,
  LoaderCircle,
  LogOut,
} from "lucide-react";
import type { FormEventHandler } from "react";
import type { RestoreAuthStateResponseDto } from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Label } from "@/components/ui/label";
import { UnlockFeedbackAlert } from "@/features/auth/unlock/components/unlock-feedback-alert";
import type { UnlockFeedback } from "@/features/auth/unlock/types";

type UnlockLockedFormProps = {
  restoreState: RestoreAuthStateResponseDto | null;
  biometricSupported: boolean;
  biometricEnabled: boolean;
  canBiometricUnlock: boolean;
  masterPassword: string;
  showPassword: boolean;
  isUnlocking: boolean;
  isBiometricUnlocking: boolean;
  isLoggingOut: boolean;
  canUnlock: boolean;
  feedback: UnlockFeedback;
  onMasterPasswordChange: (value: string) => void;
  onToggleShowPassword: () => void;
  onSubmit: FormEventHandler<HTMLFormElement>;
  onBiometricUnlock: () => void;
  onLogout: () => void;
};

export function UnlockLockedForm({
  restoreState,
  biometricSupported,
  biometricEnabled,
  canBiometricUnlock,
  masterPassword,
  showPassword,
  isUnlocking,
  isBiometricUnlocking,
  isLoggingOut,
  canUnlock,
  feedback,
  onMasterPasswordChange,
  onToggleShowPassword,
  onSubmit,
  onBiometricUnlock,
  onLogout,
}: UnlockLockedFormProps) {
  const isActionBlocked = isUnlocking || isBiometricUnlocking || isLoggingOut;

  return (
    <form className="space-y-5" onSubmit={onSubmit}>
      <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-600">
        <div>登录邮箱：{restoreState?.email ?? "unknown"}</div>
        <div>服务地址：{restoreState?.baseUrl ?? "unknown"}</div>
      </div>
      {biometricSupported && (
        <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">
          Touch ID：{biometricEnabled ? "已启用" : "未启用"}
        </div>
      )}

      <div className="space-y-2">
        <Label htmlFor="unlock-master-password">Master Password</Label>
        <InputGroup>
          <InputGroupAddon>
            <KeyRound className="text-slate-500" />
          </InputGroupAddon>
          <InputGroupInput
            id="unlock-master-password"
            type={showPassword ? "text" : "password"}
            autoComplete="current-password"
            placeholder="输入主密码解锁"
            value={masterPassword}
            onChange={(event) => onMasterPasswordChange(event.target.value)}
            disabled={isActionBlocked}
          />
          <InputGroupAddon align="inline-end" className="px-1.5">
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              className="text-slate-500 hover:text-slate-900"
              onClick={onToggleShowPassword}
              disabled={isActionBlocked}
              aria-label={showPassword ? "隐藏密码" : "显示密码"}
            >
              {showPassword ? <EyeOff /> : <Eye />}
            </Button>
          </InputGroupAddon>
        </InputGroup>
      </div>

      <UnlockFeedbackAlert feedback={feedback} />

      <Button type="submit" size="lg" className="w-full" disabled={!canUnlock}>
        {isUnlocking && <LoaderCircle className="animate-spin" />}
        {isUnlocking ? "正在解锁..." : "解锁密码库"}
      </Button>

      {canBiometricUnlock && (
        <Button
          type="button"
          variant="outline"
          size="lg"
          className="w-full"
          onClick={onBiometricUnlock}
          disabled={isActionBlocked}
        >
          {isBiometricUnlocking ? (
            <LoaderCircle className="animate-spin" />
          ) : (
            <Fingerprint />
          )}
          {isBiometricUnlocking ? "正在等待 Touch ID..." : "使用 Touch ID 解锁"}
        </Button>
      )}

      {biometricSupported && biometricEnabled && !canBiometricUnlock && (
        <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
          Touch ID
          已启用，但当前设备还没有可用于解锁的本地同步数据，请先完成一次同步并用密码解锁。
        </div>
      )}

      <Button
        type="button"
        variant="outline"
        className="w-full"
        disabled={isActionBlocked}
        onClick={onLogout}
      >
        {isLoggingOut ? <LoaderCircle className="animate-spin" /> : <LogOut />}
        {isLoggingOut ? "正在登出..." : "登出"}
      </Button>
    </form>
  );
}
