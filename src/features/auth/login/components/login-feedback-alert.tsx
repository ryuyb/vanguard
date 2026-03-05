import { ShieldCheck } from "lucide-react";
import type { LoginFeedback } from "@/features/auth/login/types";

type LoginFeedbackAlertProps = {
  feedback: LoginFeedback;
};

export function LoginFeedbackAlert({ feedback }: LoginFeedbackAlertProps) {
  if (feedback.kind === "idle") {
    return null;
  }

  return (
    <div
      className={[
        "rounded-lg border px-3 py-2 text-sm",
        feedback.kind === "error" && "border-red-200 bg-red-50 text-red-700",
        feedback.kind === "success" &&
          "border-emerald-200 bg-emerald-50 text-emerald-700",
        feedback.kind === "twoFactor" &&
          "border-amber-200 bg-amber-50 text-amber-700",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {feedback.kind === "twoFactor" && (
        <ShieldCheck className="mr-1 inline size-4" />
      )}
      {feedback.text}
    </div>
  );
}
