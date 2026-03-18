import { CheckCircle2, Mail } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { RegistrationFeedbackState } from "@/features/auth/register/types";

type RegistrationFeedbackProps = {
  feedback: RegistrationFeedbackState;
};

export function RegistrationFeedback({ feedback }: RegistrationFeedbackProps) {
  const { t } = useTranslation();

  if (feedback.kind === "idle") return null;

  const configs = {
    emailSent: {
      icon: Mail,
      title: t("auth.register.messages.emailVerificationRequired.title"),
      border: "border-emerald-200/60",
      bg: "bg-emerald-50/50",
      iconColor: "text-emerald-600",
      titleColor: "text-emerald-900",
      textColor: "text-emerald-800",
    },
    directRegistration: {
      icon: CheckCircle2,
      title: t("auth.feedback.register.success"),
      border: "border-blue-200/60",
      bg: "bg-blue-50/50",
      iconColor: "text-blue-600",
      titleColor: "text-blue-900",
      textColor: "text-blue-800",
    },
  } as const;

  const config = configs[feedback.kind];
  const Icon = config.icon;

  return (
    <div
      className={`rounded-xl border ${config.border} ${config.bg} px-4 py-3.5 text-sm`}
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
