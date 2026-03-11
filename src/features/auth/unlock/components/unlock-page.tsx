import { Link } from "@tanstack/react-router";
import { KeyRound, LoaderCircle } from "lucide-react";
import { useCallback, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { UnlockLockedForm } from "@/features/auth/unlock/components";
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
    <main className="relative flex min-h-dvh items-center justify-center overflow-hidden bg-gradient-to-br from-slate-50 via-blue-50/30 to-slate-100 p-6">
      <div
        data-tauri-drag-region
        className="absolute inset-x-0 top-0 z-20 h-6"
      />

      <section className="relative mx-auto w-full max-w-md">
        <Card className="border-slate-200/60 bg-white shadow-2xl shadow-slate-900/5">
          <CardHeader className="space-y-4 pb-6">
            <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-600 to-blue-700 shadow-lg shadow-blue-600/25">
              <KeyRound className="h-8 w-8 text-white" strokeWidth={2.5} />
            </div>
            <div className="space-y-1 text-center">
              <CardTitle className="text-2xl font-semibold tracking-tight text-slate-900">
                解锁 Vanguard
              </CardTitle>
              <p className="text-sm text-slate-600">
                输入凭据以访问你的密码库
              </p>
            </div>
          </CardHeader>
          <CardContent className="pb-8">
            {isRestoring && (
              <div className="flex items-center justify-center gap-3 rounded-xl border border-slate-200 bg-slate-50/50 px-4 py-6 text-sm text-slate-700">
                <LoaderCircle className="h-5 w-5 animate-spin text-blue-600" />
                <span>正在检查会话状态...</span>
              </div>
            )}

            {!isRestoring && restoreState?.status === "needsLogin" && (
              <div className="space-y-4">
                <div className="rounded-xl border border-amber-200/60 bg-amber-50/50 px-4 py-3.5 text-sm text-amber-900">
                  <p className="font-medium">需要登录</p>
                  <p className="mt-1 text-amber-800">
                    当前没有可解锁的会话，请先登录。
                  </p>
                </div>
                <Button asChild className="w-full" size="lg">
                  <Link to="/">前往登录</Link>
                </Button>
              </div>
            )}

            {!isRestoring &&
              restoreState?.status !== "needsLogin" &&
              isVaultUnlocked && (
                <div className="flex items-center justify-center gap-3 rounded-xl border border-emerald-200 bg-emerald-50/50 px-4 py-6 text-sm text-emerald-700">
                  <LoaderCircle className="h-5 w-5 animate-spin text-emerald-600" />
                  <span>密码库已解锁，正在跳转...</span>
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
