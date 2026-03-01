import { createFileRoute, Link } from "@tanstack/react-router";
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
import { useMemo, useState } from "react";
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

export const Route = createFileRoute("/")({
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
  const [baseUrl, setBaseUrl] = useState("https://vault.example.com");
  const [email, setEmail] = useState("");
  const [masterPassword, setMasterPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [feedback, setFeedback] = useState<LoginFeedback>({ kind: "idle" });
  const [twoFactorState, setTwoFactorState] = useState<TwoFactorState | null>(
    null,
  );

  const canSubmit = useMemo(
    () =>
      !isSubmitting &&
      normalizeBaseUrl(baseUrl).length > 0 &&
      email.trim().length > 0 &&
      masterPassword.length > 0 &&
      (twoFactorState ? twoFactorState.token.trim().length > 0 : true),
    [baseUrl, email, isSubmitting, masterPassword, twoFactorState],
  );

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    const normalizedBaseUrl = normalizeBaseUrl(baseUrl);
    const trimmedEmail = email.trim();

    if (!normalizedBaseUrl || !trimmedEmail || !masterPassword) {
      setFeedback({
        kind: "error",
        text: "请填写 server url、email 和 master password。",
      });
      return;
    }

    setIsSubmitting(true);
    setFeedback({ kind: "idle" });

    try {
      const twoFactorProvider = twoFactorState
        ? toProviderId(twoFactorState.selectedProvider)
        : null;
      const twoFactorToken = twoFactorState?.token.trim() || null;

      if (twoFactorState && (twoFactorProvider === null || !twoFactorToken)) {
        setFeedback({ kind: "error", text: "请输入有效的二步验证码。" });
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
        setTwoFactorState(null);

        const canUnlockResult = await commands.vaultCanUnlock();
        if (canUnlockResult.status === "error") {
          setFeedback({
            kind: "error",
            text: `登录成功，但无法判断本地解锁能力：${errorToText(canUnlockResult.error)}`,
          });
          return;
        }
        const canUnlock = canUnlockResult.data;

        if (canUnlock) {
          const unlockResult = await commands.vaultUnlockWithPassword({
            masterPassword,
          });
          if (unlockResult.status === "error") {
            setFeedback({
              kind: "error",
              text: `登录成功，但解锁失败：${errorToText(unlockResult.error)}`,
            });
            return;
          }

          const syncResult = await commands.vaultSyncNow({
            excludeDomains: false,
          });
          if (syncResult.status === "error") {
            setFeedback({
              kind: "error",
              text: `登录与解锁成功，但同步失败：${errorToText(syncResult.error)}`,
            });
            return;
          }

          setFeedback({
            kind: "success",
            text: "登录成功，检测到本地同步历史，已先解锁再同步。",
          });
          return;
        }

        const syncResult = await commands.vaultSyncNow({
          excludeDomains: false,
        });
        if (syncResult.status === "error") {
          setFeedback({
            kind: "error",
            text: `登录成功，但首次同步失败：${errorToText(syncResult.error)}`,
          });
          return;
        }

        const unlockResult = await commands.vaultUnlockWithPassword({
          masterPassword,
        });
        if (unlockResult.status === "error") {
          setFeedback({
            kind: "error",
            text: `首次同步成功，但解锁失败：${errorToText(unlockResult.error)}`,
          });
          return;
        }

        setFeedback({
          kind: "success",
          text: "登录成功，无本地同步历史，已先同步再解锁。",
        });
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
        text: `账号需要二步验证，请继续完成验证（可用方式：${providers.map(toProviderLabel).join("、") || "未知方式"}）。`,
      });
    } catch (error) {
      setFeedback({ kind: "error", text: errorToText(error) });
    } finally {
      setIsSubmitting(false);
    }
  };

  const onSendEmailCode = async () => {
    if (!twoFactorState) {
      return;
    }
    if (twoFactorState.selectedProvider !== "1") {
      setFeedback({
        kind: "error",
        text: "当前验证方式不是 Email 2FA，无法发送邮件验证码。",
      });
      return;
    }

    const normalizedBaseUrl = normalizeBaseUrl(baseUrl);
    const trimmedEmail = email.trim();
    if (!normalizedBaseUrl || !trimmedEmail || !masterPassword) {
      setFeedback({
        kind: "error",
        text: "发送验证码前，请确保 server url、email 和 master password 已填写。",
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
        text: "已发送邮箱验证码，请查收后输入二步验证码继续登录。",
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
              本地优先的 Vaultwarden 密码管理器
            </h1>
            <p className="text-base leading-relaxed text-slate-600">
              仅输入服务地址、邮箱和主密码。密钥推导、会话管理和同步都在 Tauri
              Rust 后端完成。
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
              Secure Login
            </Badge>
            <CardTitle className="text-2xl font-semibold">
              登录 Vaultwarden
            </CardTitle>
            <CardDescription>
              调用 Tauri `auth_login_with_password` command 完成登录
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form className="space-y-5" onSubmit={onSubmit}>
              <div className="space-y-2">
                <Label htmlFor="base-url">Server URL</Label>
                <InputGroup>
                  <InputGroupAddon>
                    <Globe className="text-slate-500" />
                  </InputGroupAddon>
                  <InputGroupInput
                    id="base-url"
                    type="url"
                    autoComplete="url"
                    placeholder="https://vault.example.com"
                    value={baseUrl}
                    onChange={(event) => setBaseUrl(event.target.value)}
                    disabled={isSubmitting}
                  />
                </InputGroup>
              </div>

              <div className="space-y-2">
                <Label htmlFor="email">Email</Label>
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
                    onChange={(event) => setEmail(event.target.value)}
                    disabled={isSubmitting}
                  />
                </InputGroup>
              </div>

              <div className="space-y-2">
                <Label htmlFor="master-password">Master Password</Label>
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
                    onChange={(event) => setMasterPassword(event.target.value)}
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
                        : "发送 Email 验证码"}
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
                  ? "正在登录..."
                  : twoFactorState
                    ? "验证并登录"
                    : "登录并同步"}
              </Button>

              <Button asChild variant="ghost" className="w-full">
                <Link to="/unlock">已登录但锁定？前往解锁页</Link>
              </Button>

              <Button asChild variant="ghost" className="w-full">
                <Link to="/vault">查看 Vault 数据</Link>
              </Button>
            </form>
          </CardContent>
        </Card>
      </section>
    </main>
  );
}
