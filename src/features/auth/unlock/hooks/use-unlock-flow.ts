import { useForm } from "@tanstack/react-form";
import { useEffect, useState } from "react";
import { commands } from "@/bindings";
import { unlockFormDefaults } from "@/features/auth/unlock/schema";
import type { UnlockFeedback } from "@/features/auth/unlock/types";
import { appI18n } from "@/i18n";
import { errorHandler } from "@/lib/error-handler";
import { useUnifiedUnlock } from "./use-unified-unlock";

type UseUnlockFlowParams = {
  navigateToHome: () => Promise<void>;
  navigateToVault: () => Promise<void>;
};

export type UnlockForm = ReturnType<typeof useUnlockFlow>["form"];

export function useUnlockFlow({
  navigateToHome,
  navigateToVault,
}: UseUnlockFlowParams) {
  const {
    state: unlockState,
    isLoading,
    isVaultUnlocked,
    unlockWithMasterPassword,
    unlockWithPin,
    unlockWithBiometric,
    logout,
    refreshState,
  } = useUnifiedUnlock();

  const [feedback, setFeedback] = useState<UnlockFeedback>({ kind: "idle" });
  const [showPassword, setShowPassword] = useState(false);
  const [unlockMethod, setUnlockMethod] = useState<"pin" | "masterPassword">(
    "masterPassword",
  );
  const [isPinUnlocking, setIsPinUnlocking] = useState(false);
  const [isBiometricUnlocking, setIsBiometricUnlocking] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);

  const form = useForm({
    defaultValues: unlockFormDefaults,
    onSubmit: async ({ value }) => {
      const trimmedPassword = value.masterPassword.trim();
      if (!trimmedPassword) {
        setFeedback({
          kind: "error",
          text: appI18n.t("auth.unlock.validation.missingMasterPassword"),
        });
        return;
      }

      setFeedback({ kind: "idle" });

      const success = await unlockWithMasterPassword(trimmedPassword);
      if (success) {
        form.reset();
        await navigateToVault();
      } else {
        setFeedback({
          kind: "error",
          text: appI18n.t("auth.unlock.messages.unlockFailed"),
        });
      }
    },
  });

  // Auto-redirect to vault if already unlocked
  useEffect(() => {
    if (!isLoading && isVaultUnlocked) {
      void navigateToVault();
    }
  }, [isLoading, isVaultUnlocked, navigateToVault]);

  const isActionBlocked =
    form.state.isSubmitting ||
    isPinUnlocking ||
    isBiometricUnlocking ||
    isLoggingOut;

  const onPinUnlock = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    const trimmedPin = form.getFieldValue("pin").trim();
    if (!trimmedPin) {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.unlock.validation.missingPin"),
      });
      return;
    }

    setIsPinUnlocking(true);
    setFeedback({ kind: "idle" });

    const success = await unlockWithPin(trimmedPin);
    if (success) {
      form.reset();
      await navigateToVault();
    } else {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.unlock.messages.unlockFailed"),
      });
    }
    setIsPinUnlocking(false);
  };

  const onBiometricUnlock = async () => {
    setIsBiometricUnlocking(true);
    setFeedback({ kind: "idle" });

    const success = await unlockWithBiometric();
    if (success) {
      form.reset();
      await navigateToVault();
    } else {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.unlock.messages.biometricFailed"),
      });
    }
    setIsBiometricUnlocking(false);
  };

  const onLogout = async () => {
    setIsLoggingOut(true);
    setFeedback({ kind: "idle" });
    const success = await logout();
    if (success) {
      form.reset();
      await navigateToHome();
    } else {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.unlock.messages.logoutFailed"),
      });
    }
    setIsLoggingOut(false);
  };

  const onShowMasterPasswordUnlock = () => {
    setUnlockMethod("masterPassword");
    setFeedback({ kind: "idle" });
  };

  const onShowPinUnlock = () => {
    setUnlockMethod("pin");
    setFeedback({ kind: "idle" });
  };

  const onToggleShowPassword = () => {
    setShowPassword((prev) => !prev);
  };

  // Load PIN/biometric capabilities from backend
  const [pinEnabled, setPinEnabled] = useState(false);
  const [pinSupported, setPinSupported] = useState(false);
  const [biometricEnabled, setBiometricEnabled] = useState(false);
  const [biometricSupported, setBiometricSupported] = useState(false);
  const [canBiometricUnlock, setCanBiometricUnlock] = useState(false);

  useEffect(() => {
    const loadCapabilities = async () => {
      try {
        const [biometricStatus, pinStatus] = await Promise.all([
          commands.vaultGetBiometricStatus(),
          commands.vaultGetPinStatus(),
        ]);

        if (biometricStatus.status === "ok") {
          setBiometricSupported(biometricStatus.data.supported);
          setBiometricEnabled(biometricStatus.data.enabled);
        }
        if (pinStatus.status === "ok") {
          setPinSupported(pinStatus.data.supported);
          setPinEnabled(pinStatus.data.enabled);
          // If PIN is enabled, default to PIN unlock method
          if (pinStatus.data.enabled) {
            setUnlockMethod("pin");
          }
        }

        // Check if can biometric unlock
        if (
          !isVaultUnlocked &&
          biometricStatus.status === "ok" &&
          biometricStatus.data.supported &&
          biometricStatus.data.enabled
        ) {
          const canUnlock = await commands.vaultCanUnlockWithBiometric();
          if (canUnlock.status === "ok") {
            setCanBiometricUnlock(canUnlock.data);
          }
        }
      } catch (error) {
        errorHandler.handle(error);
      }
    };

    void loadCapabilities();
  }, [isVaultUnlocked]);

  // Determine if we need login (no account context)
  const needsLogin = !isLoading && unlockState?.account === null;

  return {
    form,
    // Capabilities
    biometricEnabled,
    biometricSupported,
    canBiometricUnlock,
    pinEnabled,
    pinSupported,
    // State
    feedback,
    isActionBlocked,
    isBiometricUnlocking,
    isLoggingOut,
    isPinUnlocking,
    isRestoring: isLoading,
    isVaultUnlocked,
    needsLogin,
    showPassword,
    unlockMethod,
    // Account info from unified state
    account: unlockState?.account,
    // Actions
    onBiometricUnlock,
    onLogout,
    onPinUnlock,
    onShowMasterPasswordUnlock,
    onShowPinUnlock,
    onToggleShowPassword,
    refreshState,
  };
}
