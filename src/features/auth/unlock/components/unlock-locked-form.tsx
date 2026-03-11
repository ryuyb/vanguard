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
    <div className="space-y-6">
      <div className="space-y-2 rounded-xl border border-slate-200/60 bg-slate-50/50 px-4 py-3.5">
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium text-slate-500">账户</span>
          <span className="text-slate-700">
            {restoreState?.email ?? "unknown"}
          </span>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium text-slate-500">服务器</span>
          <span className="text-slate-700">
            {restoreState?.baseUrl ?? "unknown"}
          </span>
        </div>
      </div>

      <form className="space-y-5" onSubmit={isPinMode ? onPinUnlock : onSubmit}>
        {isPinMode ? (
          <div className="space-y-2.5">
            <Label
              htmlFor="unlock-pin"
              className="text-sm font-medium text-slate-700"
            >
              PIN 码
            </Label>
            <InputGroup>
              <InputGroupAddon>
                <KeyRound className="h-5 w-5 text-slate-400" />
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
                className="h-12 text-base"
              />
            </InputGroup>
          </div>
        ) : (
          <div className="space-y-2.5">
            <Label
              htmlFor="unlock-master-password"
              className="text-sm font-medium text-slate-700"
            >
              主密码
            </Label>
            <InputGroup>
              <InputGroupAddon>
                <KeyRound className="h-5 w-5 text-slate-400" />
              </InputGroupAddon>
              <InputGroupInput
                id="unlock-master-password"
                type={showPassword ? "text" : "password"}
                autoComplete="current-password"
                placeholder="输入主密码解锁"
                value={masterPassword}
                onChange={(event) => onMasterPasswordChange(event.target.value)}
                disabled={isActionBlocked}
                className="h-12 text-base"
              />
              <InputGroupAddon align="inline-end" className="px-1.5">
                <Button
                  type="button"
                  variant="ghost"
                  size="icon-sm"
                  className="text-slate-400 hover:text-slate-700 transition-colors"
                  onClick={onToggleShowPassword}
                  disabled={isActionBlocked}
                  aria-label={showPassword ? "隐藏密码" : "显示密码"}
                >
                  {showPassword ? (
                    <EyeOff className="h-5 w-5" />
                  ) : (
                    <Eye className="h-5 w-5" />
                  )}
                </Button>
              </InputGroupAddon>
            </InputGroup>
          </div>
        )}

        <UnlockFeedbackAlert feedback={feedback} />

        <Button
          type="submit"
          size="lg"
          className="h-12 w-full bg-blue-600 text-base font-medium hover:bg-blue-700 transition-colors"
          disabled={isPinMode ? !canPinUnlock : !canUnlock}
        >
          {(isPinMode ? isPinUnlocking : isUnlocking) && (
            <LoaderCircle className="h-5 w-5 animate-spin" />
          )}
          {isPinMode
            ? isPinUnlocking
              ? "正在解锁..."
              : "使用 PIN 解锁"
            : isUnlocking
              ? "正在解锁..."
              : "解锁"}
        </Button>
      </form>

      {pinEnabled && (
        <Button
          type="button"
          variant="ghost"
          className="w-full text-sm text-slate-600 hover:text-slate-900 transition-colors"
          disabled={isActionBlocked}
          onClick={isPinMode ? onShowMasterPasswordUnlock : onShowPinUnlock}
        >
          {isPinMode ? "改用主密码解锁" : "改用 PIN 解锁"}
        </Button>
      )}

      {canBiometricUnlock && (
        <Button
          type="button"
          variant="outline"
          size="lg"
          className="h-12 w-full border-slate-300 text-base font-medium hover:bg-slate-50 transition-colors"
          onClick={onBiometricUnlock}
          disabled={isActionBlocked}
        >
          {isBiometricUnlocking ? (
            <LoaderCircle className="h-5 w-5 animate-spin" />
          ) : (
            <Fingerprint className="h-5 w-5" />
          )}
          {isBiometricUnlocking ? "正在验证..." : "使用生物识别"}
        </Button>
      )}

      {biometricSupported && biometricEnabled && !canBiometricUnlock && (
        <div className="rounded-xl border border-amber-200/60 bg-amber-50/50 px-4 py-3 text-sm text-amber-900">
          <p className="font-medium">生物识别不可用</p>
          <p className="mt-1 text-amber-800">
            当前设备还没有可用于解锁的本地数据，请先用密码解锁并完成同步。
          </p>
        </div>
      )}

      <div className="pt-2 border-t border-slate-200">
        <Button
          type="button"
          variant="ghost"
          className="w-full text-sm text-slate-600 hover:text-red-600 transition-colors"
          disabled={isActionBlocked}
          onClick={onLogout}
        >
          {isLoggingOut ? (
            <LoaderCircle className="h-4 w-4 animate-spin" />
          ) : (
            <LogOut className="h-4 w-4" />
          )}
          {isLoggingOut ? "正在登出..." : "登出账户"}
        </Button>
      </div>
    </div>
  );
}
