import type { FormEvent } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { commands, type RestoreAuthStateResponseDto } from "@/bindings";
import { toErrorText } from "@/features/auth/shared/utils";
import type { UnlockFeedback } from "@/features/auth/unlock/types";

type UseUnlockFlowParams = {
  navigateToHome: () => Promise<void>;
  navigateToVault: () => Promise<void>;
};

type UnlockMethod = "pin" | "masterPassword";

type UseUnlockFlowResult = {
  biometricEnabled: boolean;
  biometricSupported: boolean;
  canBiometricUnlock: boolean;
  canPinUnlock: boolean;
  canUnlock: boolean;
  feedback: UnlockFeedback;
  isBiometricUnlocking: boolean;
  isLoggingOut: boolean;
  isPinUnlocking: boolean;
  isRestoring: boolean;
  isUnlocking: boolean;
  isVaultUnlocked: boolean;
  masterPassword: string;
  onBiometricUnlock: () => Promise<void>;
  onLogout: () => Promise<void>;
  onMasterPasswordChange: (value: string) => void;
  onPinChange: (value: string) => void;
  onPinUnlock: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  onShowMasterPasswordUnlock: () => void;
  onShowPinUnlock: () => void;
  onToggleShowPassword: () => void;
  onUnlock: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  pin: string;
  pinEnabled: boolean;
  pinSupported: boolean;
  restoreState: RestoreAuthStateResponseDto | null;
  showPassword: boolean;
  unlockMethod: UnlockMethod;
};

function toUnlockErrorText(error: unknown): string {
  return toErrorText(error, "解锁失败，请稍后重试。");
}

export function useUnlockFlow({
  navigateToHome,
  navigateToVault,
}: UseUnlockFlowParams): UseUnlockFlowResult {
  const [restoreState, setRestoreState] =
    useState<RestoreAuthStateResponseDto | null>(null);
  const [isRestoring, setIsRestoring] = useState(true);
  const [isUnlocking, setIsUnlocking] = useState(false);
  const [isPinUnlocking, setIsPinUnlocking] = useState(false);
  const [isBiometricUnlocking, setIsBiometricUnlocking] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [biometricSupported, setBiometricSupported] = useState(false);
  const [biometricEnabled, setBiometricEnabled] = useState(false);
  const [pinSupported, setPinSupported] = useState(false);
  const [pinEnabled, setPinEnabled] = useState(false);
  const [canBiometricUnlock, setCanBiometricUnlock] = useState(false);
  const [isVaultUnlocked, setIsVaultUnlocked] = useState(false);
  const [unlockMethod, setUnlockMethod] =
    useState<UnlockMethod>("masterPassword");
  const [masterPassword, setMasterPassword] = useState("");
  const [pin, setPin] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [feedback, setFeedback] = useState<UnlockFeedback>({ kind: "idle" });

  const loadRestoreState = useCallback(async () => {
    setIsRestoring(true);
    try {
      const result = await commands.authRestoreState({});
      if (result.status === "error") {
        setFeedback({ kind: "error", text: toUnlockErrorText(result.error) });
        setBiometricSupported(false);
        setBiometricEnabled(false);
        setPinSupported(false);
        setPinEnabled(false);
        setCanBiometricUnlock(false);
        setIsVaultUnlocked(false);
        setUnlockMethod("masterPassword");
        return;
      }

      setRestoreState(result.data);

      let vaultUnlocked = false;
      if (result.data.status !== "needsLogin") {
        const unlockedResult = await commands.vaultIsUnlocked();
        vaultUnlocked = unlockedResult.status === "ok" && unlockedResult.data;
      }
      setIsVaultUnlocked(vaultUnlocked);

      const [biometricStatus, pinStatus] = await Promise.all([
        commands.vaultGetBiometricStatus(),
        commands.vaultGetPinStatus(),
      ]);

      const nextBiometricSupported =
        biometricStatus.status === "ok" && biometricStatus.data.supported;
      const nextBiometricEnabled =
        biometricStatus.status === "ok" && biometricStatus.data.enabled;
      const nextPinSupported =
        pinStatus.status === "ok" && pinStatus.data.supported;
      const nextPinEnabled =
        pinStatus.status === "ok" && pinStatus.data.enabled;

      setBiometricSupported(nextBiometricSupported);
      setBiometricEnabled(nextBiometricEnabled);
      setPinSupported(nextPinSupported);
      setPinEnabled(nextPinEnabled);
      setUnlockMethod(nextPinEnabled ? "pin" : "masterPassword");

      if (!vaultUnlocked && nextBiometricSupported && nextBiometricEnabled) {
        const canUnlockWithBiometric =
          await commands.vaultCanUnlockWithBiometric();
        setCanBiometricUnlock(
          canUnlockWithBiometric.status === "ok" && canUnlockWithBiometric.data,
        );
      } else {
        setCanBiometricUnlock(false);
      }
    } catch (error) {
      setFeedback({ kind: "error", text: toUnlockErrorText(error) });
      setBiometricSupported(false);
      setBiometricEnabled(false);
      setPinSupported(false);
      setPinEnabled(false);
      setCanBiometricUnlock(false);
      setIsVaultUnlocked(false);
      setUnlockMethod("masterPassword");
    } finally {
      setIsRestoring(false);
    }
  }, []);

  useEffect(() => {
    void loadRestoreState();
  }, [loadRestoreState]);

  const canUnlock = useMemo(
    () =>
      !isRestoring &&
      !isUnlocking &&
      !isPinUnlocking &&
      !isBiometricUnlocking &&
      !isLoggingOut &&
      restoreState?.status !== "needsLogin" &&
      !isVaultUnlocked &&
      masterPassword.trim().length > 0,
    [
      isBiometricUnlocking,
      isLoggingOut,
      isPinUnlocking,
      isRestoring,
      isUnlocking,
      isVaultUnlocked,
      masterPassword,
      restoreState?.status,
    ],
  );

  const canPinUnlock = useMemo(
    () =>
      !isRestoring &&
      !isUnlocking &&
      !isPinUnlocking &&
      !isBiometricUnlocking &&
      !isLoggingOut &&
      restoreState?.status !== "needsLogin" &&
      !isVaultUnlocked &&
      pinEnabled &&
      pin.trim().length > 0,
    [
      isBiometricUnlocking,
      isLoggingOut,
      isPinUnlocking,
      isRestoring,
      isUnlocking,
      isVaultUnlocked,
      pin,
      pinEnabled,
      restoreState?.status,
    ],
  );

  const onUnlock = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (restoreState?.status === "needsLogin" || isVaultUnlocked) {
      setFeedback({
        kind: "error",
        text: "当前会话不是锁定状态，无法执行解锁。",
      });
      return;
    }

    const trimmedPassword = masterPassword.trim();
    if (!trimmedPassword) {
      setFeedback({ kind: "error", text: "请输入 master password。" });
      return;
    }

    setIsUnlocking(true);
    setFeedback({ kind: "idle" });

    try {
      const result = await commands.vaultUnlock({
        method: {
          type: "masterPassword",
          password: trimmedPassword,
        },
      });

      if (result.status === "error") {
        setFeedback({ kind: "error", text: toUnlockErrorText(result.error) });
        return;
      }

      setMasterPassword("");
      setPin("");
      await navigateToVault();
    } catch (error) {
      setFeedback({ kind: "error", text: toUnlockErrorText(error) });
    } finally {
      setIsUnlocking(false);
    }
  };

  const onPinUnlock = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (restoreState?.status === "needsLogin" || isVaultUnlocked) {
      setFeedback({
        kind: "error",
        text: "当前会话不是锁定状态，无法执行解锁。",
      });
      return;
    }

    if (!pinEnabled) {
      setFeedback({
        kind: "error",
        text: "当前账号未启用 PIN 解锁，请使用 master password 解锁。",
      });
      return;
    }

    const trimmedPin = pin.trim();
    if (!trimmedPin) {
      setFeedback({ kind: "error", text: "请输入 PIN。" });
      return;
    }

    setIsPinUnlocking(true);
    setFeedback({ kind: "idle" });

    try {
      const result = await commands.vaultUnlock({
        method: {
          type: "pin",
          pin: trimmedPin,
        },
      });

      if (result.status === "error") {
        setFeedback({ kind: "error", text: toUnlockErrorText(result.error) });
        return;
      }

      setPin("");
      setMasterPassword("");
      await navigateToVault();
    } catch (error) {
      setFeedback({ kind: "error", text: toUnlockErrorText(error) });
    } finally {
      setIsPinUnlocking(false);
    }
  };

  const onBiometricUnlock = async () => {
    if (restoreState?.status === "needsLogin" || isVaultUnlocked) {
      setFeedback({
        kind: "error",
        text: "当前会话不是锁定状态，无法执行生物识别解锁。",
      });
      return;
    }

    setIsBiometricUnlocking(true);
    setFeedback({ kind: "idle" });

    try {
      const result = await commands.vaultUnlock({
        method: {
          type: "biometric",
        },
      });
      if (result.status === "error") {
        setFeedback({ kind: "error", text: toUnlockErrorText(result.error) });
        return;
      }
      setPin("");
      setMasterPassword("");
      await navigateToVault();
    } catch (error) {
      setFeedback({ kind: "error", text: toUnlockErrorText(error) });
    } finally {
      setIsBiometricUnlocking(false);
    }
  };

  const onLogout = async () => {
    setIsLoggingOut(true);
    setFeedback({ kind: "idle" });
    try {
      const result = await commands.authLogout({});
      if (result.status === "error") {
        setFeedback({ kind: "error", text: toUnlockErrorText(result.error) });
        return;
      }
      setPin("");
      setMasterPassword("");
      await navigateToHome();
    } catch (error) {
      setFeedback({ kind: "error", text: toUnlockErrorText(error) });
    } finally {
      setIsLoggingOut(false);
    }
  };

  const onMasterPasswordChange = (value: string) => {
    setMasterPassword(value);
  };

  const onPinChange = (value: string) => {
    setPin(value);
  };

  const onShowMasterPasswordUnlock = () => {
    setUnlockMethod("masterPassword");
    setFeedback({ kind: "idle" });
  };

  const onShowPinUnlock = () => {
    if (!pinSupported) {
      return;
    }
    setUnlockMethod("pin");
    setFeedback({ kind: "idle" });
  };

  const onToggleShowPassword = () => {
    setShowPassword((previous) => !previous);
  };

  return {
    biometricEnabled,
    biometricSupported,
    canBiometricUnlock,
    canPinUnlock,
    canUnlock,
    feedback,
    isBiometricUnlocking,
    isLoggingOut,
    isPinUnlocking,
    isRestoring,
    isUnlocking,
    isVaultUnlocked,
    masterPassword,
    onBiometricUnlock,
    onLogout,
    onMasterPasswordChange,
    onPinChange,
    onPinUnlock,
    onShowMasterPasswordUnlock,
    onShowPinUnlock,
    onToggleShowPassword,
    onUnlock,
    pin,
    pinEnabled,
    pinSupported,
    restoreState,
    showPassword,
    unlockMethod,
  };
}
