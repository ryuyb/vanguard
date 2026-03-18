import { Eye, EyeOff, LoaderCircle, ShieldCheck } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PasswordStrengthIndicator } from "@/features/auth/register/components/password-strength";
import type { PwnedCheckResult } from "@/features/auth/register/hooks/use-password-strength";
import { usePasswordStrength } from "@/features/auth/register/hooks/use-password-strength";

type PasswordSetupFormProps = {
  onSubmit: (password: string, passwordHint?: string) => Promise<void>;
  onCancel: () => void;
  isSubmitting: boolean;
};

export function PasswordSetupForm({
  onSubmit,
  onCancel,
  isSubmitting,
}: PasswordSetupFormProps) {
  const { t } = useTranslation();
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [passwordHint, setPasswordHint] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [pwnedCheck, setPwnedCheck] = useState<PwnedCheckResult | null>(null);
  const [isCheckingPwned, setIsCheckingPwned] = useState(false);

  const strengthResult = usePasswordStrength(password);

  // 验证状态
  const passwordTooShort = password.length > 0 && password.length < 8;
  const passwordMismatch =
    confirmPassword.length > 0 && password !== confirmPassword;
  const isValid =
    password.length >= 8 &&
    confirmPassword.length >= 8 &&
    password === confirmPassword;

  // 检查密码泄露
  const handleCheckPwned = async () => {
    if (password.length < 8) return;

    setIsCheckingPwned(true);
    try {
      const result = await strengthResult.checkPwned();
      setPwnedCheck(result);
    } finally {
      setIsCheckingPwned(false);
    }
  };

  // 提交表单
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!isValid || isSubmitting) return;

    // 如果还没检查过泄露，先检查
    if (!pwnedCheck && password.length >= 8) {
      await handleCheckPwned();
    }

    await onSubmit(password, passwordHint || undefined);
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-5">
      {/* 主密码输入 */}
      <div className="space-y-2">
        <Label htmlFor="password">
          {t("auth.register.form.masterPassword.label")}
        </Label>
        <div className="relative">
          <Input
            id="password"
            type={showPassword ? "text" : "password"}
            value={password}
            onChange={(e) => {
              setPassword(e.target.value);
              setPwnedCheck(null); // 重置泄露检查
            }}
            placeholder={t("auth.register.form.masterPassword.placeholder")}
            disabled={isSubmitting}
            className="pr-10"
          />
          <button
            type="button"
            onClick={() => setShowPassword(!showPassword)}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-700"
            disabled={isSubmitting}
          >
            {showPassword ? (
              <EyeOff className="h-4 w-4" />
            ) : (
              <Eye className="h-4 w-4" />
            )}
          </button>
        </div>

        {/* 密码强度指示器 */}
        {password.length > 0 && (
          <PasswordStrengthIndicator
            result={strengthResult}
            showFeedback={true}
          />
        )}

        {/* 密码长度错误 */}
        {passwordTooShort && (
          <p className="text-sm text-red-600">
            {t("auth.register.validation.passwordTooShort")}
          </p>
        )}
      </div>

      {/* 确认密码输入 */}
      <div className="space-y-2">
        <Label htmlFor="confirmPassword">
          {t("auth.register.form.confirmPassword.label")}
        </Label>
        <div className="relative">
          <Input
            id="confirmPassword"
            type={showConfirmPassword ? "text" : "password"}
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            placeholder={t("auth.register.form.confirmPassword.placeholder")}
            disabled={isSubmitting}
            className="pr-10"
          />
          <button
            type="button"
            onClick={() => setShowConfirmPassword(!showConfirmPassword)}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-700"
            disabled={isSubmitting}
          >
            {showConfirmPassword ? (
              <EyeOff className="h-4 w-4" />
            ) : (
              <Eye className="h-4 w-4" />
            )}
          </button>
        </div>

        {/* 密码不一致错误 */}
        {passwordMismatch && (
          <p className="text-sm text-red-600">
            {t("auth.register.validation.passwordMismatch")}
          </p>
        )}
      </div>

      {/* 密码提示（可选） */}
      <div className="space-y-2">
        <Label htmlFor="passwordHint">
          {t("auth.register.form.passwordHint.label")}
        </Label>
        <Input
          id="passwordHint"
          type="text"
          value={passwordHint}
          onChange={(e) => setPasswordHint(e.target.value)}
          placeholder={t("auth.register.form.passwordHint.placeholder")}
          disabled={isSubmitting}
        />
      </div>

      {/* 密码泄露检查 */}
      {password.length >= 8 && (
        <div className="space-y-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={handleCheckPwned}
            disabled={isCheckingPwned || isSubmitting}
            className="w-full"
          >
            {isCheckingPwned ? (
              <>
                <LoaderCircle className="h-4 w-4 animate-spin" />
                检查密码安全性...
              </>
            ) : (
              <>
                <ShieldCheck className="h-4 w-4" />
                检查密码是否泄露
              </>
            )}
          </Button>

          {/* 泄露检查结果 */}
          {pwnedCheck && !pwnedCheck.error && (
            <div
              className={`rounded-lg p-3 ${
                pwnedCheck.isPwned
                  ? "bg-red-50 text-red-700"
                  : "bg-emerald-50 text-emerald-700"
              }`}
            >
              <p className="text-sm font-medium">
                {pwnedCheck.isPwned
                  ? `⚠️ 此密码已在 ${pwnedCheck.count.toLocaleString()} 次数据泄露中出现`
                  : "✓ 此密码未在已知泄露数据库中"}
              </p>
              {pwnedCheck.isPwned && (
                <p className="mt-1 text-sm">建议使用其他密码以提高安全性</p>
              )}
            </div>
          )}

          {/* 检查错误 */}
          {pwnedCheck?.error && (
            <p className="text-sm text-slate-600">
              无法检查密码泄露状态: {pwnedCheck.error}
            </p>
          )}
        </div>
      )}

      {/* 操作按钮 */}
      <div className="flex gap-3">
        <Button
          type="button"
          variant="outline"
          onClick={onCancel}
          disabled={isSubmitting}
          className="flex-1"
        >
          {t("auth.register.actions.backToLogin")}
        </Button>
        <Button
          type="submit"
          disabled={!isValid || isSubmitting}
          className="flex-1 bg-emerald-600 hover:bg-emerald-700"
        >
          {isSubmitting ? (
            <>
              <LoaderCircle className="h-4 w-4 animate-spin" />
              {t("auth.register.actions.finishing")}
            </>
          ) : (
            t("auth.register.actions.finishRegistration")
          )}
        </Button>
      </div>
    </form>
  );
}
