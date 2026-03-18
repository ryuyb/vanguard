import { UserPlus, LoaderCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { RegisterForm } from "@/features/auth/register/components/register-form";
import { RegistrationFeedback } from "@/features/auth/register/components/registration-feedback";
import { useRegistrationFlow } from "@/features/auth/register/hooks/use-registration-flow";

type RegisterPageProps = {
  navigateToLogin: () => Promise<void>;
};

export function RegisterPage({ navigateToLogin }: RegisterPageProps) {
  const { t } = useTranslation();
  const { form, feedback, submitProgressText } = useRegistrationFlow();

  return (
    <main className="relative flex min-h-dvh items-center justify-center overflow-hidden bg-gradient-to-br from-slate-50 via-blue-50/30 to-slate-100 p-6">
      <div
        data-tauri-drag-region
        className="absolute inset-x-0 top-0 z-20 h-6"
      />

      <section className="relative mx-auto w-full max-w-md">
        <Card className="border-slate-200/60 bg-white shadow-2xl shadow-slate-900/5">
          <CardHeader className="space-y-4 pb-6">
            <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-emerald-600 to-emerald-700 shadow-lg shadow-emerald-600/25">
              <UserPlus className="h-8 w-8 text-white" strokeWidth={2.5} />
            </div>
            <div className="space-y-1 text-center">
              <CardTitle className="text-2xl font-semibold tracking-tight text-slate-900">
                {t("auth.register.title")}
              </CardTitle>
              <p className="text-sm text-slate-600">
                {t("auth.register.subtitle")}
              </p>
            </div>
          </CardHeader>
          <CardContent className="pb-8">
            <form
              className="space-y-5"
              onSubmit={(e) => {
                e.preventDefault();
                e.stopPropagation();
                form.handleSubmit();
              }}
            >
              <RegisterForm form={form} />

              <RegistrationFeedback feedback={feedback} />

              <form.Subscribe selector={(s) => [s.canSubmit, s.isSubmitting]}>
                {([canSubmit, isSubmitting]) => (
                  <Button
                    type="submit"
                    size="lg"
                    className="h-12 w-full bg-emerald-600 text-base font-medium hover:bg-emerald-700 transition-colors"
                    disabled={!canSubmit || feedback.kind === "emailSent" || feedback.kind === "disabled"}
                  >
                    {isSubmitting && (
                      <LoaderCircle className="h-5 w-5 animate-spin" />
                    )}
                    {isSubmitting
                      ? submitProgressText || t("auth.register.actions.submitting")
                      : t("auth.register.actions.submit")}
                  </Button>
                )}
              </form.Subscribe>

              <div className="text-center">
                <button
                  type="button"
                  onClick={navigateToLogin}
                  className="text-sm text-slate-500 hover:text-slate-700 transition-colors"
                >
                  {t("auth.register.actions.backToLogin")}
                </button>
              </div>
            </form>
          </CardContent>
        </Card>
      </section>
    </main>
  );
}
