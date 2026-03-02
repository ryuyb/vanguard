import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import {
  Eye,
  EyeOff,
  Fingerprint,
  KeyRound,
  LoaderCircle,
  Lock,
  LogOut,
  ShieldCheck,
} from "lucide-react";
import type { FormEvent } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import lockedIllustration from "@/assets/locked.svg";
import { commands, type RestoreAuthStateResponseDto } from "@/bindings";
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

export const Route = createFileRoute("/unlock")({
  component: UnlockPage,
});

type UnlockFeedback =
  | { kind: "idle" }
  | { kind: "success"; text: string }
  | { kind: "error"; text: string };

function errorToText(error: unknown) {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "解锁失败，请稍后重试。";
}

function UnlockPage() {
  const navigate = useNavigate({ from: "/unlock" });
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
        setFeedback({ kind: "error", text: errorToText(result.error) });
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
      setFeedback({ kind: "error", text: errorToText(error) });
      setBiometricSupported(false);
      setBiometricEnabled(false);
      setCanBiometricUnlock(false);
      setIsVaultUnlocked(false);
    } finally {
      setIsRestoring(false);
    }
  }, []);

  useEffect(() => {
    loadRestoreState();
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
      const result = await commands.vaultUnlockWithPassword({
        masterPassword: trimmedPassword,
      });

      if (result.status === "error") {
        setFeedback({ kind: "error", text: errorToText(result.error) });
        return;
      }

      setMasterPassword("");
      await navigate({ to: "/vault" });
    } catch (error) {
      setFeedback({ kind: "error", text: errorToText(error) });
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
      const result = await commands.vaultUnlockWithBiometric();
      if (result.status === "error") {
        setFeedback({ kind: "error", text: errorToText(result.error) });
        return;
      }
      setMasterPassword("");
      await navigate({ to: "/vault" });
    } catch (error) {
      setFeedback({ kind: "error", text: errorToText(error) });
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
        setFeedback({ kind: "error", text: errorToText(result.error) });
        return;
      }
      setMasterPassword("");
      await navigate({ to: "/" });
    } catch (error) {
      setFeedback({ kind: "error", text: errorToText(error) });
    } finally {
      setIsLoggingOut(false);
    }
  };

  return (
    <main className="relative min-h-dvh overflow-hidden bg-[radial-gradient(circle_at_90%_15%,_hsl(210_85%_95%),_transparent_40%),radial-gradient(circle_at_12%_85%,_hsl(216_90%_97%),_transparent_45%),linear-gradient(160deg,_hsl(210_50%_98%),_hsl(0_0%_100%))] p-6 md:p-10">
      <div className="absolute -top-20 left-1/2 h-56 w-56 -translate-x-1/2 rounded-full bg-blue-300/15 blur-3xl" />

      <section className="relative mx-auto grid w-full max-w-5xl gap-8 md:min-h-[calc(100dvh-5rem)] md:grid-cols-[0.9fr_1.1fr] md:items-center">
        <div className="hidden rounded-3xl border border-white/80 bg-white/70 p-8 shadow-sm backdrop-blur md:flex md:flex-col md:gap-6">
          <Badge
            variant="outline"
            className="w-fit border-blue-200 bg-blue-50 text-blue-700"
          >
            Vault Unlock
          </Badge>
          <h1 className="text-3xl leading-tight font-semibold tracking-tight text-slate-900">
            会话已锁定，请输入主密码解锁
          </h1>
          <p className="text-sm leading-relaxed text-slate-600">
            输入主密码后即可解锁，继续安全访问你的密码库。
          </p>
          <img
            src={lockedIllustration}
            alt="Vault locked illustration"
            className="h-64 w-full object-contain"
          />
        </div>

        <Card className="border-white/80 bg-white/90 shadow-xl backdrop-blur-sm">
          <CardHeader className="space-y-2">
            <Badge variant="secondary" className="w-fit">
              Session Unlock
            </Badge>
            <CardTitle className="text-2xl font-semibold">解锁 Vault</CardTitle>
            <CardDescription>
              支持密码解锁与 Touch ID 解锁（需先显式启用）
            </CardDescription>
          </CardHeader>
          <CardContent>
            {isRestoring && (
              <div className="flex items-center gap-2 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">
                <LoaderCircle className="animate-spin" />
                正在检查当前会话状态...
              </div>
            )}

            {!isRestoring && restoreState?.status === "needsLogin" && (
              <div className="space-y-4">
                <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
                  当前没有可解锁的已登录会话，请先登录。
                </div>
                <Button asChild className="w-full">
                  <Link to="/">前往登录页</Link>
                </Button>
              </div>
            )}

            {!isRestoring &&
              restoreState?.status !== "needsLogin" &&
              isVaultUnlocked && (
                <div className="space-y-4">
                  <div className="rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
                    当前 Vault 已是解锁状态，无需再次输入 master password。
                  </div>
                  {restoreState?.status === "locked" && (
                    <div className="rounded-lg border border-blue-200 bg-blue-50 px-3 py-2 text-sm text-blue-800">
                      当前仅恢复了本地解锁状态，后端登录会话尚未恢复。
                    </div>
                  )}
                  {biometricSupported && (
                    <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">
                      Touch ID：{biometricEnabled ? "已启用" : "未启用"}
                    </div>
                  )}
                  {biometricSupported && !biometricEnabled && (
                    <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
                      如需启用或关闭 Touch ID，请进入 Vault 页面进行设置。
                    </div>
                  )}
                  <Button asChild className="w-full">
                    <Link to="/vault">查看 Vault 数据</Link>
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    className="w-full"
                    disabled={isLoggingOut}
                    onClick={onLogout}
                  >
                    {isLoggingOut ? (
                      <LoaderCircle className="animate-spin" />
                    ) : (
                      <LogOut />
                    )}
                    {isLoggingOut ? "正在登出..." : "登出"}
                  </Button>
                </div>
              )}

            {!isRestoring &&
              restoreState?.status !== "needsLogin" &&
              !isVaultUnlocked && (
                <form className="space-y-5" onSubmit={onUnlock}>
                  <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-600">
                    <div>登录邮箱：{restoreState?.email ?? "unknown"}</div>
                    <div>服务地址：{restoreState?.baseUrl ?? "unknown"}</div>
                  </div>
                  {biometricSupported && (
                    <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">
                      Touch ID：{biometricEnabled ? "已启用" : "未启用"}
                    </div>
                  )}

                  <div className="space-y-2">
                    <Label htmlFor="unlock-master-password">
                      Master Password
                    </Label>
                    <InputGroup>
                      <InputGroupAddon>
                        <KeyRound className="text-slate-500" />
                      </InputGroupAddon>
                      <InputGroupInput
                        id="unlock-master-password"
                        type={showPassword ? "text" : "password"}
                        autoComplete="current-password"
                        placeholder="输入主密码解锁"
                        value={masterPassword}
                        onChange={(event) =>
                          setMasterPassword(event.target.value)
                        }
                        disabled={
                          isUnlocking || isBiometricUnlocking || isLoggingOut
                        }
                      />
                      <InputGroupAddon align="inline-end" className="px-1.5">
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon-sm"
                          className="text-slate-500 hover:text-slate-900"
                          onClick={() =>
                            setShowPassword((previous) => !previous)
                          }
                          disabled={
                            isUnlocking || isBiometricUnlocking || isLoggingOut
                          }
                          aria-label={showPassword ? "隐藏密码" : "显示密码"}
                        >
                          {showPassword ? <EyeOff /> : <Eye />}
                        </Button>
                      </InputGroupAddon>
                    </InputGroup>
                  </div>

                  {feedback.kind !== "idle" && (
                    <div
                      className={[
                        "rounded-lg border px-3 py-2 text-sm",
                        feedback.kind === "error" &&
                          "border-red-200 bg-red-50 text-red-700",
                        feedback.kind === "success" &&
                          "border-emerald-200 bg-emerald-50 text-emerald-700",
                      ]
                        .filter(Boolean)
                        .join(" ")}
                    >
                      {feedback.kind === "success" && (
                        <ShieldCheck className="mr-1 inline size-4" />
                      )}
                      {feedback.kind === "error" && (
                        <Lock className="mr-1 inline size-4" />
                      )}
                      {feedback.text}
                    </div>
                  )}

                  <Button
                    type="submit"
                    size="lg"
                    className="w-full"
                    disabled={!canUnlock}
                  >
                    {isUnlocking && <LoaderCircle className="animate-spin" />}
                    {isUnlocking ? "正在解锁..." : "解锁密码库"}
                  </Button>

                  {canBiometricUnlock && (
                    <Button
                      type="button"
                      variant="outline"
                      size="lg"
                      className="w-full"
                      onClick={onBiometricUnlock}
                      disabled={
                        isUnlocking || isBiometricUnlocking || isLoggingOut
                      }
                    >
                      {isBiometricUnlocking ? (
                        <LoaderCircle className="animate-spin" />
                      ) : (
                        <Fingerprint />
                      )}
                      {isBiometricUnlocking
                        ? "正在等待 Touch ID..."
                        : "使用 Touch ID 解锁"}
                    </Button>
                  )}

                  {biometricSupported &&
                    biometricEnabled &&
                    !canBiometricUnlock && (
                      <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
                        Touch ID
                        已启用，但当前设备还没有可用于解锁的本地同步数据，请先完成一次同步并用密码解锁。
                      </div>
                    )}

                  <Button
                    type="button"
                    variant="outline"
                    className="w-full"
                    disabled={
                      isUnlocking || isBiometricUnlocking || isLoggingOut
                    }
                    onClick={onLogout}
                  >
                    {isLoggingOut ? (
                      <LoaderCircle className="animate-spin" />
                    ) : (
                      <LogOut />
                    )}
                    {isLoggingOut ? "正在登出..." : "登出"}
                  </Button>
                </form>
              )}
          </CardContent>
        </Card>
      </section>
    </main>
  );
}
