import { AlertCircle, CheckCircle2, ShieldAlert } from "lucide-react";
import type {
  PasswordStrength,
  PasswordStrengthResult,
} from "@/features/auth/register/hooks/use-password-strength";

type PasswordStrengthIndicatorProps = {
  result: PasswordStrengthResult;
  showFeedback?: boolean;
};

const strengthConfig: Record<
  PasswordStrength,
  {
    label: string;
    color: string;
    bgColor: string;
    textColor: string;
    icon: typeof AlertCircle;
  }
> = {
  weak: {
    label: "弱",
    color: "bg-red-500",
    bgColor: "bg-red-50",
    textColor: "text-red-700",
    icon: AlertCircle,
  },
  fair: {
    label: "一般",
    color: "bg-orange-500",
    bgColor: "bg-orange-50",
    textColor: "text-orange-700",
    icon: ShieldAlert,
  },
  good: {
    label: "良好",
    color: "bg-blue-500",
    bgColor: "bg-blue-50",
    textColor: "text-blue-700",
    icon: CheckCircle2,
  },
  strong: {
    label: "强",
    color: "bg-emerald-500",
    bgColor: "bg-emerald-50",
    textColor: "text-emerald-700",
    icon: CheckCircle2,
  },
};

export function PasswordStrengthIndicator({
  result,
  showFeedback = true,
}: PasswordStrengthIndicatorProps) {
  const config = strengthConfig[result.strength];
  const Icon = config.icon;

  return (
    <div className="space-y-2">
      {/* 强度条 */}
      <div className="flex items-center gap-2">
        <div className="flex-1 flex gap-1">
          {[0, 1, 2, 3].map((index) => (
            <div
              key={index}
              className={`h-1.5 flex-1 rounded-full transition-colors ${
                index < result.score ? config.color : "bg-slate-200"
              }`}
            />
          ))}
        </div>
        <div className="flex items-center gap-1.5">
          <Icon className={`h-4 w-4 ${config.textColor}`} />
          <span className={`text-sm font-medium ${config.textColor}`}>
            {config.label}
          </span>
        </div>
      </div>

      {/* 反馈建议 */}
      {showFeedback && result.feedback.length > 0 && (
        <div className={`rounded-lg p-3 ${config.bgColor}`}>
          <ul className={`space-y-1 text-sm ${config.textColor}`}>
            {result.feedback.map((item, index) => (
              <li key={index} className="flex items-start gap-2">
                <span className="mt-0.5">•</span>
                <span>{item}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
