import type { FormEvent } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { commands, type RestoreAuthStateResponseDto } from "@/bindings";
import type { UnlockFeedback } from "@/features/auth/unlock/types";
import { errorHandler } from "@/lib/error-handler";
import { loadUnlockCapabilities } from "./load-unlock-capabilities";
import {
  unlockWithBiometric,
  unlockWithMasterPassword,
  unlockWithPin,
} from "./unlock-actions";
import {
  createDefaultUnlockCapabilities,
  type UnlockMethod,
} from "./unlock-capabilities";

type UseUnlockFlowParams = {
  navigateToHome: () => Promise<void>;
  navigateToVault: () => Promise<void>;
};

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

export function useUnlockFlow({
  navigateToHome,
  navigateToVault,
}: UseUnlockFlowParams): UseUnlockFlowResult {
  const [capabilities, setCapabilities] = useState(
    createDefaultUnlockCapabilities,
  );
  const [isRestoring, setIsRestoring] = useState(true);
  const [isUnlocking, setIsUnlocking] = useState(false);
  const [isPinUnlocking, setIsPinUnlocking] = useState(false);
  const [isBiometricUnlocking, setIsBiometricUnlocking] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [masterPassword, setMasterPassword] = useState("");
  const [pin, setPin] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [feedback, setFeedback] = useState<UnlockFeedback>({ kind: "idle" });

  const loadRestoreState = useCallback(async () => {
    setIsRestoring(true);
    try {
      const nextCapabilities = await loadUnlockCapabilities();
      setCapabilities(nextCapabilities);
    } catch (error) {
      errorHandler.handle(error);
      setCapabilities(createDefaultUnlockCapabilities());
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
      capabilities.restoreState?.status !== "needsLogin" &&
      !capabilities.isVaultUnlocked &&
      masterPassword.trim().length > 0,
    [
      capabilities.isVaultUnlocked,
      capabilities.restoreState?.status,
      isBiometricUnlocking,
      isLoggingOut,
      isPinUnlocking,
      isRestoring,
      isUnlocking,
      masterPassword,
    ],
  );

  const canPinUnlock = useMemo(
    () =>
      !isRestoring &&
      !isUnlocking &&
      !isPinUnlocking &&
      !isBiometricUnlocking &&
      !isLoggingOut &&
      capabilities.restoreState?.status !== "needsLogin" &&
      !capabilities.isVaultUnlocked &&
      capabilities.pinEnabled &&
      pin.trim().length > 0,
    [
      capabilities.isVaultUnlocked,
      capabilities.pinEnabled,
      capabilities.restoreState?.status,
      isBiometricUnlocking,
      isLoggingOut,
      isPinUnlocking,
      isRestoring,
      isUnlocking,
      pin,
    ],
  );

  const onUnlock = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (
      capabilities.restoreState?.status === "needsLogin" ||
      capabilities.isVaultUnlocked
    ) {
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
      const result = await unlockWithMasterPassword(trimmedPassword);

      if (result.status === "error") {
        errorHandler.handle(result.error);
        return;
      }

      setMasterPassword("");
      setPin("");
      await navigateToVault();
    } catch (error) {
      errorHandler.handle(error);
    } finally {
      setIsUnlocking(false);
    }
  };

  const onPinUnlock = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (
      capabilities.restoreState?.status === "needsLogin" ||
      capabilities.isVaultUnlocked
    ) {
      setFeedback({
        kind: "error",
        text: "当前会话不是锁定状态，无法执行解锁。",
      });
      return;
    }

    if (!capabilities.pinEnabled) {
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
      const result = await unlockWithPin(trimmedPin);

      if (result.status === "error") {
        errorHandler.handle(result.error);
        return;
      }

      setPin("");
      setMasterPassword("");
      await navigateToVault();
    } catch (error) {
      errorHandler.handle(error);
    } finally {
      setIsPinUnlocking(false);
    }
  };

  const onBiometricUnlock = async () => {
    if (
      capabilities.restoreState?.status === "needsLogin" ||
      capabilities.isVaultUnlocked
    ) {
      setFeedback({
        kind: "error",
        text: "当前会话不是锁定状态，无法执行生物识别解锁。",
      });
      return;
    }

    setIsBiometricUnlocking(true);
    setFeedback({ kind: "idle" });

    try {
      const result = await unlockWithBiometric();
      if (result.status === "error") {
        errorHandler.handle(result.error);
        return;
      }
      setPin("");
      setMasterPassword("");
      await navigateToVault();
    } catch (error) {
      errorHandler.handle(error);
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
        errorHandler.handle(result.error);
        return;
      }
      setPin("");
      setMasterPassword("");
      await navigateToHome();
    } catch (error) {
      errorHandler.handle(error);
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
    setCapabilities((previous) => ({
      ...previous,
      unlockMethod: "masterPassword",
    }));
    setFeedback({ kind: "idle" });
  };

  const onShowPinUnlock = () => {
    if (!capabilities.pinSupported) {
      return;
    }
    setCapabilities((previous) => ({
      ...previous,
      unlockMethod: "pin",
    }));
    setFeedback({ kind: "idle" });
  };

  const onToggleShowPassword = () => {
    setShowPassword((previous) => !previous);
  };

  return {
    biometricEnabled: capabilities.biometricEnabled,
    biometricSupported: capabilities.biometricSupported,
    canBiometricUnlock: capabilities.canBiometricUnlock,
    canPinUnlock,
    canUnlock,
    feedback,
    isBiometricUnlocking,
    isLoggingOut,
    isPinUnlocking,
    isRestoring,
    isUnlocking,
    isVaultUnlocked: capabilities.isVaultUnlocked,
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
    pinEnabled: capabilities.pinEnabled,
    pinSupported: capabilities.pinSupported,
    restoreState: capabilities.restoreState,
    showPassword,
    unlockMethod: capabilities.unlockMethod,
  };
}
