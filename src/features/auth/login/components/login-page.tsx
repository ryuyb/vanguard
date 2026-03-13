import { KeyRound, LoaderCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  LoginCredentialsFields,
  LoginFeedbackAlert,
  ServerUrlField,
  TwoFactorSection,
} from "@/features/auth/login/components";
import { useLoginFlow } from "@/features/auth/login/hooks";

type LoginPageProps = {
  navigateToVault: () => Promise<void>;
};

export function LoginPage({ navigateToVault }: LoginPageProps) {
  const { t } = useTranslation();
  const {
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
  } = useLoginFlow({
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
                {t("auth.login.title")}
              </CardTitle>
              <p className="text-sm text-slate-600">
                {t("auth.login.subtitle")}
              </p>
            </div>
          </CardHeader>
          <CardContent className="pb-8">
            <form className="space-y-5" onSubmit={onSubmit}>
              {isRestoringSession && (
                <div className="flex items-center justify-center gap-3 rounded-xl border border-slate-200 bg-slate-50/50 px-4 py-6 text-sm text-slate-700">
                  <LoaderCircle className="h-5 w-5 animate-spin text-blue-600" />
                  <span>{t("auth.login.states.checkingSession")}</span>
                </div>
              )}

              <ServerUrlField
                customBaseUrl={customBaseUrl}
                serverUrlOption={serverUrlOption}
                isSubmitting={isSubmitting}
                onServerUrlOptionChange={onServerUrlOptionChange}
                onCustomBaseUrlChange={onCustomBaseUrlChange}
              />

              <LoginCredentialsFields
                email={email}
                masterPassword={masterPassword}
                showPassword={showPassword}
                isSubmitting={isSubmitting}
                onEmailChange={onEmailChange}
                onMasterPasswordChange={onMasterPasswordChange}
                onToggleShowPassword={onToggleShowPassword}
              />

              {twoFactorState && (
                <TwoFactorSection
                  state={twoFactorState}
                  isSubmitting={isSubmitting}
                  onProviderChange={onTwoFactorProviderChange}
                  onTokenChange={onTwoFactorTokenChange}
                  onSendEmailCode={onSendEmailCode}
                />
              )}

              <LoginFeedbackAlert feedback={feedback} />

              <Button
                type="submit"
                size="lg"
                className="h-12 w-full bg-blue-600 text-base font-medium hover:bg-blue-700 transition-colors"
                disabled={!canSubmit}
              >
                {isSubmitting && (
                  <LoaderCircle className="h-5 w-5 animate-spin" />
                )}
                {isSubmitting
                  ? submitProgressText || t("auth.login.actions.submitting")
                  : twoFactorState
                    ? t("auth.login.actions.verifyAndContinue")
                    : t("auth.login.actions.submit")}
              </Button>
            </form>
          </CardContent>
        </Card>
      </section>
    </main>
  );
}
