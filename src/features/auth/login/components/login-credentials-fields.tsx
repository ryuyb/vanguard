import { Eye, EyeOff, KeyRound, Mail } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Label } from "@/components/ui/label";
import type { LoginForm } from "@/features/auth/login/hooks/use-login-flow";
import { FormFieldError } from "@/features/auth/shared/form-field-error";

type LoginCredentialsFieldsProps = {
  form: LoginForm;
  showPassword: boolean;
  onToggleShowPassword: () => void;
  clearTwoFactorChallenge: () => void;
};

export function LoginCredentialsFields({
  form,
  showPassword,
  onToggleShowPassword,
  clearTwoFactorChallenge,
}: LoginCredentialsFieldsProps) {
  const { t } = useTranslation();

  return (
    <>
      <div className="space-y-2.5">
        <Label htmlFor="email" className="text-sm font-medium text-slate-700">
          {t("auth.login.form.email.label")}
        </Label>
        <form.Field name="email">
          {(field) => (
            <>
              <InputGroup>
                <InputGroupAddon>
                  <Mail className="h-5 w-5 text-slate-400" />
                </InputGroupAddon>
                <InputGroupInput
                  id="email"
                  type="email"
                  autoComplete="email"
                  placeholder={t("auth.login.form.email.placeholder")}
                  value={field.state.value}
                  onChange={(e) => {
                    clearTwoFactorChallenge();
                    field.handleChange(e.target.value);
                  }}
                  onBlur={field.handleBlur}
                  className="h-12 text-base"
                />
              </InputGroup>
              <FormFieldError errors={field.state.meta.errors} />
            </>
          )}
        </form.Field>
      </div>

      <div className="space-y-2.5">
        <Label
          htmlFor="master-password"
          className="text-sm font-medium text-slate-700"
        >
          {t("auth.login.form.masterPassword.label")}
        </Label>
        <form.Field name="masterPassword">
          {(field) => (
            <>
              <InputGroup>
                <InputGroupAddon>
                  <KeyRound className="h-5 w-5 text-slate-400" />
                </InputGroupAddon>
                <InputGroupInput
                  id="master-password"
                  type={showPassword ? "text" : "password"}
                  autoComplete="current-password"
                  placeholder={t("auth.login.form.masterPassword.placeholder")}
                  value={field.state.value}
                  onChange={(e) => {
                    clearTwoFactorChallenge();
                    field.handleChange(e.target.value);
                  }}
                  onBlur={field.handleBlur}
                  className="h-12 text-base"
                />
                <InputGroupAddon align="inline-end" className="px-1.5">
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon-sm"
                    className="text-slate-400 hover:text-slate-700 transition-colors"
                    onClick={onToggleShowPassword}
                    aria-label={
                      showPassword
                        ? t("auth.login.form.masterPassword.hidePassword")
                        : t("auth.login.form.masterPassword.showPassword")
                    }
                  >
                    {showPassword ? (
                      <EyeOff className="h-5 w-5" />
                    ) : (
                      <Eye className="h-5 w-5" />
                    )}
                  </Button>
                </InputGroupAddon>
              </InputGroup>
              <FormFieldError errors={field.state.meta.errors} />
            </>
          )}
        </form.Field>
      </div>
    </>
  );
}
