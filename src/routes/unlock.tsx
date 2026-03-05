import {
  createFileRoute,
  Link,
  redirect,
  useNavigate,
} from "@tanstack/react-router";
import { LoaderCircle } from "lucide-react";
import type { FormEvent } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
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
import { toErrorText } from "@/features/auth/shared/utils";
import { UnlockHero } from "@/features/auth/unlock/components/unlock-hero";
import { UnlockLockedForm } from "@/features/auth/unlock/components/unlock-locked-form";
import { UnlockUnlockedState } from "@/features/auth/unlock/components/unlock-unlocked-state";
import type { UnlockFeedback } from "@/features/auth/unlock/types";
import { resolveSessionRoute } from "@/lib/route-session";

export const Route = createFileRoute("/unlock")({
  beforeLoad: async () => {
    const target = await resolveSessionRoute();
    if (target !== "/unlock") {
      throw redirect({ to: target });
    }
  },
  component: UnlockPage,
});

function toUnlockErrorText(error: unknown) {
  return toErrorText(error, "解锁失败，请稍后重试。");
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
      await navigate({ to: "/vault" });
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
      await navigate({ to: "/vault" });
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
      await navigate({ to: "/" });
    } catch (error) {
      setFeedback({ kind: "error", text: toUnlockErrorText(error) });
    } finally {
      setIsLoggingOut(false);
    }
  };

  return (
    <main className="relative min-h-dvh overflow-hidden bg-[radial-gradient(circle_at_90%_15%,hsl(210_85%_95%),transparent_40%),radial-gradient(circle_at_12%_85%,hsl(216_90%_97%),transparent_45%),linear-gradient(160deg,hsl(210_50%_98%),hsl(0_0%_100%))] p-6 md:p-10">
      <div
        data-tauri-drag-region
        className="absolute inset-x-0 top-0 z-20 h-6"
      />
      <div className="absolute -top-20 left-1/2 h-56 w-56 -translate-x-1/2 rounded-full bg-blue-300/15 blur-3xl" />

      <section className="relative mx-auto grid w-full max-w-5xl gap-8 md:min-h-[calc(100dvh-5rem)] md:grid-cols-[0.9fr_1.1fr] md:items-center">
        <UnlockHero />

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
                <UnlockUnlockedState
                  restoreState={restoreState}
                  biometricSupported={biometricSupported}
                  biometricEnabled={biometricEnabled}
                  isLoggingOut={isLoggingOut}
                  onLogout={onLogout}
                />
              )}

            {!isRestoring &&
              restoreState?.status !== "needsLogin" &&
              !isVaultUnlocked && (
                <UnlockLockedForm
                  restoreState={restoreState}
                  biometricSupported={biometricSupported}
                  biometricEnabled={biometricEnabled}
                  canBiometricUnlock={canBiometricUnlock}
                  masterPassword={masterPassword}
                  showPassword={showPassword}
                  isUnlocking={isUnlocking}
                  isBiometricUnlocking={isBiometricUnlocking}
                  isLoggingOut={isLoggingOut}
                  canUnlock={canUnlock}
                  feedback={feedback}
                  onMasterPasswordChange={setMasterPassword}
                  onToggleShowPassword={() =>
                    setShowPassword((previous) => !previous)
                  }
                  onSubmit={onUnlock}
                  onBiometricUnlock={onBiometricUnlock}
                  onLogout={onLogout}
                />
              )}
          </CardContent>
        </Card>
      </section>
    </main>
  );
}
