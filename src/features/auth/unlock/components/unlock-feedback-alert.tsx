import { AlertCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { UnlockFeedback } from "@/features/auth/unlock/types";

type UnlockFeedbackAlertProps = {
  feedback: UnlockFeedback;
};

export function UnlockFeedbackAlert({ feedback }: UnlockFeedbackAlertProps) {
  const { t } = useTranslation();

  if (feedback.kind === "idle") {
    return null;
  }

  // 只处理错误状态，成功状态会立即跳转不会显示
  if (feedback.kind === "error") {
    return (
      <div className="rounded-xl border border-red-200/60 bg-red-50/50 px-4 py-3.5 text-sm">
        <div className="flex items-start gap-3">
          <AlertCircle className="mt-0.5 h-5 w-5 flex-shrink-0 text-red-600" />
          <div className="flex-1">
            <p className="font-medium text-red-900">
              {t("auth.feedback.unlock.error")}
            </p>
            <p className="mt-1 text-red-800">{feedback.text}</p>
          </div>
        </div>
      </div>
    );
  }

  return null;
}
