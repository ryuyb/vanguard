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

type UnlockMethod = "pin" | "masterPassword";

type UnlockLockedFormProps = {
  restoreState: RestoreAuthStateResponseDto | null;
  biometricSupported: boolean;
  biometricEnabled: boolean;
  canBiometricUnlock: boolean;
  canPinUnlock: boolean;
  canUnlock: boolean;
  feedback: UnlockFeedback;
  isBiometricUnlocking: boolean;
  isLoggingOut: boolean;
  isPinUnlocking: boolean;
  isUnlocking: boolean;
  masterPassword: string;
  onBiometricUnlock: () => void;
  onLogout: () => void;
  onMasterPasswordChange: (value: string) => void;
  onPinChange: (value: string) => void;
  onPinUnlock: FormEventHandler<HTMLFormElement>;
  onShowMasterPasswordUnlock: () => void;
  onShowPinUnlock: () => void;
  onSubmit: FormEventHandler<HTMLFormElement>;
  onToggleShowPassword: () => void;
  pin: string;
  pinEnabled: boolean;
  showPassword: boolean;
  unlockMethod: UnlockMethod;
};

export function UnlockLockedForm({
  restoreState,
  biometricSupported,
  biometricEnabled,
  canBiometricUnlock,
  canPinUnlock,
  canUnlock,
  feedback,
  isBiometricUnlocking,
  isLoggingOut,
  isPinUnlocking,
  isUnlocking,
  masterPassword,
  onBiometricUnlock,
  onLogout,
  onMasterPasswordChange,
  onPinChange,
  onPinUnlock,
  onShowMasterPasswordUnlock,
  onShowPinUnlock,
  onSubmit,
  onToggleShowPassword,
  pin,
  pinEnabled,
  showPassword,
  unlockMethod,
}: UnlockLockedFormProps) {
  const isActionBlocked =
    isUnlocking || isPinUnlocking || isBiometricUnlocking || isLoggingOut;
  const isPinMode = unlockMethod === "pin";

  return (
    <div className="space-y-5">
      <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-600">
        <div>登录邮箱：{restoreState?.email ?? "unknown"}</div>
        <div>服务地址：{restoreState?.baseUrl ?? "unknown"}</div>
      </div>

      <form className="space-y-5" onSubmit={isPinMode ? onPinUnlock : onSubmit}>
        {isPinMode ? (
          <div className="space-y-2">
            <Label htmlFor="unlock-pin">PIN</Label>
            <InputGroup>
              <InputGroupAddon>
                <KeyRound className="text-slate-500" />
              </InputGroupAddon>
              <InputGroupInput
                id="unlock-pin"
                type="password"
                inputMode="numeric"
                autoComplete="off"
                placeholder="输入 PIN 解锁"
                value={pin}
                onChange={(event) => onPinChange(event.target.value)}
                disabled={isActionBlocked}
              />
            </InputGroup>
          </div>
        ) : (
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
        )}

        {isPinMode && !pinEnabled && (
          <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
            当前账号未启用 PIN 解锁，请切换到 master password 解锁。
          </div>
        )}

        <UnlockFeedbackAlert feedback={feedback} />

        <Button
          type="submit"
          size="lg"
          className="w-full"
          disabled={isPinMode ? !canPinUnlock : !canUnlock}
        >
          {(isPinMode ? isPinUnlocking : isUnlocking) && (
            <LoaderCircle className="animate-spin" />
          )}
          {isPinMode
            ? isPinUnlocking
              ? "正在使用 PIN 解锁..."
              : "使用 PIN 解锁"
            : isUnlocking
              ? "正在解锁..."
              : "解锁密码库"}
        </Button>
      </form>

      {pinEnabled && (
        <Button
          type="button"
          variant="ghost"
          className="w-full"
          disabled={isActionBlocked}
          onClick={isPinMode ? onShowMasterPasswordUnlock : onShowPinUnlock}
        >
          {isPinMode ? "改用 Master Password 解锁" : "改用 PIN 解锁"}
        </Button>
      )}

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
          {isBiometricUnlocking
            ? "正在等待生物识别验证..."
            : "使用生物识别解锁"}
        </Button>
      )}

      {biometricSupported && biometricEnabled && !canBiometricUnlock && (
        <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
          生物识别
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
    </div>
  );
}
