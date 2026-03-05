import type { FormEvent } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { commands, type RestoreAuthStateResponseDto } from "@/bindings";
import { toErrorText } from "@/features/auth/shared/utils";
import type { UnlockFeedback } from "@/features/auth/unlock/types";

type UseUnlockFlowParams = {
  navigateToHome: () => Promise<void>;
  navigateToVault: () => Promise<void>;
};

type UseUnlockFlowResult = {
  biometricEnabled: boolean;
  biometricSupported: boolean;
  canBiometricUnlock: boolean;
  canUnlock: boolean;
  feedback: UnlockFeedback;
  isBiometricUnlocking: boolean;
  isLoggingOut: boolean;
  isRestoring: boolean;
  isUnlocking: boolean;
  isVaultUnlocked: boolean;
  masterPassword: string;
  onBiometricUnlock: () => Promise<void>;
  onLogout: () => Promise<void>;
  onMasterPasswordChange: (value: string) => void;
  onToggleShowPassword: () => void;
  onUnlock: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  restoreState: RestoreAuthStateResponseDto | null;
  showPassword: boolean;
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
  const [isBiometricUnlocking, setIsBiometricUnlocking] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [biometricSupported, setBiometricSupported] = useState(false);
  const [biometricEnabled, setBiometricEnabled] = useState(false);
  const [canBiometricUnlock, setCanBiometricUnlock] = useState(false);
  const [isVaultUnlocked, setIsVaultUnlocked] = useState(false);
  const [masterPassword, setMasterPassword] = useState("");
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
        setCanBiometricUnlock(false);
        setIsVaultUnlocked(false);
        return;
      }
      setRestoreState(result.data);
      let vaultUnlocked = false;
      if (result.data.status !== "needsLogin") {
        const unlockedResult = await commands.vaultIsUnlocked();
        vaultUnlocked = unlockedResult.status === "ok" && unlockedResult.data;
      }
      setIsVaultUnlocked(vaultUnlocked);

      const biometricStatus = await commands.vaultGetBiometricStatus();
      const supported =
        biometricStatus.status === "ok" && biometricStatus.data.supported;
      const enabled =
        biometricStatus.status === "ok" && biometricStatus.data.enabled;
      setBiometricSupported(supported);
      setBiometricEnabled(enabled);

      if (!vaultUnlocked && supported && enabled) {
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
      setCanBiometricUnlock(false);
      setIsVaultUnlocked(false);
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
      !isBiometricUnlocking &&
      !isLoggingOut &&
      restoreState?.status !== "needsLogin" &&
      !isVaultUnlocked &&
      masterPassword.trim().length > 0,
    [
      isBiometricUnlocking,
      isLoggingOut,
      isRestoring,
      isUnlocking,
      isVaultUnlocked,
      masterPassword,
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
      await navigateToVault();
    } catch (error) {
      setFeedback({ kind: "error", text: toUnlockErrorText(error) });
    } finally {
      setIsUnlocking(false);
    }
  };

  const onBiometricUnlock = async () => {
    if (restoreState?.status === "needsLogin" || isVaultUnlocked) {
      setFeedback({
        kind: "error",
        text: "当前会话不是锁定状态，无法执行 Touch ID 解锁。",
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

  const onToggleShowPassword = () => {
    setShowPassword((previous) => !previous);
  };

  return {
    biometricEnabled,
    biometricSupported,
    canBiometricUnlock,
    canUnlock,
    feedback,
    isBiometricUnlocking,
    isLoggingOut,
    isRestoring,
    isUnlocking,
    isVaultUnlocked,
    masterPassword,
    onBiometricUnlock,
    onLogout,
    onMasterPasswordChange,
    onToggleShowPassword,
    onUnlock,
    restoreState,
    showPassword,
  };
}
