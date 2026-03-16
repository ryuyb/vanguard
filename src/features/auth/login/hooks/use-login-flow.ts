import { useForm } from "@tanstack/react-form";
import { useEffect, useState } from "react";
import { CUSTOM_SERVER_URL_OPTION } from "@/features/auth/login/constants";
import {
  canVaultUnlockAfterLogin,
  formatTwoFactorProviders,
  loginWithPassword,
  normalizeBaseUrl,
  restoreLoginHints,
  sendEmailLoginCode,
  syncVaultAfterLogin,
  toProviderId,
  toServerUrlOption,
  unlockVaultAfterLogin,
} from "@/features/auth/login/login-flow-helpers";
import { loginFormDefaults, loginSchema } from "@/features/auth/login/schema";
import type {
  LoginFeedback,
  TwoFactorState,
} from "@/features/auth/login/types";
import { appI18n } from "@/i18n";
import { errorHandler } from "@/lib/error-handler";

type UseLoginFlowParams = {
  navigateToVault: () => Promise<void>;
};

export type LoginForm = ReturnType<typeof useLoginFlow>["form"];

export function useLoginFlow({ navigateToVault }: UseLoginFlowParams) {
  const [showPassword, setShowPassword] = useState(false);
  const [feedback, setFeedback] = useState<LoginFeedback>({ kind: "idle" });
  const [submitProgressText, setSubmitProgressText] = useState("");
  const [isRestoringSession, setIsRestoringSession] = useState(true);
  const [twoFactorState, setTwoFactorState] = useState<TwoFactorState | null>(
    null,
  );

  const form = useForm({
    defaultValues: loginFormDefaults,
    validators: {
      onSubmit: loginSchema,
    },
    onSubmit: async ({ value }) => {
      const effectiveBaseUrl =
        value.serverUrlOption === CUSTOM_SERVER_URL_OPTION
          ? value.customBaseUrl
          : value.serverUrlOption;
      const normalizedBaseUrl = normalizeBaseUrl(effectiveBaseUrl);
      const trimmedEmail = value.email.trim();

      setFeedback({ kind: "idle" });
      setSubmitProgressText(appI18n.t("auth.login.progress.verifyingAccount"));

      try {
        const twoFactorProvider = twoFactorState
          ? toProviderId(twoFactorState.selectedProvider)
          : null;
        const twoFactorToken = twoFactorState?.token.trim() || null;

        if (twoFactorState && (twoFactorProvider === null || !twoFactorToken)) {
          setFeedback({
            kind: "error",
            text: appI18n.t("auth.login.validation.incompleteTwoFactor"),
          });
          return;
        }

        const result = await loginWithPassword({
          baseUrl: normalizedBaseUrl,
          email: trimmedEmail,
          masterPassword: value.masterPassword,
          twoFactorProvider,
          twoFactorToken,
        });

        if (result.status === "error") {
          errorHandler.handle(result.error);
          return;
        }

        if (result.data.status === "authenticated") {
          setSubmitProgressText(
            appI18n.t("auth.login.progress.preparingVault"),
          );
          setTwoFactorState(null);

          const canUnlockResult = await canVaultUnlockAfterLogin();
          if (canUnlockResult.status === "error") {
            errorHandler.handle(canUnlockResult.error);
            return;
          }

          if (canUnlockResult.data) {
            setSubmitProgressText(
              appI18n.t("auth.login.progress.unlockingLocalVault"),
            );
            const unlockResult = await unlockVaultAfterLogin(
              value.masterPassword,
            );
            if (unlockResult.status === "error") {
              errorHandler.handle(unlockResult.error);
              return;
            }

            setSubmitProgressText(
              appI18n.t("auth.login.progress.syncingLatestData"),
            );
            const syncResult = await syncVaultAfterLogin();
            if (syncResult.status === "error") {
              form.setFieldValue("masterPassword", "");
              await navigateToVault();
              return;
            }

            form.setFieldValue("masterPassword", "");
            await navigateToVault();
            return;
          }

          setSubmitProgressText(appI18n.t("auth.login.progress.firstSync"));
          const syncResult = await syncVaultAfterLogin();
          if (syncResult.status === "error") {
            errorHandler.handle(syncResult.error);
            return;
          }

          setSubmitProgressText(
            appI18n.t("auth.login.progress.finishingUnlock"),
          );
          const unlockResult = await unlockVaultAfterLogin(
            value.masterPassword,
          );
          if (unlockResult.status === "error") {
            errorHandler.handle(unlockResult.error);
            return;
          }

          form.setFieldValue("masterPassword", "");
          await navigateToVault();
          return;
        }

        // Two-factor challenge
        const providers = result.data.providers.length
          ? result.data.providers
          : [];
        const selectedProvider =
          twoFactorState && providers.includes(twoFactorState.selectedProvider)
            ? twoFactorState.selectedProvider
            : (providers[0] ?? "0");

        setTwoFactorState({
          providers,
          selectedProvider,
          token: "",
          isSendingEmailCode: false,
        });

        setFeedback({
          kind: "twoFactor",
          text: appI18n.t("auth.login.messages.twoFactorPrompt", {
            providers: formatTwoFactorProviders(providers),
          }),
        });
      } catch (error) {
        errorHandler.handle(error);
      } finally {
        setSubmitProgressText("");
      }
    },
  });

  // Restore session hints on mount
  useEffect(() => {
    let cancelled = false;

    const restoreSession = async () => {
      setIsRestoringSession(true);
      try {
        const result = await restoreLoginHints();
        if (cancelled || result.status === "error") return;

        const restored = result.data;

        if (restored.baseUrl) {
          const normalizedRestoredBaseUrl = normalizeBaseUrl(restored.baseUrl);
          const restoredOption = toServerUrlOption(normalizedRestoredBaseUrl);
          if (restoredOption === CUSTOM_SERVER_URL_OPTION) {
            const current = form.getFieldValue("customBaseUrl");
            if (!current.trim()) {
              form.setFieldValue("customBaseUrl", normalizedRestoredBaseUrl);
            }
          }
          const currentOption = form.getFieldValue("serverUrlOption");
          if (currentOption === CUSTOM_SERVER_URL_OPTION) {
            form.setFieldValue("serverUrlOption", restoredOption);
          }
        }
        if (restored.email) {
          const current = form.getFieldValue("email");
          if (!current.trim()) {
            form.setFieldValue("email", restored.email);
          }
        }
      } catch {
        // ignored: first-screen hint should not block manual login
      } finally {
        if (!cancelled) setIsRestoringSession(false);
      }
    };

    void restoreSession();
    return () => {
      cancelled = true;
    };
  }, [form]);

  const clearTwoFactorChallenge = () => {
    if (!twoFactorState) return;
    setTwoFactorState(null);
    if (feedback.kind === "twoFactor") {
      setFeedback({ kind: "idle" });
    }
  };

  const onSendEmailCode = async () => {
    if (!twoFactorState) return;
    if (twoFactorState.selectedProvider !== "1") {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.login.validation.nonEmailProvider"),
      });
      return;
    }

    const serverUrlOption = form.getFieldValue("serverUrlOption");
    const customBaseUrl = form.getFieldValue("customBaseUrl");
    const effectiveBaseUrl =
      serverUrlOption === CUSTOM_SERVER_URL_OPTION
        ? customBaseUrl
        : serverUrlOption;
    const normalizedBaseUrl = normalizeBaseUrl(effectiveBaseUrl);
    const trimmedEmail = form.getFieldValue("email").trim();
    const masterPassword = form.getFieldValue("masterPassword");

    if (!normalizedBaseUrl || !trimmedEmail || !masterPassword) {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.login.validation.missingEmailCodeRequirements"),
      });
      return;
    }

    setTwoFactorState((prev) =>
      prev ? { ...prev, isSendingEmailCode: true } : prev,
    );
    setFeedback({ kind: "idle" });

    try {
      const result = await sendEmailLoginCode({
        baseUrl: normalizedBaseUrl,
        email: trimmedEmail,
        masterPassword,
      });

      if (result.status === "error") {
        errorHandler.handle(result.error);
        return;
      }

      setFeedback({
        kind: "success",
        text: appI18n.t("auth.login.messages.emailCodeSent"),
      });
    } catch (error) {
      errorHandler.handle(error);
    } finally {
      setTwoFactorState((prev) =>
        prev ? { ...prev, isSendingEmailCode: false } : prev,
      );
    }
  };

  const onTwoFactorProviderChange = (value: string) => {
    setTwoFactorState((prev) =>
      prev ? { ...prev, selectedProvider: value, token: "" } : prev,
    );
  };

  const onTwoFactorTokenChange = (value: string) => {
    setTwoFactorState((prev) => (prev ? { ...prev, token: value } : prev));
  };

  return {
    form,
    feedback,
    isRestoringSession,
    showPassword,
    submitProgressText,
    twoFactorState,
    clearTwoFactorChallenge,
    onSendEmailCode,
    onToggleShowPassword: () => setShowPassword((prev) => !prev),
    onTwoFactorProviderChange,
    onTwoFactorTokenChange,
  };
}
