import { useForm } from "@tanstack/react-form";
import { useCallback, useEffect, useState } from "react";
import { commands } from "@/bindings";
import { unlockFormDefaults } from "@/features/auth/unlock/schema";
import type { UnlockFeedback } from "@/features/auth/unlock/types";
import { appI18n } from "@/i18n";
import { errorHandler } from "@/lib/error-handler";
import { loadUnlockCapabilities } from "./load-unlock-capabilities";
import {
  unlockWithBiometric,
  unlockWithMasterPassword,
  unlockWithPin,
} from "./unlock-actions";
import { createDefaultUnlockCapabilities } from "./unlock-capabilities";

type UseUnlockFlowParams = {
  navigateToHome: () => Promise<void>;
  navigateToVault: () => Promise<void>;
};

export type UnlockForm = ReturnType<typeof useUnlockFlow>["form"];

export function useUnlockFlow({
  navigateToHome,
  navigateToVault,
}: UseUnlockFlowParams) {
  const [capabilities, setCapabilities] = useState(
    createDefaultUnlockCapabilities,
  );
  const [isRestoring, setIsRestoring] = useState(true);
  const [isPinUnlocking, setIsPinUnlocking] = useState(false);
  const [isBiometricUnlocking, setIsBiometricUnlocking] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [feedback, setFeedback] = useState<UnlockFeedback>({ kind: "idle" });

  const form = useForm({
    defaultValues: unlockFormDefaults,
    onSubmit: async ({ value }) => {
      if (
        capabilities.restoreState?.status === "needsLogin" ||
        capabilities.isVaultUnlocked
      ) {
        setFeedback({
          kind: "error",
          text: appI18n.t("auth.unlock.validation.sessionNotLocked"),
        });
        return;
      }

      const trimmedPassword = value.masterPassword.trim();
      if (!trimmedPassword) {
        setFeedback({
          kind: "error",
          text: appI18n.t("auth.unlock.validation.missingMasterPassword"),
        });
        return;
      }

      setFeedback({ kind: "idle" });

      try {
        const result = await unlockWithMasterPassword(trimmedPassword);
        if (result.status === "error") {
          errorHandler.handle(result.error);
          return;
        }
        form.reset();
        await navigateToVault();
      } catch (error) {
        errorHandler.handle(error);
      }
    },
  });

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

  const isActionBlocked =
    form.state.isSubmitting ||
    isPinUnlocking ||
    isBiometricUnlocking ||
    isLoggingOut;

  const onPinUnlock = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (
      capabilities.restoreState?.status === "needsLogin" ||
      capabilities.isVaultUnlocked
    ) {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.unlock.validation.sessionNotLocked"),
      });
      return;
    }

    if (!capabilities.pinEnabled) {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.unlock.validation.pinNotEnabled"),
      });
      return;
    }

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

    try {
      const result = await unlockWithPin(trimmedPin);
      if (result.status === "error") {
        errorHandler.handle(result.error);
        return;
      }
      form.reset();
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
        text: appI18n.t("auth.unlock.validation.sessionNotLockedBiometric"),
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
      form.reset();
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
      form.reset();
      await navigateToHome();
    } catch (error) {
      errorHandler.handle(error);
    } finally {
      setIsLoggingOut(false);
    }
  };

  const onShowMasterPasswordUnlock = () => {
    setCapabilities((prev) => ({ ...prev, unlockMethod: "masterPassword" }));
    setFeedback({ kind: "idle" });
  };

  const onShowPinUnlock = () => {
    if (!capabilities.pinSupported) return;
    setCapabilities((prev) => ({ ...prev, unlockMethod: "pin" }));
    setFeedback({ kind: "idle" });
  };

  return {
    form,
    biometricEnabled: capabilities.biometricEnabled,
    biometricSupported: capabilities.biometricSupported,
    canBiometricUnlock: capabilities.canBiometricUnlock,
    feedback,
    isActionBlocked,
    isBiometricUnlocking,
    isLoggingOut,
    isPinUnlocking,
    isRestoring,
    isVaultUnlocked: capabilities.isVaultUnlocked,
    onBiometricUnlock,
    onLogout,
    onPinUnlock,
    onShowMasterPasswordUnlock,
    onShowPinUnlock,
    onToggleShowPassword: () => setShowPassword((prev) => !prev),
    pinEnabled: capabilities.pinEnabled,
    restoreState: capabilities.restoreState,
    showPassword,
    unlockMethod: capabilities.unlockMethod,
  };
}
