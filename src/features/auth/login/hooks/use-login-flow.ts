import type { FormEvent } from "react";
import { useEffect, useMemo, useState } from "react";
import { CUSTOM_SERVER_URL_OPTION } from "@/features/auth/login/constants";
import {
  canVaultUnlockAfterLogin,
  formatTwoFactorProviders,
  isValidServerUrl,
  loginWithPassword,
  normalizeBaseUrl,
  restoreLoginHints,
  sendEmailLoginCode,
  syncVaultAfterLogin,
  toProviderId,
  toServerUrlOption,
  unlockVaultAfterLogin,
} from "@/features/auth/login/login-flow-helpers";
import type {
  LoginFeedback,
  TwoFactorState,
} from "@/features/auth/login/types";
import { appI18n } from "@/i18n";
import { errorHandler } from "@/lib/error-handler";

type UseLoginFlowParams = {
  navigateToVault: () => Promise<void>;
};

type UseLoginFlowResult = {
  canSubmit: boolean;
  customBaseUrl: string;
  email: string;
  feedback: LoginFeedback;
  isRestoringSession: boolean;
  isSubmitting: boolean;
  masterPassword: string;
  onCustomBaseUrlChange: (value: string) => void;
  onEmailChange: (value: string) => void;
  onMasterPasswordChange: (value: string) => void;
  onSendEmailCode: () => Promise<void>;
  onServerUrlOptionChange: (value: string) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  onToggleShowPassword: () => void;
  onTwoFactorProviderChange: (value: string) => void;
  onTwoFactorTokenChange: (value: string) => void;
  serverUrlOption: string;
  showPassword: boolean;
  submitProgressText: string;
  twoFactorState: TwoFactorState | null;
};

export function useLoginFlow({
  navigateToVault,
}: UseLoginFlowParams): UseLoginFlowResult {
  const [customBaseUrl, setCustomBaseUrl] = useState("");
  const [serverUrlOption, setServerUrlOption] = useState<string>(
    CUSTOM_SERVER_URL_OPTION,
  );
  const [email, setEmail] = useState("");
  const [masterPassword, setMasterPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [feedback, setFeedback] = useState<LoginFeedback>({ kind: "idle" });
  const [submitProgressText, setSubmitProgressText] = useState("");
  const [isRestoringSession, setIsRestoringSession] = useState(true);
  const [twoFactorState, setTwoFactorState] = useState<TwoFactorState | null>(
    null,
  );

  useEffect(() => {
    let cancelled = false;

    const restoreSession = async () => {
      setIsRestoringSession(true);
      try {
        const result = await restoreLoginHints();
        if (cancelled) {
          return;
        }
        if (result.status === "error") {
          return;
        }

        const restored = result.data;

        if (restored.baseUrl) {
          const normalizedRestoredBaseUrl = normalizeBaseUrl(restored.baseUrl);
          const restoredOption = toServerUrlOption(normalizedRestoredBaseUrl);
          if (restoredOption === CUSTOM_SERVER_URL_OPTION) {
            setCustomBaseUrl((previous) =>
              previous.trim().length > 0 ? previous : normalizedRestoredBaseUrl,
            );
          }
          setServerUrlOption((previous) =>
            previous === CUSTOM_SERVER_URL_OPTION ? restoredOption : previous,
          );
        }
        if (restored.email) {
          setEmail((previous) =>
            previous.trim().length > 0 ? previous : (restored.email ?? ""),
          );
        }
      } catch {
        // ignored: first-screen hint should not block manual login
      } finally {
        if (!cancelled) {
          setIsRestoringSession(false);
        }
      }
    };

    void restoreSession();
    return () => {
      cancelled = true;
    };
  }, []);

  const effectiveBaseUrl =
    serverUrlOption === CUSTOM_SERVER_URL_OPTION
      ? customBaseUrl
      : serverUrlOption;

  const canSubmit = useMemo(
    () =>
      !isSubmitting &&
      !isRestoringSession &&
      normalizeBaseUrl(effectiveBaseUrl).length > 0 &&
      email.trim().length > 0 &&
      masterPassword.length > 0 &&
      (twoFactorState ? twoFactorState.token.trim().length > 0 : true),
    [
      email,
      effectiveBaseUrl,
      isRestoringSession,
      isSubmitting,
      masterPassword,
      twoFactorState,
    ],
  );

  const clearTwoFactorChallenge = () => {
    if (!twoFactorState) {
      return;
    }
    setTwoFactorState(null);
    if (feedback.kind === "twoFactor") {
      setFeedback({ kind: "idle" });
    }
  };

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    const normalizedBaseUrl = normalizeBaseUrl(effectiveBaseUrl);
    const trimmedEmail = email.trim();

    if (!normalizedBaseUrl || !trimmedEmail || !masterPassword) {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.login.validation.missingCredentials"),
      });
      return;
    }
    if (!isValidServerUrl(normalizedBaseUrl)) {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.login.validation.invalidServerUrl"),
      });
      return;
    }
    if (!trimmedEmail.includes("@")) {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.login.validation.invalidEmail"),
      });
      return;
    }

    setIsSubmitting(true);
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
        masterPassword,
        twoFactorProvider,
        twoFactorToken,
      });

      if (result.status === "error") {
        errorHandler.handle(result.error);
        return;
      }

      if (result.data.status === "authenticated") {
        setSubmitProgressText(appI18n.t("auth.login.progress.preparingVault"));
        setTwoFactorState(null);

        const canUnlockResult = await canVaultUnlockAfterLogin();
        if (canUnlockResult.status === "error") {
          errorHandler.handle(canUnlockResult.error);
          return;
        }
        const canUnlock = canUnlockResult.data;

        if (canUnlock) {
          setSubmitProgressText(
            appI18n.t("auth.login.progress.unlockingLocalVault"),
          );
          const unlockResult = await unlockVaultAfterLogin(masterPassword);
          if (unlockResult.status === "error") {
            errorHandler.handle(unlockResult.error);
            return;
          }

          setSubmitProgressText(
            appI18n.t("auth.login.progress.syncingLatestData"),
          );
          const syncResult = await syncVaultAfterLogin();
          if (syncResult.status === "error") {
            setMasterPassword("");
            await navigateToVault();
            return;
          }

          setMasterPassword("");
          await navigateToVault();
          return;
        }

        setSubmitProgressText(appI18n.t("auth.login.progress.firstSync"));
        const syncResult = await syncVaultAfterLogin();
        if (syncResult.status === "error") {
          errorHandler.handle(syncResult.error);
          return;
        }

        setSubmitProgressText(appI18n.t("auth.login.progress.finishingUnlock"));
        const unlockResult = await unlockVaultAfterLogin(masterPassword);
        if (unlockResult.status === "error") {
          errorHandler.handle(unlockResult.error);
          return;
        }

        setMasterPassword("");
        await navigateToVault();
        return;
      }

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
      setIsSubmitting(false);
      setSubmitProgressText("");
    }
  };

  const onSendEmailCode = async () => {
    if (!twoFactorState) {
      return;
    }
    if (twoFactorState.selectedProvider !== "1") {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.login.validation.nonEmailProvider"),
      });
      return;
    }

    const normalizedBaseUrl = normalizeBaseUrl(effectiveBaseUrl);
    const trimmedEmail = email.trim();
    if (!normalizedBaseUrl || !trimmedEmail || !masterPassword) {
      setFeedback({
        kind: "error",
        text: appI18n.t("auth.login.validation.missingEmailCodeRequirements"),
      });
      return;
    }

    setTwoFactorState((previous) =>
      previous ? { ...previous, isSendingEmailCode: true } : previous,
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
      setTwoFactorState((previous) =>
        previous ? { ...previous, isSendingEmailCode: false } : previous,
      );
    }
  };

  const onServerUrlOptionChange = (value: string) => {
    clearTwoFactorChallenge();
    setServerUrlOption(value);
  };

  const onCustomBaseUrlChange = (value: string) => {
    clearTwoFactorChallenge();
    setCustomBaseUrl(value);
  };

  const onEmailChange = (value: string) => {
    clearTwoFactorChallenge();
    setEmail(value);
  };

  const onMasterPasswordChange = (value: string) => {
    clearTwoFactorChallenge();
    setMasterPassword(value);
  };

  const onToggleShowPassword = () => {
    setShowPassword((previous) => !previous);
  };

  const onTwoFactorProviderChange = (value: string) => {
    setTwoFactorState((previous) =>
      previous
        ? {
            ...previous,
            selectedProvider: value,
            token: "",
          }
        : previous,
    );
  };

  const onTwoFactorTokenChange = (value: string) => {
    setTwoFactorState((previous) =>
      previous
        ? {
            ...previous,
            token: value,
          }
        : previous,
    );
  };

  return {
    canSubmit,
    customBaseUrl,
    email,
    feedback,
    isRestoringSession,
    isSubmitting,
    masterPassword,
    onCustomBaseUrlChange,
    onEmailChange,
    onMasterPasswordChange,
    onSendEmailCode,
    onServerUrlOptionChange,
    onSubmit,
    onToggleShowPassword,
    onTwoFactorProviderChange,
    onTwoFactorTokenChange,
    serverUrlOption,
    showPassword,
    submitProgressText,
    twoFactorState,
  };
}
