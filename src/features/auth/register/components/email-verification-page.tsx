import { Mail } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";

type EmailVerificationPageProps = {
  email: string;
  onBackToEdit: () => void;
  onBackToLogin: () => Promise<void>;
};

export function EmailVerificationPage({
  email,
  onBackToEdit,
  onBackToLogin,
}: EmailVerificationPageProps) {
  const { t } = useTranslation();

  return (
    <main className="relative flex min-h-dvh items-center justify-center overflow-hidden bg-gradient-to-br from-slate-50 via-blue-50/30 to-slate-100 p-6">
      <div
        data-tauri-drag-region
        className="absolute inset-x-0 top-0 z-20 h-6"
      />

      <section className="relative mx-auto w-full max-w-md">
        <Card className="border-slate-200/60 bg-white shadow-2xl shadow-slate-900/5">
          <CardHeader className="space-y-4 pb-2">
            <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-emerald-600 to-emerald-700 shadow-lg shadow-emerald-600/25">
              <Mail className="h-8 w-8 text-white" strokeWidth={2.5} />
            </div>
            <h2 className="text-center text-2xl font-semibold tracking-tight text-slate-900">
              {t("auth.register.emailVerification.title")}
            </h2>
          </CardHeader>
          <CardContent className="space-y-6 pb-8">
            <p className="text-center text-sm text-slate-600">
              {t("auth.register.emailVerification.description", { email })}
            </p>

            <p className="text-center text-sm text-slate-500">
              {t("auth.register.emailVerification.notReceived")}{" "}
              <button
                type="button"
                onClick={onBackToEdit}
                className="text-emerald-600 hover:text-emerald-700 font-medium transition-colors"
              >
                {t("auth.register.emailVerification.editEmail")}
              </button>
            </p>

            <Button
              size="lg"
              variant="outline"
              className="h-12 w-full text-base"
              onClick={onBackToLogin}
            >
              {t("auth.register.emailVerification.backToLogin")}
            </Button>
          </CardContent>
        </Card>
      </section>
    </main>
  );
}
