import { LoaderCircle } from "lucide-react";
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
  LoginCredentialsFields,
  LoginFeedbackAlert,
  LoginHero,
  ServerUrlField,
  TwoFactorSection,
} from "@/features/auth/login/components";
import { useLoginFlow } from "@/features/auth/login/hooks";

type LoginPageProps = {
  navigateToVault: () => Promise<void>;
};

export function LoginPage({ navigateToVault }: LoginPageProps) {
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
    <main className="relative min-h-dvh overflow-hidden bg-[radial-gradient(circle_at_15%_15%,hsl(219_100%_97%),transparent_45%),radial-gradient(circle_at_85%_8%,hsl(210_100%_96%),transparent_40%),linear-gradient(130deg,hsl(220_46%_98%),hsl(0_0%_100%))] p-6 md:p-10">
      <div
        data-tauri-drag-region
        className="absolute inset-x-0 top-0 z-20 h-6"
      />
      <div className="absolute -top-24 -right-16 h-64 w-64 rounded-full bg-sky-300/15 blur-3xl" />
      <div className="absolute -bottom-28 -left-10 h-72 w-72 rounded-full bg-blue-500/10 blur-3xl" />

      <section className="relative mx-auto grid w-full max-w-6xl gap-8 md:min-h-[calc(100dvh-5rem)] md:grid-cols-[1.2fr_0.8fr] md:items-center">
        <LoginHero />

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
