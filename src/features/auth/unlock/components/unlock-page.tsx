import { Link } from "@tanstack/react-router";
import { KeyRound, LoaderCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
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
  const { t } = useTranslation();
  const {
    form,
    account,
    biometricEnabled,
    biometricSupported,
    canBiometricUnlock,
    feedback,
    isActionBlocked,
    isBiometricUnlocking,
    isLoggingOut,
    isPinUnlocking,
    isRestoring,
    isVaultUnlocked,
    needsLogin,
    onBiometricUnlock,
    onLogout,
    onPinUnlock,
    onShowMasterPasswordUnlock,
    onShowPinUnlock,
    onToggleShowPassword,
    pinEnabled,
    showPassword,
    unlockMethod,
  } = useUnlockFlow({
    navigateToHome,
    navigateToVault,
  });

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
                {t("auth.unlock.title")}
              </CardTitle>
              <p className="text-sm text-slate-600">
                {t("auth.unlock.subtitle")}
              </p>
            </div>
          </CardHeader>
          <CardContent className="pb-8">
            {isRestoring && (
              <div className="flex items-center justify-center gap-3 rounded-xl border border-slate-200 bg-slate-50/50 px-4 py-6 text-sm text-slate-700">
                <LoaderCircle className="h-5 w-5 animate-spin text-blue-600" />
                <span>{t("auth.unlock.states.checkingSession")}</span>
              </div>
            )}

            {!isRestoring && needsLogin && (
              <div className="space-y-4">
                <div className="rounded-xl border border-amber-200/60 bg-amber-50/50 px-4 py-3.5 text-sm text-amber-900">
                  <p className="font-medium">
                    {t("auth.unlock.states.needsLogin.title")}
                  </p>
                  <p className="mt-1 text-amber-800">
                    {t("auth.unlock.states.needsLogin.description")}
                  </p>
                </div>
                <Button asChild className="w-full" size="lg">
                  <Link to="/">{t("auth.unlock.actions.goToLogin")}</Link>
                </Button>
              </div>
            )}

            {!isRestoring && !needsLogin && isVaultUnlocked && (
              <div className="flex items-center justify-center gap-3 rounded-xl border border-emerald-200 bg-emerald-50/50 px-4 py-6 text-sm text-emerald-700">
                <LoaderCircle className="h-5 w-5 animate-spin text-emerald-600" />
                <span>{t("auth.unlock.states.unlocked")}</span>
              </div>
            )}

            {!isRestoring && !needsLogin && !isVaultUnlocked && (
              <UnlockLockedForm
                form={form}
                account={account ?? null}
                biometricSupported={biometricSupported}
                biometricEnabled={biometricEnabled}
                canBiometricUnlock={canBiometricUnlock}
                feedback={feedback}
                isActionBlocked={isActionBlocked}
                isBiometricUnlocking={isBiometricUnlocking}
                isLoggingOut={isLoggingOut}
                isPinUnlocking={isPinUnlocking}
                onBiometricUnlock={onBiometricUnlock}
                onLogout={onLogout}
                onPinUnlock={onPinUnlock}
                onShowMasterPasswordUnlock={onShowMasterPasswordUnlock}
                onShowPinUnlock={onShowPinUnlock}
                onToggleShowPassword={onToggleShowPassword}
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
