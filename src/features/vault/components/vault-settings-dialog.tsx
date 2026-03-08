import { Shield, SlidersHorizontal, X } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { commands } from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { toErrorText } from "@/features/auth/shared/utils";

type VaultSettingsDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

type SettingsSection = "security" | "general";
type LanguageOption = "zh" | "en";
type RequireMasterPasswordOption = "1d" | "7d" | "14d" | "30d" | "never";
type AutoLockIdleOption =
  | "1m"
  | "2m"
  | "5m"
  | "10m"
  | "15m"
  | "30m"
  | "1h"
  | "4h"
  | "8h"
  | "never";
type ClipboardClearOption =
  | "10s"
  | "20s"
  | "30s"
  | "1m"
  | "2m"
  | "5m"
  | "never";

const AUTO_LOCK_IDLE_OPTIONS: Array<{
  value: AutoLockIdleOption;
  label: string;
}> = [
  { value: "1m", label: "1 分钟" },
  { value: "2m", label: "2 分钟" },
  { value: "5m", label: "5 分钟" },
  { value: "10m", label: "10 分钟" },
  { value: "15m", label: "15 分钟" },
  { value: "30m", label: "30 分钟" },
  { value: "1h", label: "1 小时" },
  { value: "4h", label: "4 小时" },
  { value: "8h", label: "8 小时" },
  { value: "never", label: "从不" },
];

const REQUIRE_MASTER_PASSWORD_OPTIONS: Array<{
  value: RequireMasterPasswordOption;
  label: string;
}> = [
  { value: "1d", label: "1 天后" },
  { value: "7d", label: "7 天后" },
  { value: "14d", label: "14 天后" },
  { value: "30d", label: "30 天后" },
  { value: "never", label: "从不" },
];

const CLIPBOARD_CLEAR_OPTIONS: Array<{
  value: ClipboardClearOption;
  label: string;
}> = [
  { value: "10s", label: "10 秒" },
  { value: "20s", label: "20 秒" },
  { value: "30s", label: "30 秒" },
  { value: "1m", label: "1 分钟" },
  { value: "2m", label: "2 分钟" },
  { value: "5m", label: "5 分钟" },
  { value: "never", label: "从不" },
];

const LANGUAGE_OPTIONS: Array<{ value: LanguageOption; label: string }> = [
  { value: "zh", label: "中文" },
  { value: "en", label: "英语" },
];

const QUICK_ACCESS_SHORTCUT = "⌃⇧Space";
const LOCK_SHORTCUT = "⇧⌘L";

function formatShortcutFromKeyboardEvent(event: {
  key: string;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
}): string | null {
  if (
    event.key === "Shift" ||
    event.key === "Control" ||
    event.key === "Alt" ||
    event.key === "Meta"
  ) {
    return null;
  }

  const modifiers: string[] = [];
  if (event.ctrlKey) {
    modifiers.push("⌃");
  }
  if (event.shiftKey) {
    modifiers.push("⇧");
  }
  if (event.altKey) {
    modifiers.push("⌥");
  }
  if (event.metaKey) {
    modifiers.push("⌘");
  }

  const keyMap: Record<string, string> = {
    " ": "Space",
    Spacebar: "Space",
    Escape: "Esc",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
  };
  const keyLabel =
    keyMap[event.key] ??
    (event.key.length === 1 ? event.key.toUpperCase() : event.key);

  return `${modifiers.join("")}${keyLabel}`;
}

export function VaultSettingsDialog({
  open,
  onOpenChange,
}: VaultSettingsDialogProps) {
  const [activeSection, setActiveSection] =
    useState<SettingsSection>("general");
  const [language, setLanguage] = useState<LanguageOption>("zh");
  const [showWebsiteIcon, setShowWebsiteIcon] = useState(true);
  const [quickAccessShortcut, setQuickAccessShortcut] = useState(
    QUICK_ACCESS_SHORTCUT,
  );
  const [lockShortcut, setLockShortcut] = useState(LOCK_SHORTCUT);
  const [isQuickAccessCapturing, setIsQuickAccessCapturing] = useState(false);
  const [isLockCapturing, setIsLockCapturing] = useState(false);
  const [requireMasterPasswordAfter, setRequireMasterPasswordAfter] =
    useState<RequireMasterPasswordOption>("never");
  const [launchOnLogin, setLaunchOnLogin] = useState(false);
  const [lockWhenDeviceSleep, setLockWhenDeviceSleep] = useState(false);
  const [autoLockIdleDelay, setAutoLockIdleDelay] =
    useState<AutoLockIdleOption>("never");
  const [clipboardClearAfter, setClipboardClearAfter] =
    useState<ClipboardClearOption>("never");
  const [isStatusLoading, setIsStatusLoading] = useState(false);
  const [isBiometricSupported, setIsBiometricSupported] = useState<
    boolean | null
  >(null);
  const [isBiometricEnabled, setIsBiometricEnabled] = useState(false);
  const [isPinSupported, setIsPinSupported] = useState(false);
  const [isPinEnabled, setIsPinEnabled] = useState(false);
  const [isBiometricBusy, setIsBiometricBusy] = useState(false);
  const [isPinBusy, setIsPinBusy] = useState(false);
  const [isPinDialogOpen, setIsPinDialogOpen] = useState(false);
  const [pinInput, setPinInput] = useState("");
  const [pinDialogError, setPinDialogError] = useState<string | null>(null);
  const [errorText, setErrorText] = useState<string | null>(null);
  const quickAccessOriginalShortcutRef = useRef(quickAccessShortcut);
  const lockOriginalShortcutRef = useRef(lockShortcut);
  const quickAccessCapturedRef = useRef(false);
  const lockCapturedRef = useRef(false);

  const loadSecuritySettings = useCallback(async () => {
    setIsStatusLoading(true);
    setErrorText(null);
    setIsBiometricSupported(null);
    try {
      const [biometricResult, pinResult] = await Promise.all([
        commands.vaultGetBiometricStatus(),
        commands.vaultGetPinStatus(),
      ]);

      if (biometricResult.status === "ok") {
        setIsBiometricSupported(biometricResult.data.supported);
        setIsBiometricEnabled(biometricResult.data.enabled);
      } else {
        setErrorText(
          toErrorText(biometricResult.error, "读取生物识别状态失败。"),
        );
      }

      if (pinResult.status === "ok") {
        setIsPinSupported(pinResult.data.supported);
        setIsPinEnabled(pinResult.data.enabled);
      } else {
        setErrorText(toErrorText(pinResult.error, "读取 PIN 状态失败。"));
      }
    } catch (error) {
      setErrorText(toErrorText(error, "读取安全设置失败，请稍后重试。"));
    } finally {
      setIsStatusLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!open) {
      setIsPinDialogOpen(false);
      setPinInput("");
      setPinDialogError(null);
      return;
    }
    setActiveSection("general");
    void loadSecuritySettings();
  }, [loadSecuritySettings, open]);

  const onPinDialogOpenChange = useCallback(
    (nextOpen: boolean) => {
      if (!nextOpen && isPinBusy) {
        return;
      }

      setIsPinDialogOpen(nextOpen);
      if (!nextOpen) {
        setPinInput("");
        setPinDialogError(null);
      }
    },
    [isPinBusy],
  );

  const onBiometricCheckedChange = useCallback(
    async (checked: boolean) => {
      if (!isBiometricSupported || checked === isBiometricEnabled) {
        return;
      }

      setErrorText(null);
      setIsBiometricBusy(true);
      try {
        const result = checked
          ? await commands.vaultEnableBiometricUnlock({})
          : await commands.vaultDisableBiometricUnlock({});
        if (result.status === "error") {
          setErrorText(
            toErrorText(
              result.error,
              checked ? "启用生物识别失败。" : "禁用生物识别失败。",
            ),
          );
          return;
        }
        setIsBiometricEnabled(checked);
      } catch (error) {
        setErrorText(
          toErrorText(
            error,
            checked ? "启用生物识别失败。" : "禁用生物识别失败。",
          ),
        );
      } finally {
        setIsBiometricBusy(false);
      }
    },
    [isBiometricEnabled, isBiometricSupported],
  );

  const onPinCheckedChange = useCallback(
    async (checked: boolean) => {
      if (!isPinSupported || checked === isPinEnabled) {
        return;
      }

      setErrorText(null);
      if (checked) {
        setPinDialogError(null);
        setPinInput("");
        setIsPinDialogOpen(true);
        return;
      }

      setIsPinBusy(true);
      try {
        const disableResult = await commands.vaultDisablePinUnlock({});
        if (disableResult.status === "error") {
          setErrorText(toErrorText(disableResult.error, "禁用 PIN 失败。"));
          return;
        }
        setIsPinEnabled(false);
      } catch (error) {
        setErrorText(
          toErrorText(error, checked ? "启用 PIN 失败。" : "禁用 PIN 失败。"),
        );
      } finally {
        setIsPinBusy(false);
      }
    },
    [isPinEnabled, isPinSupported],
  );

  const onPinEnableSubmit = useCallback(async () => {
    const normalizedPin = pinInput.trim();
    if (!normalizedPin) {
      setPinDialogError("PIN 不能为空。");
      return;
    }

    setErrorText(null);
    setPinDialogError(null);
    setIsPinBusy(true);
    try {
      const result = await commands.vaultEnablePinUnlock({
        pin: normalizedPin,
        lockType: "persistent",
      });
      if (result.status === "error") {
        setPinDialogError(toErrorText(result.error, "启用 PIN 失败。"));
        return;
      }

      setIsPinEnabled(true);
      setIsPinDialogOpen(false);
      setPinInput("");
    } catch (error) {
      setPinDialogError(toErrorText(error, "启用 PIN 失败。"));
    } finally {
      setIsPinBusy(false);
    }
  }, [pinInput]);

  const onQuickAccessFocusCapture = useCallback(() => {
    quickAccessOriginalShortcutRef.current = quickAccessShortcut;
    quickAccessCapturedRef.current = false;
    setIsQuickAccessCapturing(true);
    setQuickAccessShortcut("");
  }, [quickAccessShortcut]);

  const onQuickAccessBlurCapture = useCallback(() => {
    setIsQuickAccessCapturing(false);
    if (!quickAccessCapturedRef.current) {
      setQuickAccessShortcut(quickAccessOriginalShortcutRef.current);
    }
  }, []);

  const onQuickAccessKeyDownCapture = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === "Tab") {
        return;
      }
      if (event.key === "Escape") {
        event.preventDefault();
        setQuickAccessShortcut(quickAccessOriginalShortcutRef.current);
        quickAccessCapturedRef.current = false;
        event.currentTarget.blur();
        return;
      }

      const formatted = formatShortcutFromKeyboardEvent(event);
      if (!formatted) {
        return;
      }

      event.preventDefault();
      quickAccessCapturedRef.current = true;
      setQuickAccessShortcut(formatted);
    },
    [],
  );

  const onLockFocusCapture = useCallback(() => {
    lockOriginalShortcutRef.current = lockShortcut;
    lockCapturedRef.current = false;
    setIsLockCapturing(true);
    setLockShortcut("");
  }, [lockShortcut]);

  const onLockBlurCapture = useCallback(() => {
    setIsLockCapturing(false);
    if (!lockCapturedRef.current) {
      setLockShortcut(lockOriginalShortcutRef.current);
    }
  }, []);

  const onLockKeyDownCapture = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === "Tab") {
        return;
      }
      if (event.key === "Escape") {
        event.preventDefault();
        setLockShortcut(lockOriginalShortcutRef.current);
        lockCapturedRef.current = false;
        event.currentTarget.blur();
        return;
      }

      const formatted = formatShortcutFromKeyboardEvent(event);
      if (!formatted) {
        return;
      }

      event.preventDefault();
      lockCapturedRef.current = true;
      setLockShortcut(formatted);
    },
    [],
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="grid h-170 grid-rows-[auto_minmax(0,1fr)_auto] overflow-hidden sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>设置</DialogTitle>
          <DialogDescription>
            在这里管理 Vault 的相关偏好设置。更多设置项将逐步开放。
          </DialogDescription>
        </DialogHeader>

        <div className="grid h-full min-h-0 gap-4 sm:grid-cols-[170px_minmax(0,1fr)]">
          <aside className="rounded-lg border border-slate-200 bg-slate-50/80 p-2">
            <div className="space-y-1">
              <button
                type="button"
                className={[
                  "w-full rounded-md px-3 py-2 text-left text-sm transition-colors",
                  activeSection === "general"
                    ? "bg-sky-100 font-medium text-sky-800"
                    : "text-slate-700 hover:bg-slate-100",
                ].join(" ")}
                onClick={() => setActiveSection("general")}
              >
                <span className="inline-flex items-center gap-2">
                  <SlidersHorizontal className="size-4" />
                  通用
                </span>
              </button>
              <button
                type="button"
                className={[
                  "w-full rounded-md px-3 py-2 text-left text-sm transition-colors",
                  activeSection === "security"
                    ? "bg-sky-100 font-medium text-sky-800"
                    : "text-slate-700 hover:bg-slate-100",
                ].join(" ")}
                onClick={() => setActiveSection("security")}
              >
                <span className="inline-flex items-center gap-2">
                  <Shield className="size-4" />
                  安全
                </span>
              </button>
            </div>
          </aside>

          <section className="h-full overflow-y-auto rounded-lg border border-slate-200 bg-slate-50/70 p-3">
            {activeSection === "security" && (
              <>
                <h3 className="text-sm font-medium text-slate-900">解锁</h3>
                <p className="mt-1 text-xs text-slate-600">
                  配置快速解锁方式。
                </p>

                <div className="mt-3 space-y-2">
                  {isBiometricSupported !== false && (
                    <label
                      htmlFor="vault-setting-biometric"
                      className="flex items-start justify-between gap-3 rounded-md border border-slate-200 bg-white px-3 py-2"
                    >
                      <div className="space-y-0.5">
                        <div className="text-sm text-slate-900">
                          通过生物识别解锁
                        </div>
                        <div className="text-xs text-slate-600">
                          {isBiometricSupported
                            ? "使用系统生物识别快速解锁。"
                            : "正在检测生物识别可用性。"}
                        </div>
                      </div>
                      <input
                        id="vault-setting-biometric"
                        type="checkbox"
                        className="mt-0.5 size-4 accent-sky-600"
                        checked={isBiometricEnabled}
                        disabled={
                          isStatusLoading ||
                          isBiometricBusy ||
                          !isBiometricSupported
                        }
                        onChange={(event) => {
                          void onBiometricCheckedChange(event.target.checked);
                        }}
                      />
                    </label>
                  )}

                  <label
                    htmlFor="vault-setting-pin"
                    className="flex items-start justify-between gap-3 rounded-md border border-slate-200 bg-white px-3 py-2"
                  >
                    <div className="space-y-0.5">
                      <div className="text-sm text-slate-900">
                        通过 PIN 解锁
                      </div>
                      <div className="text-xs text-slate-600">
                        {isPinSupported
                          ? "启用后可通过 PIN 快速解锁。"
                          : "当前设备暂不支持 PIN 解锁。"}
                      </div>
                    </div>
                    <input
                      id="vault-setting-pin"
                      type="checkbox"
                      className="mt-0.5 size-4 accent-sky-600"
                      checked={isPinEnabled}
                      disabled={isStatusLoading || isPinBusy || !isPinSupported}
                      onChange={(event) => {
                        void onPinCheckedChange(event.target.checked);
                      }}
                    />
                  </label>

                  <div className="rounded-md border border-slate-200 bg-white px-3 py-2">
                    <div className="mb-2 text-sm text-slate-900">
                      需要主密码
                    </div>
                    <Select
                      value={requireMasterPasswordAfter}
                      onValueChange={(value) =>
                        setRequireMasterPasswordAfter(
                          value as RequireMasterPasswordOption,
                        )
                      }
                    >
                      <SelectTrigger
                        id="vault-setting-require-master-password"
                        className="w-full bg-white"
                      >
                        <SelectValue placeholder="选择需要主密码的时间" />
                      </SelectTrigger>
                      <SelectContent>
                        {REQUIRE_MASTER_PASSWORD_OPTIONS.map((option) => (
                          <SelectItem key={option.value} value={option.value}>
                            {option.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <h3 className="mt-4 text-sm font-medium text-slate-900">
                  自动锁定
                </h3>
                <p className="mt-1 text-xs text-slate-600">
                  配置设备状态或闲置时的自动锁定行为。
                </p>

                <div className="mt-3 space-y-2">
                  <label
                    htmlFor="vault-setting-lock-on-sleep"
                    className="flex items-start justify-between gap-3 rounded-md border border-slate-200 bg-white px-3 py-2"
                  >
                    <div className="space-y-0.5">
                      <div className="text-sm text-slate-900">
                        设备锁定或休眠时锁定
                      </div>
                    </div>
                    <input
                      id="vault-setting-lock-on-sleep"
                      type="checkbox"
                      className="mt-0.5 size-4 accent-sky-600"
                      checked={lockWhenDeviceSleep}
                      onChange={(event) => {
                        setLockWhenDeviceSleep(event.target.checked);
                      }}
                    />
                  </label>

                  <div className="rounded-md border border-slate-200 bg-white px-3 py-2">
                    <div className="mb-2 text-sm text-slate-900">
                      自动锁定需等待设备闲置
                    </div>
                    <Select
                      value={autoLockIdleDelay}
                      onValueChange={(value) =>
                        setAutoLockIdleDelay(value as AutoLockIdleOption)
                      }
                    >
                      <SelectTrigger
                        id="vault-setting-auto-lock-idle-delay"
                        className="w-full bg-white"
                      >
                        <SelectValue placeholder="选择闲置时长" />
                      </SelectTrigger>
                      <SelectContent>
                        {AUTO_LOCK_IDLE_OPTIONS.map((option) => (
                          <SelectItem key={option.value} value={option.value}>
                            {option.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <h3 className="mt-4 text-sm font-medium text-slate-900">
                  剪贴板
                </h3>
                <p className="mt-1 text-xs text-slate-600">
                  配置复制信息后的自动清理行为。
                </p>

                <div className="mt-3 space-y-2">
                  <div className="rounded-md border border-slate-200 bg-white px-3 py-2">
                    <div className="mb-2 text-sm text-slate-900">
                      移除复制的信息
                    </div>
                    <Select
                      value={clipboardClearAfter}
                      onValueChange={(value) =>
                        setClipboardClearAfter(value as ClipboardClearOption)
                      }
                    >
                      <SelectTrigger
                        id="vault-setting-clipboard-clear-after"
                        className="w-full bg-white"
                      >
                        <SelectValue placeholder="选择清理时间" />
                      </SelectTrigger>
                      <SelectContent>
                        {CLIPBOARD_CLEAR_OPTIONS.map((option) => (
                          <SelectItem key={option.value} value={option.value}>
                            {option.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                {errorText && (
                  <div className="mt-3 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700">
                    {errorText}
                  </div>
                )}
              </>
            )}

            {activeSection === "general" && (
              <div className="flex h-full flex-col">
                <h3 className="text-sm font-medium text-slate-900">通用</h3>
                <p className="mt-1 text-xs text-slate-600">
                  管理应用的通用偏好设置。
                </p>
                <div className="mt-3 space-y-2">
                  <label
                    htmlFor="vault-setting-launch-on-login"
                    className="flex items-start justify-between gap-3 rounded-md border border-slate-200 bg-white px-3 py-2"
                  >
                    <div className="space-y-0.5">
                      <div className="text-sm text-slate-900">登录时启动</div>
                    </div>
                    <input
                      id="vault-setting-launch-on-login"
                      type="checkbox"
                      className="mt-0.5 size-4 accent-sky-600"
                      checked={launchOnLogin}
                      onChange={(event) => {
                        setLaunchOnLogin(event.target.checked);
                      }}
                    />
                  </label>

                  <label
                    htmlFor="vault-setting-show-website-icon"
                    className="flex items-start justify-between gap-3 rounded-md border border-slate-200 bg-white px-3 py-2"
                  >
                    <div className="space-y-0.5">
                      <div className="text-sm text-slate-900">显示网站图标</div>
                    </div>
                    <input
                      id="vault-setting-show-website-icon"
                      type="checkbox"
                      className="mt-0.5 size-4 accent-sky-600"
                      checked={showWebsiteIcon}
                      onChange={(event) => {
                        setShowWebsiteIcon(event.target.checked);
                      }}
                    />
                  </label>

                  <div className="rounded-md border border-slate-200 bg-white px-3 py-2">
                    <div className="mb-2 text-sm text-slate-900">语言</div>
                    <Select
                      value={language}
                      onValueChange={(value) =>
                        setLanguage(value as LanguageOption)
                      }
                    >
                      <SelectTrigger
                        id="vault-setting-language"
                        className="w-full bg-white"
                      >
                        <SelectValue placeholder="选择语言" />
                      </SelectTrigger>
                      <SelectContent>
                        {LANGUAGE_OPTIONS.map((option) => (
                          <SelectItem key={option.value} value={option.value}>
                            {option.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <h3 className="mt-4 text-sm font-medium text-slate-900">
                  键盘快捷键
                </h3>
                <div className="mt-3 space-y-2">
                  <div className="rounded-md border border-slate-200 bg-white px-3 py-2">
                    <div className="mb-2 text-sm text-slate-900">快速访问</div>
                    <div className="relative">
                      <Input
                        value={quickAccessShortcut}
                        onChange={(event) =>
                          setQuickAccessShortcut(event.target.value)
                        }
                        onFocus={onQuickAccessFocusCapture}
                        onBlur={onQuickAccessBlurCapture}
                        onKeyDown={onQuickAccessKeyDownCapture}
                        placeholder={
                          isQuickAccessCapturing ? "输入键盘快捷键" : "未配置"
                        }
                        className={quickAccessShortcut ? "pr-8" : undefined}
                      />
                      {quickAccessShortcut && (
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon-xs"
                          className="absolute top-1/2 right-1 -translate-y-1/2"
                          aria-label="清空快速访问快捷键"
                          title="清空"
                          onClick={() => {
                            setQuickAccessShortcut("");
                          }}
                        >
                          <X className="size-3.5" />
                        </Button>
                      )}
                    </div>
                  </div>
                  <div className="rounded-md border border-slate-200 bg-white px-3 py-2">
                    <div className="mb-2 text-sm text-slate-900">锁定</div>
                    <div className="relative">
                      <Input
                        value={lockShortcut}
                        onChange={(event) =>
                          setLockShortcut(event.target.value)
                        }
                        onFocus={onLockFocusCapture}
                        onBlur={onLockBlurCapture}
                        onKeyDown={onLockKeyDownCapture}
                        placeholder={
                          isLockCapturing ? "输入键盘快捷键" : "未配置"
                        }
                        className={lockShortcut ? "pr-8" : undefined}
                      />
                      {lockShortcut && (
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon-xs"
                          className="absolute top-1/2 right-1 -translate-y-1/2"
                          aria-label="清空锁定快捷键"
                          title="清空"
                          onClick={() => {
                            setLockShortcut("");
                          }}
                        >
                          <X className="size-3.5" />
                        </Button>
                      )}
                    </div>
                  </div>
                </div>
                <div className="min-h-0 flex-1" />
              </div>
            )}
          </section>
        </div>

        <DialogFooter showCloseButton />
      </DialogContent>

      <Dialog open={isPinDialogOpen} onOpenChange={onPinDialogOpenChange}>
        <DialogContent className="sm:max-w-sm" showCloseButton={!isPinBusy}>
          <DialogHeader>
            <DialogTitle>启用 PIN 解锁</DialogTitle>
            <DialogDescription>
              输入用于解锁 Vault 的 PIN，确认后将立即启用。
            </DialogDescription>
          </DialogHeader>

          <form
            className="space-y-4"
            onSubmit={(event) => {
              event.preventDefault();
              void onPinEnableSubmit();
            }}
          >
            <div className="space-y-2">
              <Input
                autoFocus
                type="password"
                inputMode="numeric"
                autoComplete="off"
                placeholder="请输入 PIN"
                value={pinInput}
                disabled={isPinBusy}
                onChange={(event) => {
                  setPinInput(event.target.value);
                  if (pinDialogError) {
                    setPinDialogError(null);
                  }
                }}
              />

              {pinDialogError && (
                <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700">
                  {pinDialogError}
                </div>
              )}
            </div>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                disabled={isPinBusy}
                onClick={() => onPinDialogOpenChange(false)}
              >
                取消
              </Button>
              <Button type="submit" disabled={isPinBusy}>
                {isPinBusy ? "启用中..." : "确定"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </Dialog>
  );
}
