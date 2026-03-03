import { createFileRoute, redirect, useNavigate } from "@tanstack/react-router";
import {
  Eye,
  EyeOff,
  Globe,
  KeyRound,
  LoaderCircle,
  Mail,
  Send,
  ShieldCheck,
} from "lucide-react";
import type { FormEvent } from "react";
import { useEffect, useMemo, useState } from "react";
import loginIllustration from "@/assets/login.svg";
import { commands } from "@/bindings";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { resolveSessionRoute } from "@/lib/route-session";

export const Route = createFileRoute("/")({
  beforeLoad: async () => {
    const target = await resolveSessionRoute();
    if (target !== "/") {
      throw redirect({ to: target });
    }
  },
  component: Index,
});

type LoginFeedback =
  | { kind: "idle" }
  | { kind: "success"; text: string }
  | { kind: "twoFactor"; text: string }
  | { kind: "error"; text: string };

type TwoFactorState = {
  providers: string[];
  selectedProvider: string;
  token: string;
  isSendingEmailCode: boolean;
};

const CUSTOM_SERVER_URL_OPTION = "__custom__";

const SERVER_URL_OPTIONS = [
  {
    value: "https://bitwarden.com",
    label: "Bitwarden.com",
  },
  {
    value: "https://bitwarden.eu",
    label: "Bitwarden.eu",
  },
] as const;

function toServerUrlOption(value: string) {
  const normalized = normalizeBaseUrl(value);
  const matched = SERVER_URL_OPTIONS.find(
    (option) => option.value === normalized,
  );
  return matched ? matched.value : CUSTOM_SERVER_URL_OPTION;
}

const TWO_FACTOR_PROVIDER_LABELS: Record<string, string> = {
  "0": "Authenticator",
  "1": "Email",
  "2": "Duo",
  "3": "YubiKey",
  "5": "Remember",
  "7": "WebAuthn",
  "8": "Recovery Code",
};

function normalizeBaseUrl(value: string) {
  return value.trim().replace(/\/+$/, "");
}

function isValidServerUrl(value: string) {
  try {
    const parsed = new URL(value);
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch {
    return false;
  }
}

function toProviderLabel(provider: string) {
  return TWO_FACTOR_PROVIDER_LABELS[provider] ?? `Provider ${provider}`;
}

function toProviderId(provider: string) {
  const parsed = Number.parseInt(provider, 10);
  if (Number.isNaN(parsed)) {
    return null;
  }
  return parsed;
}

function errorToText(error: unknown) {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "登录失败，请稍后重试。";
}

function Index() {
  const navigate = useNavigate({ from: "/" });
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

    restoreSession();
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
        setFeedback({ kind: "error", text: errorToText(result.error) });
        return;
      }

      if (result.data.status === "authenticated") {
        setSubmitProgressText("正在准备你的密码库...");
        setTwoFactorState(null);

        const canUnlockResult = await commands.vaultCanUnlock();
        if (canUnlockResult.status === "error") {
          setFeedback({
            kind: "error",
            text: `你已登录成功，但暂时无法判断解锁状态：${errorToText(canUnlockResult.error)}`,
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
              text: `你已登录成功，但解锁失败：${errorToText(unlockResult.error)}`,
            });
            return;
          }

          setSubmitProgressText("正在同步最新数据...");
          const syncResult = await commands.vaultSyncNow({
            excludeDomains: false,
          });
          if (syncResult.status === "error") {
            setMasterPassword("");
            await navigate({ to: "/vault" });
            return;
          }

          setMasterPassword("");
          await navigate({ to: "/vault" });
          return;
        }

        setSubmitProgressText("正在首次同步密码库...");
        const syncResult = await commands.vaultSyncNow({
          excludeDomains: false,
        });
        if (syncResult.status === "error") {
          setFeedback({
            kind: "error",
            text: `你已登录成功，但首次同步失败：${errorToText(syncResult.error)}`,
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
            text: `首次同步已完成，但解锁失败：${errorToText(unlockResult.error)}`,
          });
          return;
        }

        setMasterPassword("");
        await navigate({ to: "/vault" });
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
      setFeedback({ kind: "error", text: errorToText(error) });
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
        setFeedback({ kind: "error", text: errorToText(result.error) });
        return;
      }

      setFeedback({
        kind: "success",
        text: "验证码已发送到邮箱，请查收后输入并继续登录。",
      });
    } catch (error) {
      setFeedback({ kind: "error", text: errorToText(error) });
    } finally {
      setTwoFactorState((previous) =>
        previous ? { ...previous, isSendingEmailCode: false } : previous,
      );
    }
  };

  return (
    <main className="relative min-h-dvh overflow-hidden bg-[radial-gradient(circle_at_15%_15%,_hsl(219_100%_97%),_transparent_45%),radial-gradient(circle_at_85%_8%,_hsl(210_100%_96%),_transparent_40%),linear-gradient(130deg,_hsl(220_46%_98%),_hsl(0_0%_100%))] p-6 md:p-10">
      <div
        data-tauri-drag-region
        className="absolute inset-x-0 top-0 z-20 h-6"
      />
      <div className="absolute -top-24 -right-16 h-64 w-64 rounded-full bg-sky-300/15 blur-3xl" />
      <div className="absolute -bottom-28 -left-10 h-72 w-72 rounded-full bg-blue-500/10 blur-3xl" />

      <section className="relative mx-auto grid w-full max-w-6xl gap-8 md:min-h-[calc(100dvh-5rem)] md:grid-cols-[1.2fr_0.8fr] md:items-center">
        <div className="hidden rounded-3xl border border-white/70 bg-white/75 p-10 shadow-sm backdrop-blur md:flex md:flex-col md:gap-8">
          <Badge
            variant="outline"
            className="w-fit border-blue-200 bg-blue-50 text-blue-700"
          >
            Vanguard Vault
          </Badge>
          <div className="space-y-4">
            <h1 className="text-4xl leading-tight font-semibold tracking-tight text-slate-900">
              欢迎回来，继续管理你的密码库
            </h1>
            <p className="text-base leading-relaxed text-slate-600">
              输入服务地址、邮箱和主密码后，即可完成登录并自动准备好你的密码库。
            </p>
          </div>
          <img
            src={loginIllustration}
            alt="Vault login illustration"
            className="h-72 w-full object-contain"
          />
        </div>

        <Card className="border-white/70 bg-white/90 shadow-xl backdrop-blur-sm">
          <CardHeader className="space-y-2">
            <Badge variant="secondary" className="w-fit">
              登录
            </Badge>
            <CardTitle className="text-2xl font-semibold">
              登录你的 Vaultwarden 账号
            </CardTitle>
            <CardDescription>
              支持二步验证，登录后会自动准备密码库
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form className="space-y-5" onSubmit={onSubmit}>
              {isRestoringSession && (
                <div className="flex items-center gap-2 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">
                  <LoaderCircle className="animate-spin" />
                  正在检查上次会话...
                </div>
              )}

              <div className="space-y-2">
                <Label htmlFor="base-url">服务地址</Label>
                <Select
                  value={serverUrlOption}
                  onValueChange={(value) => {
                    clearTwoFactorChallenge();
                    setServerUrlOption(value);
                  }}
                  disabled={isSubmitting}
                >
                  <SelectTrigger id="base-url" className="w-full bg-white">
                    <SelectValue placeholder="选择服务地址" />
                  </SelectTrigger>
                  <SelectContent>
                    {SERVER_URL_OPTIONS.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                    <SelectItem value={CUSTOM_SERVER_URL_OPTION}>
                      自定义地址
                    </SelectItem>
                  </SelectContent>
                </Select>

                {serverUrlOption === CUSTOM_SERVER_URL_OPTION && (
                  <InputGroup>
                    <InputGroupAddon>
                      <Globe className="text-slate-500" />
                    </InputGroupAddon>
                    <InputGroupInput
                      id="base-url-custom"
                      type="url"
                      autoComplete="url"
                      placeholder="https://vault.example.com"
                      value={customBaseUrl}
                      onChange={(event) => {
                        clearTwoFactorChallenge();
                        setCustomBaseUrl(event.target.value);
                      }}
                      disabled={isSubmitting}
                    />
                  </InputGroup>
                )}
              </div>

              <div className="space-y-2">
                <Label htmlFor="email">登录邮箱</Label>
                <InputGroup>
                  <InputGroupAddon>
                    <Mail className="text-slate-500" />
                  </InputGroupAddon>
                  <InputGroupInput
                    id="email"
                    type="email"
                    autoComplete="email"
                    placeholder="you@example.com"
                    value={email}
                    onChange={(event) => {
                      clearTwoFactorChallenge();
                      setEmail(event.target.value);
                    }}
                    disabled={isSubmitting}
                  />
                </InputGroup>
              </div>

              <div className="space-y-2">
                <Label htmlFor="master-password">主密码</Label>
                <InputGroup>
                  <InputGroupAddon>
                    <KeyRound className="text-slate-500" />
                  </InputGroupAddon>
                  <InputGroupInput
                    id="master-password"
                    type={showPassword ? "text" : "password"}
                    autoComplete="current-password"
                    placeholder="输入主密码"
                    value={masterPassword}
                    onChange={(event) => {
                      clearTwoFactorChallenge();
                      setMasterPassword(event.target.value);
                    }}
                    disabled={isSubmitting}
                  />
                  <InputGroupAddon align="inline-end" className="px-1.5">
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon-sm"
                      className="text-slate-500 hover:text-slate-900"
                      onClick={() => setShowPassword((previous) => !previous)}
                      disabled={isSubmitting}
                      aria-label={showPassword ? "隐藏密码" : "显示密码"}
                    >
                      {showPassword ? <EyeOff /> : <Eye />}
                    </Button>
                  </InputGroupAddon>
                </InputGroup>
              </div>

              {twoFactorState && (
                <div className="space-y-4 rounded-xl border border-amber-200 bg-amber-50/80 p-4">
                  <div className="flex items-center gap-2 text-sm font-medium text-amber-800">
                    <ShieldCheck className="size-4" />
                    二步验证
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="two-factor-provider">验证方式</Label>
                    <Select
                      value={twoFactorState.selectedProvider}
                      onValueChange={(value) =>
                        setTwoFactorState((previous) =>
                          previous
                            ? {
                                ...previous,
                                selectedProvider: value,
                                token: "",
                              }
                            : previous,
                        )
                      }
                      disabled={
                        isSubmitting || twoFactorState.isSendingEmailCode
                      }
                    >
                      <SelectTrigger
                        id="two-factor-provider"
                        className="w-full bg-white"
                      >
                        <SelectValue placeholder="选择验证方式" />
                      </SelectTrigger>
                      <SelectContent>
                        {twoFactorState.providers.map((provider) => (
                          <SelectItem key={provider} value={provider}>
                            {toProviderLabel(provider)}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="two-factor-token">二步验证码</Label>
                    <InputGroup>
                      <InputGroupAddon>
                        <ShieldCheck className="text-slate-500" />
                      </InputGroupAddon>
                      <InputGroupInput
                        id="two-factor-token"
                        type="text"
                        inputMode="numeric"
                        autoComplete="one-time-code"
                        placeholder="输入验证码"
                        value={twoFactorState.token}
                        onChange={(event) =>
                          setTwoFactorState((previous) =>
                            previous
                              ? {
                                  ...previous,
                                  token: event.target.value,
                                }
                              : previous,
                          )
                        }
                        disabled={
                          isSubmitting || twoFactorState.isSendingEmailCode
                        }
                      />
                    </InputGroup>
                  </div>

                  {twoFactorState.selectedProvider === "1" && (
                    <Button
                      type="button"
                      variant="outline"
                      className="w-full"
                      disabled={
                        isSubmitting || twoFactorState.isSendingEmailCode
                      }
                      onClick={onSendEmailCode}
                    >
                      {twoFactorState.isSendingEmailCode && (
                        <LoaderCircle className="animate-spin" />
                      )}
                      {!twoFactorState.isSendingEmailCode && <Send />}
                      {twoFactorState.isSendingEmailCode
                        ? "正在发送邮箱验证码..."
                        : "发送邮箱验证码"}
                    </Button>
                  )}
                </div>
              )}

              {feedback.kind !== "idle" && (
                <div
                  className={[
                    "rounded-lg border px-3 py-2 text-sm",
                    feedback.kind === "error" &&
                      "border-red-200 bg-red-50 text-red-700",
                    feedback.kind === "success" &&
                      "border-emerald-200 bg-emerald-50 text-emerald-700",
                    feedback.kind === "twoFactor" &&
                      "border-amber-200 bg-amber-50 text-amber-700",
                  ]
                    .filter(Boolean)
                    .join(" ")}
                >
                  {feedback.kind === "twoFactor" && (
                    <ShieldCheck className="mr-1 inline size-4" />
                  )}
                  {feedback.text}
                </div>
              )}

              <Button
                type="submit"
                size="lg"
                className="w-full"
                disabled={!canSubmit}
              >
                {isSubmitting && <LoaderCircle className="animate-spin" />}
                {isSubmitting
                  ? submitProgressText || "正在登录..."
                  : twoFactorState
                    ? "验证后继续"
                    : "登录并进入密码库"}
              </Button>
            </form>
          </CardContent>
        </Card>
      </section>
    </main>
  );
}
