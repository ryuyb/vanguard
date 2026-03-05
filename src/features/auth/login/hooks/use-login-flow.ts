import type { FormEvent } from "react";
import { useEffect, useMemo, useState } from "react";
import { commands } from "@/bindings";
import { CUSTOM_SERVER_URL_OPTION } from "@/features/auth/login/constants";
import type {
  LoginFeedback,
  TwoFactorState,
} from "@/features/auth/login/types";
import {
  isValidServerUrl,
  normalizeBaseUrl,
  toProviderId,
  toProviderLabel,
  toServerUrlOption,
} from "@/features/auth/login/utils";
import { toErrorText } from "@/features/auth/shared/utils";

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

function toLoginErrorText(error: unknown): string {
  return toErrorText(error, "登录失败，请稍后重试。");
}

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
        const result = await commands.authRestoreState({});
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
        text: "请先填写服务地址、登录邮箱和主密码。",
      });
      return;
    }
    if (!isValidServerUrl(normalizedBaseUrl)) {
      setFeedback({
        kind: "error",
        text: "服务地址格式不正确，请以 http:// 或 https:// 开头。",
      });
      return;
    }
    if (!trimmedEmail.includes("@")) {
      setFeedback({
        kind: "error",
        text: "邮箱格式看起来不正确，请检查后重试。",
      });
      return;
    }

    setIsSubmitting(true);
    setFeedback({ kind: "idle" });
    setSubmitProgressText("正在验证账号信息...");

    try {
      const twoFactorProvider = twoFactorState
        ? toProviderId(twoFactorState.selectedProvider)
        : null;
      const twoFactorToken = twoFactorState?.token.trim() || null;

      if (twoFactorState && (twoFactorProvider === null || !twoFactorToken)) {
        setFeedback({
          kind: "error",
          text: "请输入完整的二步验证码后再继续。",
        });
        return;
      }

      const result = await commands.authLoginWithPassword({
        baseUrl: normalizedBaseUrl,
        email: trimmedEmail,
        masterPassword,
        twoFactorProvider,
        twoFactorToken,
        twoFactorRemember: false,
        authrequest: null,
      });

      if (result.status === "error") {
        setFeedback({ kind: "error", text: toLoginErrorText(result.error) });
        return;
      }

      if (result.data.status === "authenticated") {
        setSubmitProgressText("正在准备你的密码库...");
        setTwoFactorState(null);

        const canUnlockResult = await commands.vaultCanUnlock();
        if (canUnlockResult.status === "error") {
          setFeedback({
            kind: "error",
            text: `你已登录成功，但暂时无法判断解锁状态：${toLoginErrorText(canUnlockResult.error)}`,
          });
          return;
        }
        const canUnlock = canUnlockResult.data;

        if (canUnlock) {
          setSubmitProgressText("正在解锁本地密码库...");
          const unlockResult = await commands.vaultUnlock({
            method: {
              type: "masterPassword",
              password: masterPassword,
            },
          });
          if (unlockResult.status === "error") {
            setFeedback({
              kind: "error",
              text: `你已登录成功，但解锁失败：${toLoginErrorText(unlockResult.error)}`,
            });
            return;
          }

          setSubmitProgressText("正在同步最新数据...");
          const syncResult = await commands.vaultSyncNow({
            excludeDomains: false,
          });
          if (syncResult.status === "error") {
            setMasterPassword("");
            await navigateToVault();
            return;
          }

          setMasterPassword("");
          await navigateToVault();
          return;
        }

        setSubmitProgressText("正在首次同步密码库...");
        const syncResult = await commands.vaultSyncNow({
          excludeDomains: false,
        });
        if (syncResult.status === "error") {
          setFeedback({
            kind: "error",
            text: `你已登录成功，但首次同步失败：${toLoginErrorText(syncResult.error)}`,
          });
          return;
        }

        setSubmitProgressText("正在完成解锁...");
        const unlockResult = await commands.vaultUnlock({
          method: {
            type: "masterPassword",
            password: masterPassword,
          },
        });
        if (unlockResult.status === "error") {
          setFeedback({
            kind: "error",
            text: `首次同步已完成，但解锁失败：${toLoginErrorText(unlockResult.error)}`,
          });
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
        text: `需要二步验证，请输入验证码继续（可用方式：${providers.map(toProviderLabel).join("、") || "未知方式"}）。`,
      });
    } catch (error) {
      setFeedback({ kind: "error", text: toLoginErrorText(error) });
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
        text: "当前不是邮箱验证方式，无法发送邮件验证码。",
      });
      return;
    }

    const normalizedBaseUrl = normalizeBaseUrl(effectiveBaseUrl);
    const trimmedEmail = email.trim();
    if (!normalizedBaseUrl || !trimmedEmail || !masterPassword) {
      setFeedback({
        kind: "error",
        text: "发送验证码前，请先填写服务地址、登录邮箱和主密码。",
      });
      return;
    }

    setTwoFactorState((previous) =>
      previous ? { ...previous, isSendingEmailCode: true } : previous,
    );
    setFeedback({ kind: "idle" });

    try {
      const result = await commands.authSendEmailLogin({
        baseUrl: normalizedBaseUrl,
        email: trimmedEmail,
        masterPassword,
        authRequestId: null,
        authRequestAccessCode: null,
      });

      if (result.status === "error") {
        setFeedback({ kind: "error", text: toLoginErrorText(result.error) });
        return;
      }

      setFeedback({
        kind: "success",
        text: "验证码已发送到邮箱，请查收后输入并继续登录。",
      });
    } catch (error) {
      setFeedback({ kind: "error", text: toLoginErrorText(error) });
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
