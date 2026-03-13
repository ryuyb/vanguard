import { AlertCircle, ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { LoginFeedback } from "@/features/auth/login/types";

type LoginFeedbackAlertProps = {
  feedback: LoginFeedback;
};

export function LoginFeedbackAlert({ feedback }: LoginFeedbackAlertProps) {
  const { t } = useTranslation();

  if (feedback.kind === "idle") {
    return null;
  }

  const config = {
    error: {
      icon: AlertCircle,
      title: t("auth.feedback.login.error"),
      borderColor: "border-red-200/60",
      bgColor: "bg-red-50/50",
      iconColor: "text-red-600",
      titleColor: "text-red-900",
      textColor: "text-red-800",
    },
    success: {
      icon: ShieldCheck,
      title: t("auth.feedback.login.success"),
      borderColor: "border-emerald-200/60",
      bgColor: "bg-emerald-50/50",
      iconColor: "text-emerald-600",
      titleColor: "text-emerald-900",
      textColor: "text-emerald-800",
    },
    twoFactor: {
      icon: ShieldCheck,
      title: t("auth.feedback.login.twoFactorRequired"),
      borderColor: "border-blue-200/60",
      bgColor: "bg-blue-50/50",
      iconColor: "text-blue-600",
      titleColor: "text-blue-900",
      textColor: "text-blue-800",
    },
  }[feedback.kind];

  const Icon = config.icon;

  return (
    <div
      className={`rounded-xl border ${config.borderColor} ${config.bgColor} px-4 py-3.5 text-sm`}
    >
      <div className="flex items-start gap-3">
        <Icon className={`mt-0.5 h-5 w-5 flex-shrink-0 ${config.iconColor}`} />
        <div className="flex-1">
          <p className={`font-medium ${config.titleColor}`}>{config.title}</p>
          <p className={`mt-1 ${config.textColor}`}>{feedback.text}</p>
        </div>
      </div>
    </div>
  );
}
