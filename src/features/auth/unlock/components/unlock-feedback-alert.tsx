import { Lock, ShieldCheck } from "lucide-react";
import type { UnlockFeedback } from "@/features/auth/unlock/types";

type UnlockFeedbackAlertProps = {
  feedback: UnlockFeedback;
};

export function UnlockFeedbackAlert({ feedback }: UnlockFeedbackAlertProps) {
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
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {feedback.kind === "success" && (
        <ShieldCheck className="mr-1 inline size-4" />
      )}
      {feedback.kind === "error" && <Lock className="mr-1 inline size-4" />}
      {feedback.text}
    </div>
  );
}
