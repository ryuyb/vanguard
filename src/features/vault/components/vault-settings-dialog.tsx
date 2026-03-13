import { Shield, SlidersHorizontal, X } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
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
import {
  APP_LOCALE_OPTIONS,
  type AppLocale,
  appI18n,
  changeAppLocale,
} from "@/i18n";

type VaultSettingsDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

type SettingsSection = "security" | "general";
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

const AUTO_LOCK_IDLE_OPTIONS: AutoLockIdleOption[] = [
  "1m",
  "2m",
  "5m",
  "10m",
  "15m",
  "30m",
  "1h",
  "4h",
  "8h",
  "never",
];

const REQUIRE_MASTER_PASSWORD_OPTIONS: RequireMasterPasswordOption[] = [
  "1d",
  "7d",
  "14d",
  "30d",
  "never",
];

const CLIPBOARD_CLEAR_OPTIONS: ClipboardClearOption[] = [
  "10s",
  "20s",
  "30s",
  "1m",
  "2m",
  "5m",
  "never",
];

const QUICK_ACCESS_SHORTCUT = "⌃⇧␣";
const LOCK_SHORTCUT = "⇧⌘L";

function formatShortcutFromKeyboardEvent(
  event: {
    key: string;
    ctrlKey: boolean;
    shiftKey: boolean;
    altKey: boolean;
    metaKey: boolean;
  },
  labels: {
    space: string;
    esc: string;
    up: string;
    down: string;
    left: string;
    right: string;
  },
): string | null {
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
    " ": labels.space,
    Spacebar: labels.space,
    Escape: labels.esc,
    ArrowUp: labels.up,
    ArrowDown: labels.down,
    ArrowLeft: labels.left,
    ArrowRight: labels.right,
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
  const { t } = useTranslation();
  const [activeSection, setActiveSection] =
    useState<SettingsSection>("general");
  const [language, setLanguage] = useState<AppLocale>(
    appI18n.language as AppLocale,
  );
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

  const loadAppConfig = useCallback(async () => {
    try {
      const result = await commands.configGetAppConfig();
      if (result.status === "ok") {
        const config = result.data;
        setLanguage(config.locale as AppLocale);
        setLaunchOnLogin(config.launchOnLogin);
        setShowWebsiteIcon(config.showWebsiteIcon);
        setQuickAccessShortcut(config.quickAccessShortcut);
        setLockShortcut(config.lockShortcut);
        setRequireMasterPasswordAfter(
          config.requireMasterPasswordInterval as RequireMasterPasswordOption,
        );
        setLockWhenDeviceSleep(config.lockOnSleep);
        setAutoLockIdleDelay(config.idleAutoLockDelay as AutoLockIdleOption);
        setClipboardClearAfter(
          config.clipboardClearDelay as ClipboardClearOption,
        );
      }
    } catch {
      // Ignore config load errors, use default values
    }
  }, []);

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
          toErrorText(
            biometricResult.error,
            t("vault.dialogs.settings.errors.loadBiometricStatus"),
          ),
        );
      }

      if (pinResult.status === "ok") {
        setIsPinSupported(pinResult.data.supported);
        setIsPinEnabled(pinResult.data.enabled);
      } else {
        setErrorText(
          toErrorText(
            pinResult.error,
            t("vault.dialogs.settings.errors.loadPinStatus"),
          ),
        );
      }
    } catch (error) {
      setErrorText(
        toErrorText(
          error,
          t("vault.dialogs.settings.errors.loadSecuritySettings"),
        ),
      );
    } finally {
      setIsStatusLoading(false);
    }
  }, [t]);

  useEffect(() => {
    if (!open) {
      setIsPinDialogOpen(false);
      setPinInput("");
      setPinDialogError(null);
      return;
    }
    setActiveSection("general");
    void loadAppConfig();
    void loadSecuritySettings();
  }, [loadAppConfig, loadSecuritySettings, open]);

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

  const onLanguageChange = useCallback(
    async (value: string) => {
      const newLocale = value as AppLocale;
      try {
        await changeAppLocale(newLocale);
        setLanguage(newLocale);
      } catch (error) {
        toast.error(
          toErrorText(error, t("vault.dialogs.settings.errors.saveFailed")),
        );
      }
    },
    [t],
  );

  const onLaunchOnLoginChange = useCallback(
    async (checked: boolean) => {
      setLaunchOnLogin(checked);
      try {
        await commands.configUpdateAppConfig({ launchOnLogin: checked });
      } catch (error) {
        toast.error(
          toErrorText(error, t("vault.dialogs.settings.errors.saveFailed")),
        );
      }
    },
    [t],
  );

  const onShowWebsiteIconChange = useCallback(
    async (checked: boolean) => {
      setShowWebsiteIcon(checked);
      try {
        await commands.configUpdateAppConfig({ showWebsiteIcon: checked });
      } catch (error) {
        toast.error(
          toErrorText(error, t("vault.dialogs.settings.errors.saveFailed")),
        );
      }
    },
    [t],
  );

  const onQuickAccessShortcutSave = useCallback(
    async (shortcut: string) => {
      try {
        await commands.configUpdateAppConfig({ quickAccessShortcut: shortcut });
      } catch (error) {
        toast.error(
          toErrorText(error, t("vault.dialogs.settings.errors.saveFailed")),
        );
      }
    },
    [t],
  );

  const onLockShortcutSave = useCallback(
    async (shortcut: string) => {
      try {
        await commands.configUpdateAppConfig({ lockShortcut: shortcut });
      } catch (error) {
        toast.error(
          toErrorText(error, t("vault.dialogs.settings.errors.saveFailed")),
        );
      }
    },
    [t],
  );

  const onRequireMasterPasswordChange = useCallback(
    async (value: RequireMasterPasswordOption) => {
      setRequireMasterPasswordAfter(value);
      try {
        await commands.configUpdateAppConfig({
          requireMasterPasswordInterval: value,
        });
      } catch (error) {
        toast.error(
          toErrorText(error, t("vault.dialogs.settings.errors.saveFailed")),
        );
      }
    },
    [t],
  );

  const onLockOnSleepChange = useCallback(
    async (checked: boolean) => {
      setLockWhenDeviceSleep(checked);
      try {
        await commands.configUpdateAppConfig({ lockOnSleep: checked });
      } catch (error) {
        toast.error(
          toErrorText(error, t("vault.dialogs.settings.errors.saveFailed")),
        );
      }
    },
    [t],
  );

  const onAutoLockIdleDelayChange = useCallback(
    async (value: AutoLockIdleOption) => {
      setAutoLockIdleDelay(value);
      try {
        await commands.configUpdateAppConfig({ idleAutoLockDelay: value });
      } catch (error) {
        toast.error(
          toErrorText(error, t("vault.dialogs.settings.errors.saveFailed")),
        );
      }
    },
    [t],
  );

  const onClipboardClearAfterChange = useCallback(
    async (value: ClipboardClearOption) => {
      setClipboardClearAfter(value);
      try {
        await commands.configUpdateAppConfig({ clipboardClearDelay: value });
      } catch (error) {
        toast.error(
          toErrorText(error, t("vault.dialogs.settings.errors.saveFailed")),
        );
      }
    },
    [t],
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
              checked
                ? t("vault.dialogs.settings.errors.enableBiometric")
                : t("vault.dialogs.settings.errors.disableBiometric"),
            ),
          );
          return;
        }
        setIsBiometricEnabled(checked);
      } catch (error) {
        setErrorText(
          toErrorText(
            error,
            checked
              ? t("vault.dialogs.settings.errors.enableBiometric")
              : t("vault.dialogs.settings.errors.disableBiometric"),
          ),
        );
      } finally {
        setIsBiometricBusy(false);
      }
    },
    [isBiometricEnabled, isBiometricSupported, t],
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
          setErrorText(
            toErrorText(
              disableResult.error,
              t("vault.dialogs.settings.errors.disablePin"),
            ),
          );
          return;
        }
        setIsPinEnabled(false);
      } catch (error) {
        setErrorText(
          toErrorText(
            error,
            checked
              ? t("vault.dialogs.settings.errors.enablePin")
              : t("vault.dialogs.settings.errors.disablePin"),
          ),
        );
      } finally {
        setIsPinBusy(false);
      }
    },
    [isPinEnabled, isPinSupported, t],
  );

  const onPinEnableSubmit = useCallback(async () => {
    const normalizedPin = pinInput.trim();
    if (!normalizedPin) {
      setPinDialogError(t("vault.dialogs.settings.errors.pinRequired"));
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
        setPinDialogError(
          toErrorText(
            result.error,
            t("vault.dialogs.settings.errors.enablePin"),
          ),
        );
        return;
      }

      setIsPinEnabled(true);
      setIsPinDialogOpen(false);
      setPinInput("");
    } catch (error) {
      setPinDialogError(
        toErrorText(error, t("vault.dialogs.settings.errors.enablePin")),
      );
    } finally {
      setIsPinBusy(false);
    }
  }, [pinInput, t]);

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
    } else {
      void onQuickAccessShortcutSave(quickAccessShortcut);
    }
  }, [onQuickAccessShortcutSave, quickAccessShortcut]);

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

      const formatted = formatShortcutFromKeyboardEvent(event, {
        space: t("vault.dialogs.settings.general.shortcuts.keys.space"),
        esc: t("vault.dialogs.settings.general.shortcuts.keys.esc"),
        up: t("vault.dialogs.settings.general.shortcuts.keys.up"),
        down: t("vault.dialogs.settings.general.shortcuts.keys.down"),
        left: t("vault.dialogs.settings.general.shortcuts.keys.left"),
        right: t("vault.dialogs.settings.general.shortcuts.keys.right"),
      });
      if (!formatted) {
        return;
      }

      event.preventDefault();
      quickAccessCapturedRef.current = true;
      setQuickAccessShortcut(formatted);
    },
    [t],
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
    } else {
      void onLockShortcutSave(lockShortcut);
    }
  }, [lockShortcut, onLockShortcutSave]);

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

      const formatted = formatShortcutFromKeyboardEvent(event, {
        space: t("vault.dialogs.settings.general.shortcuts.keys.space"),
        esc: t("vault.dialogs.settings.general.shortcuts.keys.esc"),
        up: t("vault.dialogs.settings.general.shortcuts.keys.up"),
        down: t("vault.dialogs.settings.general.shortcuts.keys.down"),
        left: t("vault.dialogs.settings.general.shortcuts.keys.left"),
        right: t("vault.dialogs.settings.general.shortcuts.keys.right"),
      });
      if (!formatted) {
        return;
      }

      event.preventDefault();
      lockCapturedRef.current = true;
      setLockShortcut(formatted);
    },
    [t],
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="grid h-170 grid-rows-[auto_minmax(0,1fr)_auto] overflow-hidden sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t("vault.dialogs.settings.title")}</DialogTitle>
          <DialogDescription>
            {t("vault.dialogs.settings.description")}
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
                  {t("vault.dialogs.settings.sections.general")}
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
                  {t("vault.dialogs.settings.sections.security")}
                </span>
              </button>
            </div>
          </aside>

          <section className="h-full overflow-y-auto rounded-lg border border-slate-200 bg-slate-50/70 p-3">
            {activeSection === "security" && (
              <>
                <h3 className="text-sm font-medium text-slate-900">
                  {t("vault.dialogs.settings.security.unlock.title")}
                </h3>
                <p className="mt-1 text-xs text-slate-600">
                  {t("vault.dialogs.settings.security.unlock.description")}
                </p>

                <div className="mt-3 space-y-2">
                  {isBiometricSupported !== false && (
                    <label
                      htmlFor="vault-setting-biometric"
                      className="flex items-start justify-between gap-3 rounded-md border border-slate-200 bg-white px-3 py-2"
                    >
                      <div className="space-y-0.5">
                        <div className="text-sm text-slate-900">
                          {t("vault.dialogs.settings.security.biometric.label")}
                        </div>
                        <div className="text-xs text-slate-600">
                          {isBiometricSupported
                            ? t(
                                "vault.dialogs.settings.security.biometric.enabledHint",
                              )
                            : t(
                                "vault.dialogs.settings.security.biometric.checkingHint",
                              )}
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
                        {t("vault.dialogs.settings.security.pin.label")}
                      </div>
                      <div className="text-xs text-slate-600">
                        {isPinSupported
                          ? t("vault.dialogs.settings.security.pin.enabledHint")
                          : t(
                              "vault.dialogs.settings.security.pin.unsupportedHint",
                            )}
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
                      {t(
                        "vault.dialogs.settings.security.requireMasterPassword",
                      )}
                    </div>
                    <Select
                      value={requireMasterPasswordAfter}
                      onValueChange={(value) =>
                        void onRequireMasterPasswordChange(
                          value as RequireMasterPasswordOption,
                        )
                      }
                    >
                      <SelectTrigger
                        id="vault-setting-require-master-password"
                        className="w-full bg-white"
                      >
                        <SelectValue
                          placeholder={t(
                            "vault.dialogs.settings.placeholders.requireMasterPassword",
                          )}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        {REQUIRE_MASTER_PASSWORD_OPTIONS.map((option) => (
                          <SelectItem key={option} value={option}>
                            {t(
                              `vault.dialogs.settings.options.requireMasterPassword.${option}`,
                            )}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <h3 className="mt-4 text-sm font-medium text-slate-900">
                  {t("vault.dialogs.settings.security.autoLock.title")}
                </h3>
                <p className="mt-1 text-xs text-slate-600">
                  {t("vault.dialogs.settings.security.autoLock.description")}
                </p>

                <div className="mt-3 space-y-2">
                  <label
                    htmlFor="vault-setting-lock-on-sleep"
                    className="flex items-start justify-between gap-3 rounded-md border border-slate-200 bg-white px-3 py-2"
                  >
                    <div className="space-y-0.5">
                      <div className="text-sm text-slate-900">
                        {t("vault.dialogs.settings.security.lockOnSleep")}
                      </div>
                    </div>
                    <input
                      id="vault-setting-lock-on-sleep"
                      type="checkbox"
                      className="mt-0.5 size-4 accent-sky-600"
                      checked={lockWhenDeviceSleep}
                      onChange={(event) => {
                        void onLockOnSleepChange(event.target.checked);
                      }}
                    />
                  </label>

                  <div className="rounded-md border border-slate-200 bg-white px-3 py-2">
                    <div className="mb-2 text-sm text-slate-900">
                      {t("vault.dialogs.settings.security.idleLockDelay")}
                    </div>
                    <Select
                      value={autoLockIdleDelay}
                      onValueChange={(value) =>
                        void onAutoLockIdleDelayChange(value as AutoLockIdleOption)
                      }
                    >
                      <SelectTrigger
                        id="vault-setting-auto-lock-idle-delay"
                        className="w-full bg-white"
                      >
                        <SelectValue
                          placeholder={t(
                            "vault.dialogs.settings.placeholders.autoLockIdle",
                          )}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        {AUTO_LOCK_IDLE_OPTIONS.map((option) => (
                          <SelectItem key={option} value={option}>
                            {t(
                              `vault.dialogs.settings.options.autoLockIdle.${option}`,
                            )}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <h3 className="mt-4 text-sm font-medium text-slate-900">
                  {t("vault.dialogs.settings.security.clipboard.title")}
                </h3>
                <p className="mt-1 text-xs text-slate-600">
                  {t("vault.dialogs.settings.security.clipboard.description")}
                </p>

                <div className="mt-3 space-y-2">
                  <div className="rounded-md border border-slate-200 bg-white px-3 py-2">
                    <div className="mb-2 text-sm text-slate-900">
                      {t(
                        "vault.dialogs.settings.security.clipboard.clearAfter",
                      )}
                    </div>
                    <Select
                      value={clipboardClearAfter}
                      onValueChange={(value) =>
                        void onClipboardClearAfterChange(value as ClipboardClearOption)
                      }
                    >
                      <SelectTrigger
                        id="vault-setting-clipboard-clear-after"
                        className="w-full bg-white"
                      >
                        <SelectValue
                          placeholder={t(
                            "vault.dialogs.settings.placeholders.clipboardClear",
                          )}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        {CLIPBOARD_CLEAR_OPTIONS.map((option) => (
                          <SelectItem key={option} value={option}>
                            {t(
                              `vault.dialogs.settings.options.clipboardClear.${option}`,
                            )}
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
                <h3 className="text-sm font-medium text-slate-900">
                  {t("vault.dialogs.settings.general.title")}
                </h3>
                <p className="mt-1 text-xs text-slate-600">
                  {t("vault.dialogs.settings.general.description")}
                </p>
                <div className="mt-3 space-y-2">
                  <label
                    htmlFor="vault-setting-launch-on-login"
                    className="flex items-start justify-between gap-3 rounded-md border border-slate-200 bg-white px-3 py-2"
                  >
                    <div className="space-y-0.5">
                      <div className="text-sm text-slate-900">
                        {t("vault.dialogs.settings.general.launchOnLogin")}
                      </div>
                    </div>
                    <input
                      id="vault-setting-launch-on-login"
                      type="checkbox"
                      className="mt-0.5 size-4 accent-sky-600"
                      checked={launchOnLogin}
                      onChange={(event) => {
                        void onLaunchOnLoginChange(event.target.checked);
                      }}
                    />
                  </label>

                  <label
                    htmlFor="vault-setting-show-website-icon"
                    className="flex items-start justify-between gap-3 rounded-md border border-slate-200 bg-white px-3 py-2"
                  >
                    <div className="space-y-0.5">
                      <div className="text-sm text-slate-900">
                        {t("vault.dialogs.settings.general.showWebsiteIcon")}
                      </div>
                    </div>
                    <input
                      id="vault-setting-show-website-icon"
                      type="checkbox"
                      className="mt-0.5 size-4 accent-sky-600"
                      checked={showWebsiteIcon}
                      onChange={(event) => {
                        void onShowWebsiteIconChange(event.target.checked);
                      }}
                    />
                  </label>

                  <div className="rounded-md border border-slate-200 bg-white px-3 py-2">
                    <div className="mb-2 text-sm text-slate-900">
                      {t("common.locale.label")}
                    </div>
                    <Select value={language} onValueChange={onLanguageChange}>
                      <SelectTrigger
                        id="vault-setting-language"
                        className="w-full bg-white"
                      >
                        <SelectValue
                          placeholder={t(
                            "vault.dialogs.settings.placeholders.language",
                          )}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        {APP_LOCALE_OPTIONS.map((option) => (
                          <SelectItem key={option.value} value={option.value}>
                            {option.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <h3 className="mt-4 text-sm font-medium text-slate-900">
                  {t("vault.dialogs.settings.general.shortcuts.title")}
                </h3>
                <div className="mt-3 space-y-2">
                  <div className="rounded-md border border-slate-200 bg-white px-3 py-2">
                    <div className="mb-2 text-sm text-slate-900">
                      {t(
                        "vault.dialogs.settings.general.shortcuts.quickAccess",
                      )}
                    </div>
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
                          isQuickAccessCapturing
                            ? t(
                                "vault.dialogs.settings.general.shortcuts.inputHint",
                              )
                            : t(
                                "vault.dialogs.settings.general.shortcuts.unset",
                              )
                        }
                        className={quickAccessShortcut ? "pr-8" : undefined}
                      />
                      {quickAccessShortcut && (
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon-xs"
                          className="absolute top-1/2 right-1 -translate-y-1/2"
                          aria-label={t(
                            "vault.dialogs.settings.general.shortcuts.clearQuickAccess",
                          )}
                          title={t(
                            "vault.dialogs.settings.general.shortcuts.clear",
                          )}
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
                    <div className="mb-2 text-sm text-slate-900">
                      {t("vault.dialogs.settings.general.shortcuts.lock")}
                    </div>
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
                          isLockCapturing
                            ? t(
                                "vault.dialogs.settings.general.shortcuts.inputHint",
                              )
                            : t(
                                "vault.dialogs.settings.general.shortcuts.unset",
                              )
                        }
                        className={lockShortcut ? "pr-8" : undefined}
                      />
                      {lockShortcut && (
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon-xs"
                          className="absolute top-1/2 right-1 -translate-y-1/2"
                          aria-label={t(
                            "vault.dialogs.settings.general.shortcuts.clearLock",
                          )}
                          title={t(
                            "vault.dialogs.settings.general.shortcuts.clear",
                          )}
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
            <DialogTitle>
              {t("vault.dialogs.settings.pinDialog.title")}
            </DialogTitle>
            <DialogDescription>
              {t("vault.dialogs.settings.pinDialog.description")}
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
                placeholder={t(
                  "vault.dialogs.settings.pinDialog.pinPlaceholder",
                )}
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
                {t("common.actions.cancel")}
              </Button>
              <Button type="submit" disabled={isPinBusy}>
                {isPinBusy
                  ? t("vault.dialogs.settings.pinDialog.enabling")
                  : t("common.actions.confirm")}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </Dialog>
  );
}
