import { Link } from "@tanstack/react-router";
import { LoaderCircle } from "lucide-react";
import { useCallback, useEffect } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  UnlockHero,
  UnlockLockedForm,
} from "@/features/auth/unlock/components";
import { useUnlockFlow } from "@/features/auth/unlock/hooks";

type UnlockPageProps = {
  navigateToHome: () => Promise<void>;
  navigateToVault: () => Promise<void>;
};

export function UnlockPage({
  navigateToHome,
  navigateToVault,
}: UnlockPageProps) {
  const {
    biometricEnabled,
    biometricSupported,
    canBiometricUnlock,
    canPinUnlock,
    canUnlock,
    feedback,
    isBiometricUnlocking,
    isLoggingOut,
    isPinUnlocking,
    isRestoring,
    isUnlocking,
    isVaultUnlocked,
    masterPassword,
    onBiometricUnlock,
    onLogout,
    onMasterPasswordChange,
    onPinChange,
    onPinUnlock,
    onShowMasterPasswordUnlock,
    onShowPinUnlock,
    onToggleShowPassword,
    onUnlock,
    pin,
    pinEnabled,
    restoreState,
    showPassword,
    unlockMethod,
  } = useUnlockFlow({
    navigateToHome,
    navigateToVault,
  });

  const redirectToVaultIfUnlocked = useCallback(async () => {
    await navigateToVault();
  }, [navigateToVault]);

  useEffect(() => {
    if (
      !isRestoring &&
      restoreState?.status !== "needsLogin" &&
      isVaultUnlocked
    ) {
      void redirectToVaultIfUnlocked();
    }
  }, [
    isRestoring,
    isVaultUnlocked,
    redirectToVaultIfUnlocked,
    restoreState?.status,
  ]);

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
                <div className="flex items-center gap-2 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">
                  <LoaderCircle className="animate-spin" />
                  当前 Vault 已解锁，正在跳转...
                </div>
              )}

            {!isRestoring &&
              restoreState?.status !== "needsLogin" &&
              !isVaultUnlocked && (
                <UnlockLockedForm
                  restoreState={restoreState}
                  biometricSupported={biometricSupported}
                  biometricEnabled={biometricEnabled}
                  canBiometricUnlock={canBiometricUnlock}
                  canPinUnlock={canPinUnlock}
                  canUnlock={canUnlock}
                  feedback={feedback}
                  isBiometricUnlocking={isBiometricUnlocking}
                  isLoggingOut={isLoggingOut}
                  isPinUnlocking={isPinUnlocking}
                  isUnlocking={isUnlocking}
                  masterPassword={masterPassword}
                  onBiometricUnlock={onBiometricUnlock}
                  onLogout={onLogout}
                  onMasterPasswordChange={onMasterPasswordChange}
                  onPinChange={onPinChange}
                  onPinUnlock={onPinUnlock}
                  onShowMasterPasswordUnlock={onShowMasterPasswordUnlock}
                  onShowPinUnlock={onShowPinUnlock}
                  onSubmit={onUnlock}
                  onToggleShowPassword={onToggleShowPassword}
                  pin={pin}
                  pinEnabled={pinEnabled}
                  showPassword={showPassword}
                  unlockMethod={unlockMethod}
                />
              )}
          </CardContent>
        </Card>
      </section>
    </main>
  );
}
